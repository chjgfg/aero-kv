// # 结构体：KV 数据映射

use anchor_client::anchor_lang::prelude::*;

// 必须加上这个宏，它会自动帮你实现 try_deserialize 等方法
#[derive(AnchorDeserialize)]
pub struct ValueAccount {
    pub data: Vec<u8>, // <= MAX_VALUE_LEN
}
