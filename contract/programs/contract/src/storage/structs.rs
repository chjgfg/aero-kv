use anchor_lang::prelude::*;

use crate::constants::{MAX_VALUE_LEN};

// #[account]
// pub struct SkipListMeta {
//     pub head: Pubkey,
//     pub max_level: u8,
// }

// impl SkipListMeta {
//     pub const LEN: usize = 8 + 32 + 1;
// }

// #[account]
// pub struct SkipNode {
//     pub key: Vec<u8>,                 // <= MAX_KEY_LEN
//     pub value: Pubkey,                // ValueAccount
//     pub level: u8,                    // 1..=MAX_LEVEL
//     pub forward: [Pubkey; MAX_LEVEL], // 固定长度
// }

// impl SkipNode {
//     pub const LEN: usize = 8 + // discriminator
//         4 + MAX_KEY_LEN + // Vec<u8> (len + data)
//         32 + // value
//         1 + // level
//         32 * MAX_LEVEL; // forward
// }

#[account]
pub struct ValueAccount {
    pub data: Vec<u8>, // <= MAX_VALUE_LEN
}

impl ValueAccount {
    pub const LEN: usize = 8 + 4 + MAX_VALUE_LEN;
}
