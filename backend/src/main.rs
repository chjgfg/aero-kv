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

use crate::api::raft_api;
use crate::block_chain::client::ChainClient;
use crate::config::log_config::log_config;
use crate::raft::server::Server;
use crate::raft::types::AppContext;
use crate::storage::engine::DiskClient;
use crate::error::Result;
use crate::{config::env_config::Config, error::Error};
use axum::{
    Router,
    routing::{delete, get, post},
};
use log::info;
use std::{env, fs};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

// 给 ChainClient 加 Arc 包装，满足 Clone 约束（Axum State 要求 Clone）
// 1. 给两个状态都包上 Arc（满足 Clone + Send + Sync + 'static）
pub type ChainState = Arc<ChainClient>;
pub type StorageState = Arc<Mutex<DiskClient>>;

// 2. 定义统一的顶层状态结构体，派生 FromRef
// #[derive(Clone, FromRef)]
// pub struct AppState {
//     pub chain: ChainState,
//     pub storage: StorageState,
// }

#[tokio::main]
async fn main() -> Result<()> {
    let dir = PathBuf::from("./kv");
    if !dir.exists() {
        fs::create_dir(dir).unwrap();
    }

    // 解析参数: cargo run -- <ID> <PORT> <PEER_IDS...>
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: aerokv <id> <port> [peer_ids...]");
        return Ok(());
    }

    let node_id: u64 = args[1].parse().expect("Invalid ID");
    let port: u16 = args[2].parse().expect("Invalid Port");
    let peer_ids: Vec<u64> = args[3..].iter().map(|s| s.parse().expect("Invalid Peer ID")).collect();

    let kv_log = format!("./kv/{}/kv.log", port);
    let raft_log = format!("./kv/{}/raft.log", port);
    let app_log = format!("./kv/{}/app.log", port);

    let raft = DiskClient::new(PathBuf::from(raft_log))?;
    let storage = Arc::new(RwLock::new(raft));

    // 创建各种通道
    let (task_tx, task_rx) = crossbeam::channel::unbounded();
    let (peers_tx, peers_rx) = crossbeam::channel::unbounded();
    let (ticker_tx, ticker_rx) = crossbeam::channel::unbounded();

    // 启动时钟线程 (每 100ms 触发一次 Raft tick)
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(100));
        ticker_tx.send(Instant::now()).ok();
    });

    let _ = log_config(app_log.as_str());
    let config = Config::from_env().map_err(|e| Error::ConfigError(e.to_string()))?;
    info!("开始创建client");

    let chain = Arc::new(ChainClient::new(&config)?);
    let disk = Arc::new(Mutex::new(DiskClient::new(PathBuf::from(kv_log))?));

    // 启动 Raft 核心线程
    let storage_for_raft = storage.clone();
    let chain_for_raft = chain.clone();
    let kv_storage_for_raft = disk.clone();

    let handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        let _guard = handle.enter(); // 🚀 必须进入句柄，spawn 才能在独立线程生效
        let server = Server::new(node_id, peer_ids, storage_for_raft, chain_for_raft,  kv_storage_for_raft);
        server.run(ticker_rx, peers_rx, task_rx);
    });



    let chain_for_storage = chain.clone();
    let kv_storage_for_storage = disk.clone();
    // 配置 Axum
    let state =  Arc::new(AppContext {
        node_id,
        task_sender: task_tx,
        peers_sender: peers_tx,
        kv_store: storage,
        chain: chain_for_storage,
        storage: kv_storage_for_storage,
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
        .route("/raft/message", post(raft_api::handle_raft_msg)) // 🚀 必须加这一行！
        .route("/health", get(api::health))
        // KV 接口
        .route("/kv/init-storage", post(api::init_storage))
        .route("/kv/upsert", post(raft_api::kv_set))
        .route("/kv/delete", delete(raft_api::kv_set))
        .route("/kv/get", get(api::gets))
        .route("/kv/scan", post(api::scan))
        .route("/kv/page", post(api::page))
        // 权限接口
        .route("/auth/init-admin", post(api::init_auth))
        .route("/auth/set-admin", post(raft_api::kv_set))
        .route("/auth/set-pause", post(raft_api::kv_set))
        // 手续费接口
        .route("/fee/init-fee", post(api::init_fee))
        .route("/fee/set-fee", post(raft_api::kv_set))
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
