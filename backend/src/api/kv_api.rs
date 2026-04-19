use std::sync::{Arc, Mutex};

// # upsert / get / scan 接口
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;

use crate::{
    block_chain::{self, client::ChainClient},
    storage::engine::DiskClient,
};

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
pub async fn init_storage(State(chain): State<Arc<ChainClient>>) -> impl IntoResponse {
    info!("init storage");
    let _ = block_chain::init_storage(chain).await;
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "init storage success")
}

pub async fn upsert(
    State(chain): State<Arc<ChainClient>>,
    State(storage): State<Arc<Mutex<DiskClient>>>,
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    info!("upsert key: {}, value: {}", req.key, req.value);
    let k = req.key.into_bytes();
    let v = req.value.into_bytes();
    let _ = block_chain::upsert(chain, storage, k, v).await;
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "upsert success")
}

pub async fn delete(
    State(chain): State<Arc<ChainClient>>,
    State(storage): State<Arc<Mutex<DiskClient>>>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    info!("delete key: {}", req.key);
    let k = req.key.into_bytes();
    let _ = block_chain::delete(chain, storage, k).await;
    (StatusCode::OK, "delete success")
}

pub async fn gets(
    State(chain): State<Arc<ChainClient>>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    info!("get key: {}", req.key);
    let k = req.key.into_bytes();
    let _ = block_chain::get(chain, k).await;
    (StatusCode::OK, "get success")
}

pub async fn scan(
    State(chain): State<Arc<ChainClient>>,
    State(storage): State<Arc<Mutex<DiskClient>>>,
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    info!("scan key: {}, limit: {}", req.key, req.limit);
    let key = req.key.into_bytes();
    let l = req.limit;
    let _ = block_chain::scan(chain, storage, key, l).await;
    (StatusCode::OK, "scan success")
}
