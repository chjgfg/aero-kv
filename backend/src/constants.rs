// # 和合约完全一致：SEEDS、上限值

pub const META_SEEDS: &[&[u8]] = &["meta".as_bytes()];
pub const HEAD_SEEDS: &[&[u8]] = &["head".as_bytes()];

pub const AUTH_SEEDS: &[&[u8]] = &["auth".as_bytes()];
pub const FEE_SEEDS: &[&[u8]] = &["fee".as_bytes()];

pub const NODE_SEEDS: &[u8] = "node".as_bytes();
pub const VALUE_SEEDS: &[u8] = "value".as_bytes();

// 必须和合约一致
pub const MAX_LEVEL: usize = 8;