mod api;
mod auth;
mod block_chain;
mod config;
mod constants;
mod core;
mod storage;
mod error;
mod fee;
mod utils;
mod raft;

use crate::block_chain::client::ChainClient;
use crate::config::log_config::log_config;
use crate::storage::engine::DiskClient;
use crate::error::Result;
use crate::{config::env_config::Config, error::Error};
use axum::extract::FromRef;
use axum::{
    Router,
    routing::{delete, get, post},
};
use log::info;
use rs_merkle::{MerkleTree, algorithms::Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tokio::sync::{Mutex};

use openraft::{Config as RaftConfig, Raft};
use crate::raft::network::Network;
use crate::raft::log_storage::MyLogStorage;
use crate::raft::state_machine::MyStateMachine;
use crate::raft::types::{RaftConfig as MyRaftConfig};

// 给 ChainClient 加 Arc 包装，满足 Clone 约束（Axum State 要求 Clone）
// 1. 给两个状态都包上 Arc（满足 Clone + Send + Sync + 'static）
pub type ChainState = Arc<ChainClient>;
pub type StorageState = Arc<Mutex<DiskClient>>;
pub type MyRaft = Raft<MyRaftConfig>; // 定义 Raft 实例类型

// 2. 定义统一的顶层状态结构体，派生 FromRef
#[derive(Clone, FromRef)]
pub struct AppState {
    pub chain: ChainState,
    pub storage: StorageState,
    // 用 Mutex 包裹，实现线程安全的修改
    pub merkle_tree: Arc<Mutex<MerkleTree<Sha256>>>,
    pub leaf_hashes: Arc<Mutex<Vec<[u8; 32]>>>,
    // --- 新增 Raft 相关状态 ---
    pub raft: Arc<MyRaft>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 获取命令行参数：cargo run -- <NODE_ID> <PORT>
    let args: Vec<String> = std::env::args().collect();
    let node_id: u64 = args.get(1).expect("Missing NodeID").parse().expect("Invalid NodeID");
    let port: u16 = args.get(2).expect("Missing Port").parse().expect("Invalid Port");

    // 2. 为每个节点创建独立的存储目录，避免文件锁冲突
    let base_dir = format!("./kv/node_{}", node_id);
    let dir = PathBuf::from(&base_dir);
    if !dir.exists() {
        fs::create_dir_all(&dir).unwrap();
    }

    let kv_log = dir.join("kv.log");
    let app_log = dir.join("app.log");
    let raft_log = dir.join("raft.log");
    

    let _ = log_config(app_log.to_str().unwrap());
    let config = Config::from_env().map_err(|e| Error::ConfigError(e.to_string()))?;
    info!("开始创建client");
    let chain = Arc::new(ChainClient::new(&config)?);
    let disk = Arc::new(Mutex::new(DiskClient::new(PathBuf::from(kv_log))?));
    // let disk = Arc::new(RwLock::new(DiskClient::new(PathBuf::from(kv_log))?));
    let raft = Arc::new(Mutex::new(DiskClient::new(PathBuf::from(raft_log))?));

    // --- 1. 先创建共享的内存状态 ---
    let merkle_tree = Arc::new(tokio::sync::Mutex::new(MerkleTree::new()));
    let leaf_hashes = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // 3. 初始化 Raft 存储层与网络层
    let log_store = MyLogStorage { db: raft };
    let sm_store = MyStateMachine { 
        db: disk.clone(), 
        merkle_tree: merkle_tree.clone(), // 共享引用
        leaf_hashes: leaf_hashes.clone(), // 共享引用
    };
    let network = Network::new();

    // 4. 配置 Raft 参数
    let raft_config = Arc::new(RaftConfig {
        heartbeat_interval: 500,
        election_timeout_min: 1500,
        election_timeout_max: 3000,
        ..Default::default()
    }.validate().unwrap());

    // 5. 创建 Raft 实例[cite: 1]
    // 注意：NodeId 应该从配置文件读取，这里暂时硬编码为 1
    // let node_id: NodeId = 1; 
    let raft = MyRaft::new(node_id, raft_config, network, log_store, sm_store).await.unwrap();
    let raft = Arc::new(raft);


    // 用 Arc 包装，再组合成 AppState
    let state : Arc<AppState> = Arc::new(AppState {
        chain: chain,
        storage: disk,
        merkle_tree,
        leaf_hashes,
        // 修正：添加缺失的 raft 字段
        raft: raft.clone(),
    });

    println!("✅ 链客户端初始化完成，程序ID: {}", state.chain.program_id);

    // 在你创建路由的地方加上这段 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(Any)
        .allow_credentials(false);

    // 4. 注册路由（Axum 0.8.x 标准写法）
    let app = Router::new()
        // --- Raft 内部 RPC 路由 (必须添加) ---[cite: 1]
        .route("/raft/init", post(api::raft_init))
        .route("/raft/append", post(api::raft_append))
        .route("/raft/vote", post(api::raft_vote))
        .route("/raft/snapshot", post(api::raft_snapshot))
        
        .route("/health", get(api::health))
        // KV 接口
        .route("/kv/init-storage", post(api::init_storage))
        .route("/kv/init-counter", post(api::init_counter))
        .route("/kv/upsert", post(api::raft_upsert))
        .route("/kv/delete", delete(api::raft_delete))

        // .route("/kv/get", get(api::raft_gets))
        // .route("/kv/scan", post(api::raft_scan))
        // .route("/kv/page", post(api::raft_page))

        .route("/kv/get", get(api::gets))
        .route("/kv/scan", post(api::scan))
        .route("/kv/page", post(api::page))

        // 权限接口
        .route("/auth/init-admin", post(api::init_auth))
        .route("/auth/set-pause", post(api::raft_pause))
        // 手续费接口
        .route("/fee/init-fee", post(api::init_fee))
        .route("/fee/set-fee", post(api::raft_fee))
        // 添加 CORS 中间件，允许所有来源（开发用，生产环境限制域名）
        .layer(cors)
        // 注入状态（Arc<ChainClient>）
        .with_state(state);

    // 5. 启动服务
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    log::info!("🚀 后端服务启动成功，监听端口: {}", port);

    // 启动 Axum 服务
    axum::serve(listener, app).await?;

    Ok(())
}
