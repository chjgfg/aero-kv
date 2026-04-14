use anchor_lang::prelude::*;

#[account]
pub struct FeeConfig {
    pub base_fee: u64,        // 基础操作费
    pub fee_per_byte: u64,    // 每字节收费
    pub scan_fee_per_item: u64, // scan 每条收费
    pub treasury: Pubkey,
}

impl FeeConfig {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 32;
}