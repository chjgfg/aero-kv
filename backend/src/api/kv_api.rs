use std::sync::{Arc};

// # upsert / get / scan 接口
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;
use rs_merkle::{Hasher as _, algorithms::Sha256};

use crate::{
    AppState,
    block_chain::{self}, utils,
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
    let _ = block_chain::init_storage(chain).await;
    // 创建第一棵树
    let _ = utils::calc_merkle_root(state);
    (StatusCode::OK, "init storage success")
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
    let _ = block_chain::upsert(chain, storage, k, v).await;
    //  重建树
    let _ = utils::calc_merkle_root(state);
    (StatusCode::OK, "upsert success")
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    info!("delete key: {}", req.key);
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    let k = req.key.into_bytes();
    let _ = block_chain::delete(chain, storage, k).await;
    // 重建树
    let _ = utils::calc_merkle_root(state);
    (StatusCode::OK, "delete success")
}

pub async fn gets(
    State(state): State<Arc<AppState>>,
    Query(req): Query<KVQuery>,
) -> impl IntoResponse {
    info!("get key: {}", req.key);
    let chain = state.chain.clone();
    let k = req.key.into_bytes();
    // let _ = block_chain::get(chain, k).await;
    let value = match block_chain::get(chain, k).await {
        Ok(v) => v,
        Ok(_) => return (StatusCode::NOT_FOUND, "key not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "storage error").into_response(),
    };

    // 3. 获取全局的叶子哈希列表（用来找索引）
    let leaf_hashes = state.leaf_hashes.lock().await;

    // 4. 计算当前 key 的哈希
    let key_hash = Sha256::hash(&k);

    // 5. 查找 key 在 Merkle 树中的位置（索引）
    let Some(index) = leaf_hashes.iter().position(|hash| hash == &key_hash) else {
        return (StatusCode::NOT_FOUND, "key not in merkle tree");
    };

    // 6. 获取全局 Merkle 树
    let tree = state.merkle_tree.lock().await;

    // 7. 生成证明（关键：传入索引）
    let proof = tree.proof(&[index]);

    // 8. 序列化证明（返回给前端/用户验证）
    let proof_bytes = match bincode::serialize(&proof) {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "proof serialize error"),
    };

    // 9. 返回 value + 证明 + root
    let json = Json(serde_json::json!({
        "key": req.key,
        "value": hex::encode(value),
        "proof": hex::encode(proof_bytes),
        "merkle_root": hex::encode(tree.root().unwrap_or_default()),
    }));
    (StatusCode::OK, json.as_str().unwrap())
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
    // match block_chain::scan(chain, storage, key, l).await {
    //     Ok(_) => (StatusCode::OK, "scan success".to_string()),
    //     Err(e) => {
    //         // 把错误信息返回给前端
    //         (StatusCode::BAD_REQUEST, format!("scan failed: {}", e))
    //     }
    // }
    // 1. 先拿到 scan 的结果（保持你原来的调用方式）
    let scan_result = match block_chain::scan(chain, storage.clone(), key, l).await {
        Ok(keys) => keys, // 假设这里返回 Vec<Vec<u8>>
        Err(e) => return (StatusCode::BAD_REQUEST, format!("scan failed: {}", e)).into_response(),
    };
    // ===================== 新增 Merkle 证明部分 =====================
    // 2. 读取全局叶子哈希列表
    let leaf_hashes = state.leaf_hashes.lock().await;

    // 3. 批量找索引
    let mut indices = Vec::new();
    for key in &scan_result {
        let key_hash = rs_merkle::algorithms::Sha256::hash(key);
        if let Some(idx) = leaf_hashes.iter().position(|h| h == &key_hash) {
            indices.push(idx);
        }
    }

    // 4. 读取全局 Merkle 树，生成批量证明
    let tree = state.merkle_tree.lock().await;
    let proof = tree.proof(&indices);
    let proof_bytes = bincode::serialize(&proof).unwrap_or_default();
    let root = tree.root().unwrap_or_default();

    // ===================== 返回带证明的结果 =====================
    Json(serde_json::json!({
        "keys": scan_result.iter().map(|k| hex::encode(k)).collect::<Vec<_>>(),
        "merkle_root": hex::encode(root),
        "proof": hex::encode(proof_bytes),
    })).into_response()
}

pub async fn page(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PageRequest>,
) -> impl IntoResponse {
    info!("page offset: {}, limit: {}", req.page, req.limit);
    let chain = state.chain.clone();
    let storage = state.storage.clone();
    let o = req.page;
    let l = req.limit;
    // let _ = block_chain::page(chain, storage, o, l).await;
    // 1. 拿到分页结果（保持你原来的调用方式）
    let page_result = match block_chain::page(chain, storage, o, l).await {
        Ok(keys) => keys, // 假设这里返回 Vec<Vec<u8>>
        Err(e) => return (StatusCode::BAD_REQUEST, format!("page failed: {}", e)).into_response(),
    };

    // ===================== 新增 Merkle 证明部分 =====================
    let leaf_hashes = state.leaf_hashes.lock().await;
    let mut indices = Vec::new();
    for key in &page_result {
        let key_hash = rs_merkle::algorithms::Sha256::hash(key);
        if let Some(idx) = leaf_hashes.iter().position(|h| h == &key_hash) {
            indices.push(idx);
        }
    }

    let tree = state.merkle_tree.lock().await;
    let proof = tree.proof(&indices);
    let proof_bytes = bincode::serialize(&proof).unwrap_or_default();
    let root = tree.root().unwrap_or_default();

    // ===================== 返回带证明的结果 =====================
    Json(serde_json::json!({
        "page": page,
        "limit": limit,
        "keys": page_result.iter().map(|k| hex::encode(k)).collect::<Vec<_>>(),
        "merkle_root": hex::encode(root),
        "proof": hex::encode(proof_bytes),
    }))
    .into_response()
    // (StatusCode::OK, "page success")
}
