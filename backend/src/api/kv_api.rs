use std::sync::Arc;

// # upsert / get / scan 接口
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;
use rs_merkle::{Hasher as _, algorithms::Sha256};
use serde_json::json;

use crate::{
    AppState,
    block_chain::{self},
    utils,
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
pub async fn init_storage(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("init storage");
    let chain = state.chain.clone();
    let Ok(res) = block_chain::init_storage(chain).await else {
        let json_response = serde_json::json!({
            "status": "error",
            "signature": "init storage error",
        });
        return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
    };
    // 创建第一棵树
    let _ = utils::calc_merkle_root(state).await;
    let json_response = serde_json::json!({
        "status": "success",
        "signature": res.to_string()
    });
    (StatusCode::OK, Json(json_response)).into_response()
}

// 新增一个接口，专门用来初始化计数器
pub async fn init_counter(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("init counter");
    let chain = state.chain.clone();

    match block_chain::init_counter(chain).await {
        Ok(sig) => {
            return (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "signature": sig.to_string()
                })),
            )
                .into_response();
        }
        Err(e) => {
            log::error!("init counter failed: {:?}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({ "status": "error", "message": format!("init counter failed: {}", e) }),
                ),
            )
                .into_response();
        }
    }
}

pub async fn upsert(
    State(state): State<Arc<AppState>>,
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    info!("upsert key: {}, value: {}", req.key, req.value);
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    let k = req.key.into_bytes();
    let v = req.value.into_bytes();
    let Ok(res) = block_chain::upsert(chain, storage, k, v).await else {
        let json_response = serde_json::json!({
            "status": "error",
            "signature": "upsert error",
        });
        return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
    };
    //  重建树
    let _ = utils::calc_merkle_root(state).await;
    let json_response = serde_json::json!({
        "status": "success",
        "signature": res.to_string()
    });
    (StatusCode::OK, Json(json_response)).into_response()
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    info!("delete key: {}", req.key);
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    let k = req.key.into_bytes();
    let Ok(res) = block_chain::delete(chain, storage, k).await else {
        let json_response = serde_json::json!({
            "status": "error",
            "signature": "delete error",
        });
        return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
    };
    // 重建树
    let _ = utils::calc_merkle_root(state).await;
    let json_response = serde_json::json!({
        "status": "success",
        "signature": res.to_string()
    });
    (StatusCode::OK, Json(json_response)).into_response()
}

pub async fn gets(
    State(state): State<Arc<AppState>>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    info!("get key: {}", req.key);
    let chain = state.chain.clone();
    let k = req.key.clone().into_bytes();
    // let _ = block_chain::get(chain, k).await;
    let value = match block_chain::get(chain, k.clone()).await {
        Ok(v) => v,
        // Ok(None) => return (StatusCode::NOT_FOUND, "key not found").into_response(),
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "signature": e.to_string()
            });
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json_response)).into_response();
        }
    };

    // 3. 获取全局的叶子哈希列表（用来找索引）
    let leaf_hashes = state.leaf_hashes.lock().await;

    // 4. 计算当前 key 的哈希
    let key_hash = Sha256::hash(&k);

    // 5. 查找 key 在 Merkle 树中的位置（索引）
    // 3. 找不到索引
    let Some(index) = leaf_hashes.iter().position(|h| h == &key_hash) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": "error",
                "msg": "key not in merkle tree"
            })),
        )
            .into_response();
    };

    // 6. 获取全局 Merkle 树
    // 5. 获取树和根
    let tree = state.merkle_tree.lock().await;
    let proof = tree.proof(&[index].to_vec()); // 生成 Merkle 证明路径
    let proof_bytes: Vec<String> = proof
        .proof_hashes()
        .iter()
        .map(|h| hex::encode(h))
        .collect();
    let root = tree.root().unwrap_or_default();

    // 6. 不序列化 proof，直接返回验证所需的信息
    let json_response = serde_json::json!({
        "status": "success",
        "key": req.key,
        "value": value,
        "merkle_root": hex::encode(root),
        "key_hash": hex::encode(key_hash),
        "leaf_index": index,
        "proof": proof_bytes,
    });
    (StatusCode::OK, Json(json_response)).into_response()
}

pub async fn scan(
    State(state): State<Arc<AppState>>,
    Json(req): Json<KVRequest>,
) -> impl IntoResponse {
    info!("scan key: {}, limit: {}", req.key, req.limit);
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    let key = req.key.into_bytes();
    let l = req.limit;
    // 关键：用 match 处理 scan 的 Result，而不是直接 _
    // 1. 先拿到 scan 的结果（保持你原来的调用方式）
    let scan_result = match block_chain::scan(chain, storage.clone(), key, l).await {
        Ok(keys) => keys, // 假设这里返回 Vec<Vec<u8>>
        // Err(e) => return (StatusCode::BAD_REQUEST, format!("scan failed: {}", e)).into_response(),
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "signature": e.to_string()
            });
            return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
        }
    };
    // ===================== 新增 Merkle 证明部分 =====================
    // 2. 读取全局叶子哈希列表
    let leaf_hashes = state.leaf_hashes.lock().await;

    // 3. 批量找索引
    // 3. 批量找索引和哈希
    let mut indices = Vec::new();
    let mut key_hashes = Vec::new();
    for entry in &scan_result {
        let key = entry.0.clone();
        let key_hash = Sha256::hash(&key.as_bytes().to_vec());
        if let Some(idx) = leaf_hashes.iter().position(|h| h == &key_hash) {
            indices.push(idx);
            key_hashes.push(key_hash);
        }
    }

    // 4. 读取全局 Merkle 树，生成批量证明
    let tree = state.merkle_tree.lock().await;
    let proof = tree.proof(&indices); // 生成 Merkle 证明路径
    let proof_bytes: Vec<String> = proof
        .proof_hashes()
        .iter()
        .map(|h| hex::encode(h))
        .collect();
    let root = tree.root().unwrap_or_default();

    // 5. 返回批量验证所需的信息
    let json_response = serde_json::json!({
        "status": "success",
        "pairs": scan_result.iter().map(|(k, v)| serde_json::json!({
            "key": k,
            "value": v
        })).collect::<Vec<_>>(),
        "merkle_root": hex::encode(root),
        "key_hashes": key_hashes.iter().map(|h| hex::encode(h)).collect::<Vec<_>>(),
        "leaf_indices": indices,
        "proof": proof_bytes,
    });
    (StatusCode::OK, Json(json_response)).into_response()
}

pub async fn page(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PageRequest>,
) -> impl IntoResponse {
    info!("page offset: {}, limit: {}", req.page, req.limit);
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    let page = req.page;
    let limit: usize = req.limit;
    // let _ = block_chain::page(chain, storage, o, l).await;
    // 1. 拿到分页结果（保持你原来的调用方式）
    let result = match block_chain::page(chain, storage, page, limit).await {
        Ok(keys) => keys, // 假设这里返回 Vec<Vec<u8>>
        // Err(e) => return (StatusCode::BAD_REQUEST, format!("page failed: {}", e)).into_response(),
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "signature": e.to_string()
            });
            return (StatusCode::BAD_REQUEST, Json(json_response)).into_response();
        }
    };
    let page_result = result.0;
    let counter = result.1;

    // ===================== 新增 Merkle 证明部分 =====================
    let leaf_hashes = state.leaf_hashes.lock().await;

    // 3. 批量找索引和 key 哈希
    let mut indices = Vec::new();
    let mut key_hashes = Vec::new();
    for entry in &page_result {
        let key = &entry.0;
        let key_hash = rs_merkle::algorithms::Sha256::hash(&key.as_bytes().to_vec());
        if let Some(idx) = leaf_hashes.iter().position(|h| h == &key_hash) {
            indices.push(idx);
            key_hashes.push(key_hash);
        }
    }

    // 4. 获取全局 Merkle 树和根
    let tree = state.merkle_tree.lock().await;
    let proof = tree.proof(&indices);
    let proof_bytes: Vec<String> = proof
        .proof_hashes()
        .iter()
        .map(|h| hex::encode(h))
        .collect();
    let root = tree.root().unwrap_or_default();

    // ===================== 返回带证明的结果 =====================
    let json_response = serde_json::json!({
        "status": "success",
        "page": page,
        "limit": limit,
        "total": counter, // 🔥 这里返回链上真实总数！
        "pairs": page_result.iter().map(|(k, v)| serde_json::json!({
            "key": k,
            "value": v
        })).collect::<Vec<_>>(),
        "merkle_root": hex::encode(root),
        "key_hashes": key_hashes.iter().map(|h| hex::encode(h)).collect::<Vec<_>>(),
        "leaf_indices": indices,
        "proof": proof_bytes,
    });
    (StatusCode::OK, Json(json_response)).into_response()
}
