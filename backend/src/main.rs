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
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

// 给 ChainClient 加 Arc 包装，满足 Clone 约束（Axum State 要求 Clone）
// 1. 给两个状态都包上 Arc（满足 Clone + Send + Sync + 'static）
pub type ChainState = Arc<ChainClient>;
pub type StorageState = Arc<Mutex<DiskClient>>;

// 2. 定义统一的顶层状态结构体，派生 FromRef
#[derive(Clone, FromRef)]
pub struct AppState {
    pub chain: ChainState,
    pub storage: StorageState,
    // 用 Mutex 包裹，实现线程安全的修改
    pub merkle_tree: Arc<Mutex<MerkleTree<Sha256>>>,
    pub leaf_hashes: Arc<Mutex<Vec<[u8; 32]>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let dir = PathBuf::from("./kv");
    if !dir.exists() {
        fs::create_dir(dir).unwrap();
    }
    let kv_log = format!("./kv/kv.log");
    let app_log = format!("./kv/app.log");

    let _ = log_config(app_log.as_str());
    let config = Config::from_env().map_err(|e| Error::ConfigError(e.to_string()))?;
    info!("开始创建client");
    let chain = ChainClient::new(&config)?;
    let disk = DiskClient::new(PathBuf::from(kv_log))?;

    // 用 Arc 包装，再组合成 AppState
    let state : Arc<AppState> = Arc::new(AppState {
        chain: Arc::new(chain),
        storage: Arc::new(Mutex::new(disk)),
        merkle_tree: Arc::new(Mutex::new(MerkleTree::new())),
        leaf_hashes: Arc::new(Mutex::new(Vec::new())),
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
        .route("/health", get(api::health))
        // KV 接口
        .route("/kv/init-storage", post(api::init_storage))
        .route("/kv/upsert", post(api::upsert))
        .route("/kv/delete", delete(api::delete))
        .route("/kv/get", get(api::gets))
        .route("/kv/scan", post(api::scan))
        .route("/kv/page", post(api::page))
        // 权限接口
        .route("/auth/init-admin", post(api::init_auth))
        .route("/auth/set-admin", post(api::set_admin))
        .route("/auth/set-pause", post(api::set_pause))
        // 手续费接口
        .route("/fee/init-fee", post(api::init_fee))
        .route("/fee/set-fee", post(api::set_fee))
        // 添加 CORS 中间件，允许所有来源（开发用，生产环境限制域名）
        .layer(cors)
        // 注入状态（Arc<ChainClient>）
        .with_state(state);

    // 5. 启动服务
    let addr = format!("0.0.0.0:{}", config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    log::info!("🚀 后端服务启动成功，监听端口: {}", config.server_port);

    // 启动 Axum 服务
    axum::serve(listener, app).await?;

    Ok(())
}
