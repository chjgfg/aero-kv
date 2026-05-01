// # 链交互模块（对应合约 storage）

use std::sync::Arc;

use rs_merkle::{Hasher, MerkleTree, algorithms::Sha256};
use tokio::sync::Mutex;

use crate::{
    AppState,
    block_chain::types::KVEvent,
    error::{Error, Result},
    storage::engine::DiskClient,
};

pub fn parse_keypair_array(s: &str) -> Result<Vec<u8>> {
    // 去掉前后的 []，按逗号分割，解析成 u8
    let s = s
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or(Error::InvalidKey)?;
    s.split(',')
        .map(|part| part.trim().parse::<u8>().map_err(|_| Error::InvalidKey))
        .collect()
}

pub fn bytes_to_str(event: &KVEvent) -> Result<(String, String)> {
    let event = event.clone();
    let key = String::from_utf8(event.key).unwrap();
    let value = String::from_utf8(event.value).unwrap();
    Ok((key, value))
}

pub async fn calc_merkle_root(state: Arc<AppState>) -> Result<[u8; 32]> {
    let disk = state.storage.lock().await;
    // 1. 拿所有 key
    let keys = disk.get_keys().map_err(|_| Error::InvalidKey)?;
    // 2. 创建叶子（对每个 key 做哈希）
    let leaves = keys.iter().map(|k| Sha256::hash(k)).collect::<Vec<_>>();
    // 3. ✅ 创建整棵 Merkle 树
    let tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    let root = tree.root().unwrap_or([0u8; 32]);
    // 4. 更新全局状态（必须先 lock()）
    *state.merkle_tree.lock().await = tree;
    *state.leaf_hashes.lock().await = leaves;
    Ok(root)
}

pub async fn rebuild_merkle_tree(
    disk: &Arc<Mutex<DiskClient>>,
    merkle_tree_lock: &Arc<Mutex<MerkleTree<Sha256>>>,
    leaf_hashes_lock: &Arc<Mutex<Vec<[u8; 32]>>>,
) -> Result<[u8; 32]> {
    let disk_guard = disk.lock().await;

    // 1. 获取所有 key 并进行排序 (排序对于多节点达成一致根哈希至关重要)
    let mut keys = disk_guard.get_keys().map_err(|_| Error::InvalidKey)?;
    keys.sort(); // 必须排序！确保所有节点生成的树结构一模一样

    // 2. 创建叶子
    let leaves = keys.iter().map(|k| Sha256::hash(k)).collect::<Vec<_>>();

    // 3. 创建 Merkle 树
    let tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    let root = tree.root().unwrap_or([0u8; 32]);

    // 4. 更新全局状态
    *merkle_tree_lock.lock().await = tree;
    *leaf_hashes_lock.lock().await = leaves;

    Ok(root)
}

// pub async fn ensure_leader_and_fresh(
//     state: &Arc<AppState>,
// ) -> std::result::Result<(), (StatusCode, Json<serde_json::Value>)> {
//     // 方案二：通过 Raft 心跳确认自己仍是有效 Leader 且数据已同步
//     if let Err(e) = state.raft.ensure_linearizable().await {
//         let json_response = serde_json::json!({
//             "status": "error",
//             "message": "Consistent read failed (Node may not be Leader)",
//             "detail": e.to_string()
//         });
//         return Err((StatusCode::SERVICE_UNAVAILABLE, Json(json_response)));
//     }
//     Ok(())
// }
