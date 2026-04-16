use crate::chain::client::ChainClient;
use crate::{config::Config, error::Error};

mod api;
mod auth;
mod chain;
mod config;
mod constants;
mod core;
mod error;
mod fee;
mod log;
mod utils;

use crate::error::Result;

fn main() -> Result<()> {
    let config = Config::from_env().map_err(|e| Error::ConfigError("()".to_string()))?;
    println!("{:?}", config);
    let client = ChainClient::new(&config)?;

    println!("✅ 链客户端初始化完成，程序ID: {}", client.program_id);

    // // 注册路由
    // let app = Router::new()
    //     // KV 接口
    //     .route("/kv/upsert", post(kv_api::upsert))
    //     .route("/kv/get", get(kv_api::get))
    //     .route("/kv/scan", get(kv_api::scan))
    //     // 权限接口
    //     .route("/auth/set-admin", post(auth_api::set_admin))
    //     .route("/auth/set-pause", post(auth_api::set_pause))
    //     // 手续费接口
    //     .route("/fee/set-fee", post(fee_api::set_fee))
    //     // 注入依赖
    //     .with_state(chain_client);

    // // 启动服务
    // let addr = format!("0.0.0.0:{}", config.server_port);
    // let listener = TcpListener::bind(&addr).await?;
    // println!("🚀 后端服务启动成功，监听端口: {}", config.server_port);

    // axum::serve(listener, app).await?;
    Ok(())
}
