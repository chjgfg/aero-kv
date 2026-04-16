mod api;
mod auth;
mod chain;
mod config;
mod constants;
mod core;
mod error;
mod fee;
mod utils;

use std::sync::Arc;
use tokio::net::TcpListener;

use log::info;

use crate::chain::client::ChainClient;
use crate::config::log_config::log_config;
use crate::error::Result;
use crate::{config::env_config::Config, error::Error};
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    serve::Serve,
};

// 给 ChainClient 加 Arc 包装，满足 Clone 约束（Axum State 要求 Clone）
type AppState = Arc<ChainClient>;

// 示例 KV 接口（你可以替换成自己的业务逻辑）
async fn upsert(State(client): State<AppState>, key: Vec<u8>, value: Vec<u8>) -> impl IntoResponse {
    client.upsert(key, value);
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "upsert success")
}

async fn gets(State(client): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "get success")
}

async fn scan(State(client): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "scan success")
}

// 权限接口示例
async fn set_admin(State(client): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "set admin success")
}

async fn set_pause(State(client): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "set pause success")
}

// 手续费接口示例
async fn set_fee(State(client): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "set fee success")
}

async fn health(State(client): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, "set fee success")
}

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
        .route("/test", get(health))
        // KV 接口
        // .route("/kv/upsert", post(upsert))
        // .route("/kv/get", get(gets))
        // .route("/kv/scan", get(scan))
        // // 权限接口
        // .route("/auth/set-admin", post(set_admin))
        // .route("/auth/set-pause", post(set_pause))
        // // 手续费接口
        // .route("/fee/set-fee", post(set_fee))
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
