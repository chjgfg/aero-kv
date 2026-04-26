// # set_admin / set_pause 接口
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;
use solana_sdk::pubkey::Pubkey;
use std::{str::FromStr, sync::Arc};

use crate::{AppState, auth, error::Error};

#[derive(Debug, serde::Deserialize)]
pub struct AuthQuery {
    pub new_admin: Option<String>,
    pub paused: Option<bool>,
}

// 权限接口示例
pub async fn init_auth(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("init auth");
    let chain = state.chain.clone();
    let _ = auth::init_auth(chain).await;
    (StatusCode::OK, "init auth success")
}

pub async fn set_admin(
    State(state): State<Arc<AppState>>,
    Query(req): Query<AuthQuery>,
) -> impl IntoResponse {
    let Ok(new_admin) = req.new_admin.ok_or(|e| Error::InvalidParam(e)) else {
        return (StatusCode::BAD_REQUEST, "set admin error");
    };
    info!("set admin new_admin: {}", new_admin);
    let chain = state.chain.clone();
    let pubkey = Pubkey::from_str(new_admin.as_str()).unwrap();
    let _ = auth::set_admin(chain, pubkey).await;
    (StatusCode::OK, "set admin success")
}

pub async fn set_pause(
    State(state): State<Arc<AppState>>,
    Query(req): Query<AuthQuery>,
) -> impl IntoResponse {
    let Ok(paused) = req.paused.ok_or(|e| Error::InvalidParam(e)) else {
        return (StatusCode::BAD_REQUEST, "set pause error");
    };
    info!("set pause paused: {}", paused);
    let chain = state.chain.clone();
    let _ = auth::set_pause(chain, paused).await;
    (StatusCode::OK, "set pause success")
}
