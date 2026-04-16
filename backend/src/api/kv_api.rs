// # upsert / get / scan 接口
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;

use crate::{AppState, chain};

#[derive(Debug, serde::Deserialize)]
pub struct KVRequest {
    pub key: String,
    pub value: String,
    pub limit: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct KVQuery {
    pub key: String,
}

// 示例 KV 接口（你可以替换成自己的业务逻辑）
pub async fn init_storage(State(client): State<AppState>) -> impl IntoResponse {
    info!("init storage");
    let _ = chain::init_storage(client).await;
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "init storage success")
}

pub async fn upsert(
    State(client): State<AppState>,
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    info!("upsert key: {}, value: {}", req.key, req.value);
    let k = req.key.into_bytes();
    let v = req.value.into_bytes();
    let _ = chain::upsert(client, k, v).await;
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "upsert success")
}

pub async fn delete(
    State(client): State<AppState>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    info!("delete key: {}", req.key);
    let k = req.key.into_bytes();
    let _ = chain::delete(client, k).await;
    (StatusCode::OK, "delete success")
}

pub async fn gets(State(client): State<AppState>, Query(req): Query<KVQuery>) -> impl IntoResponse {
    info!("get key: {}", req.key);
    let k = req.key.into_bytes();
    let _ = chain::get(client, k).await;
    (StatusCode::OK, "get success")
}

pub async fn scan(State(client): State<AppState>, Json(req): Json<KVRequest>) -> impl IntoResponse {
    info!("scan key: {}, limit: {}", req.key, req.limit);
    let k = req.key.into_bytes();
    let l = req.limit;
    let _ = chain::scan(client, k, l).await;
    (StatusCode::OK, "scan success")
}
