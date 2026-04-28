// # 结构体：KV 数据映射

use anchor_client::anchor_lang::prelude::{borsh::{BorshDeserialize, BorshSerialize}, *};

// 必须加上这个宏，它会自动帮你实现 try_deserialize 等方法
#[derive(AnchorDeserialize)]
pub struct ValueAccount {
    pub data: Vec<u8>, // <= MAX_VALUE_LEN
}

// 后端里的定义（必须和上面完全一样）
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct KVEvent {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct KvCounter {
    pub total_count: u64, // 真实总条数
}
