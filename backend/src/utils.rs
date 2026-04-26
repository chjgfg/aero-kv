// # 链交互模块（对应合约 storage）

use crate::{block_chain::types::KVEvent, error::{Error, Result}};

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

// pub fn calc_merkle_root(state: Arc<AppState>) -> Result<[u8; 32]> {
//     let disk = state.storage.lock().unwrap();
//     let keys = disk.get_keys().map_err(|_| Error::InvalidKey)?;
//     let leaves = keys.iter().map(|k| Sha256::hash(k)).collect::<Vec<_>>();
//     let tree = MerkleTree::<Sha256>::from_leaves(&leaves);
//     Ok(tree.root().unwrap_or([0u8; 32]))
// }