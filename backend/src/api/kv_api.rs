// # upsert / get / scan 接口
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::AppState;

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
    client.init_storage().await;
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "init storage success")
}

pub async fn upsert(
    State(client): State<AppState>,
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    let k = req.key.into_bytes();
    let v = req.value.into_bytes();
    client.upsert(k, v).await;
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "upsert success")
}

pub async fn delete(
    State(client): State<AppState>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    let k = req.key.into_bytes();
    client.delete(k).await;
    (StatusCode::OK, "delete success")
}

pub async fn gets(State(client): State<AppState>, Query(req): Query<KVQuery>) -> impl IntoResponse {
    let k = req.key.into_bytes();
    client.get(k).await;
    (StatusCode::OK, "get success")
}

pub async fn scan(State(client): State<AppState>, Json(req): Json<KVRequest>) -> impl IntoResponse {
    let k = req.key.into_bytes();
    let l = req.limit;
    client.scan(k, l).await;
    (StatusCode::OK, "scan success")
}
