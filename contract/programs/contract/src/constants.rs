use anchor_lang::prelude::*;

// pub const KV_ACCOUNT_SEED: &[u8] = "aero_kv".as_bytes();

// pub const TREASURY_SEED: &[u8] = "treasury".as_bytes();

// pub const PDA_LEN: usize = 8 + 32 + 8 + 4;

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
// pub enum StorageMode {
//     /// 免费版：走 ZK 压缩（这里简化展示，实际需接入压缩库）
//     Compressed,
//     /// 付费版：走原生账户存储
//     Permanent,
// }

// #[account]
// pub struct KVData {
//     pub owner: Pubkey,
//     pub last_updated: i64,
//     pub data: Vec<u8>,
// }

pub const MAX_LEVEL: usize = 8;
pub const MAX_KEY_LEN: usize = 64;
pub const MAX_VALUE_LEN: usize = 1024;