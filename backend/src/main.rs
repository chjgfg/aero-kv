mod api;
mod auth;
mod chain;
mod config;
mod constants;
mod core;
mod error;
mod fee;
mod utils;

use crate::chain::client::ChainClient;
use crate::config::log_config::log_config;
use crate::error::Result;
use crate::{config::env_config::Config, error::Error};
use axum::{
    Router,
    routing::{delete, get, post},
};
use log::info;
use std::sync::Arc;
use tokio::net::TcpListener;

// 给 ChainClient 加 Arc 包装，满足 Clone 约束（Axum State 要求 Clone）
type AppState = Arc<ChainClient>;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = log_config();
    let config = Config::from_env().map_err(|e| Error::ConfigError(e.to_string()))?;
    info!("开始创建client");
    let client = ChainClient::new(&config)?;
    let state = Arc::new(client); // 用 Arc 包装，满足 Clone 约束

    println!("✅ 链客户端初始化完成，程序ID: {}", state.program_id);

    // 4. 注册路由（Axum 0.8.x 标准写法）
    let app = Router::new()
        .route("/health", get(api::health))
        // KV 接口
        .route("/kv/init-storage", post(api::init_storage))
        .route("/kv/upsert", post(api::upsert))
        .route("/kv/delete", delete(api::delete))
        .route("/kv/get", get(api::gets))
        .route("/kv/scan", get(api::scan))
        // 权限接口
        .route("/auth/init-admin", post(api::init_auth))
        .route("/auth/set-admin", post(api::set_admin))
        .route("/auth/set-pause", post(api::set_pause))
        // 手续费接口
        .route("/fee/init-fee", post(api::init_fee))
        .route("/fee/set-fee", post(api::set_fee))
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
