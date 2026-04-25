use std::sync::Arc;

// # upsert / get / scan 接口
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;

use crate::{
    block_chain::{self},
    raft::types::AppContext,
};

#[derive(Debug, serde::Deserialize)]
pub struct KVRequest {
    pub key: String,
    pub value: String,
    pub limit: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct PageRequest {
    pub page: usize,
    pub limit: usize,
}

#[derive(Debug, serde::Deserialize)]
pub struct KVQuery {
    pub key: String,
}

// 示例 KV 接口（你可以替换成自己的业务逻辑）
pub async fn init_storage(State(state): State<Arc<AppContext>>) -> impl IntoResponse {
    info!("init storage");
    let chain = state.chain.clone();
    let _ = block_chain::init_storage(chain).await;
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "init storage success")
}

#[allow(dead_code)]
pub async fn upsert(
    // State(chain): State<Arc<ChainClient>>,
    // State(storage): State<Arc<Mutex<DiskClient>>>,
    State(state): State<Arc<AppContext>>,
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    info!("upsert key: {}, value: {}", req.key, req.value);
    let k = req.key.into_bytes();
    let v = req.value.into_bytes();
    let _ = block_chain::upsert(chain, storage, k, v).await;
    // 这里写你的 upsert 业务逻辑
    (StatusCode::OK, "upsert success")
}

#[allow(dead_code)]
pub async fn delete(
    // State(chain): State<Arc<ChainClient>>,
    // State(storage): State<Arc<Mutex<DiskClient>>>,
    State(state): State<Arc<AppContext>>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    info!("delete key: {}", req.key);
    let k = req.key.into_bytes();
    let _ = block_chain::delete(chain, storage, k).await;
    (StatusCode::OK, "delete success")
}

pub async fn gets(
    State(state): State<Arc<AppContext>>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    let chain = state.chain.clone();
    info!("get key: {}", req.key);
    let k = req.key.into_bytes();
    let _ = block_chain::get(chain, k).await;
    (StatusCode::OK, "get success")
}

pub async fn scan(
    // State(chain): State<Arc<ChainClient>>,
    // State(storage): State<Arc<Mutex<DiskClient>>>,
    State(state): State<Arc<AppContext>>,
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    info!("scan key: {}, limit: {}", req.key, req.limit);
    let key = req.key.into_bytes();
    let l = req.limit;
    // let _ = block_chain::scan(chain, storage, key, l).await;
    // (StatusCode::OK, "scan success")
    // 关键：用 match 处理 scan 的 Result，而不是直接 _
    match block_chain::scan(chain, storage, key, l).await {
        Ok(_) => (StatusCode::OK, "scan success".to_string()),
        Err(e) => {
            // 把错误信息返回给前端
            (StatusCode::BAD_REQUEST, format!("scan failed: {}", e))
        }
    }
}

pub async fn page(
    // State(chain): State<Arc<ChainClient>>,
    // State(storage): State<Arc<Mutex<DiskClient>>>,
    State(state): State<Arc<AppContext>>,
    Json(req): Json<PageRequest>,
) -> impl IntoResponse {
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    info!("page offset: {}, limit: {}", req.page, req.limit);
    let o = req.page;
    let l = req.limit;
    let _ = block_chain::page(chain, storage, o, l).await;
    (StatusCode::OK, "page success")
}
