// # 和合约完全一致：SEEDS、上限值

pub const META_SEEDS: &[&[u8]] = &["meta".as_bytes()];
pub const HEAD_SEEDS: &[&[u8]] = &["head".as_bytes()];

pub const AUTH_SEEDS: &[&[u8]] = &["auth".as_bytes()];
pub const FEE_SEEDS: &[&[u8]] = &["fee".as_bytes()];

// pub const NODE_SEEDS: &[u8] = "node".as_bytes();
pub const VALUE_SEEDS: &[u8] = "value".as_bytes();

// 后端代码里，和 VALUE_SEEDS 同级定义
pub const COUNTER_SEEDS: &[&[u8]] = &["counter".as_bytes()];

pub const SYS_PAUSED: &[u8] = "sys_paused".as_bytes();
pub const SYS_BASE_FEE: &[u8] = "sys_base_fee".as_bytes();

// 必须和合约一致
pub const MAX_KEY_LENGTH: usize = 256;    // Key 最大长度
pub const MAX_VALUE_LENGTH: usize = 2048; // Value 最大长度