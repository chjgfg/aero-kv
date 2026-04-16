// # set_admin / set_pause 接口
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct AuthQuery {
    pub new_admin: String,
    pub paused: bool,
}

// 权限接口示例
pub async fn init_auth(State(client): State<AppState>) -> impl IntoResponse {
    info!("init auth");
    let _ = client.init_auth().await;
    (StatusCode::OK, "init auth success")
}

pub async fn set_admin(
    State(client): State<AppState>,
    Query(req): Query<AuthQuery>,
) -> impl IntoResponse {
    info!("set admin new_admin: {}", req.new_admin);
    let pubkey = Pubkey::from_str(req.new_admin.as_str()).unwrap();
    let _ = client.set_admin(pubkey).await;
    (StatusCode::OK, "set admin success")
}

pub async fn set_pause(
    State(client): State<AppState>,
    Query(req): Query<AuthQuery>,
) -> impl IntoResponse {
    info!("set pause paused: {}", req.paused);
    let _ = client.set_pause(req.paused).await;
    (StatusCode::OK, "set pause success")
}
