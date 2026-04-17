// storage/init_storage.rs
use anchor_lang::prelude::*;
use crate::auth::structs::AuthConfig;
use crate::fee::FeeConfig;
use crate::constants::{AUTH_SEEDS, FEE_SEEDS, META_SEEDS, HEAD_SEEDS};

#[derive(Accounts)]
pub struct InitStorage<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // 👇 简化：meta 只保留 8 字节 discriminator + 1 字节（空数据），不占多余空间
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 1,
        seeds=[META_SEEDS],
        bump
    )]
    pub meta: Account<'info, EmptyMeta>,

    // 👇 head 同理，只保留 8 + 1 字节
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 1,
        seeds=[HEAD_SEEDS],
        bump
    )]
    pub head: Account<'info, EmptyMeta>,

    // 👇 auth_config 必须用你实际的 AuthConfig::LEN，这里假设是 8 + 1（根据你自己的结构体改）
    #[account(
        init_if_needed,
        payer = signer,
        space = AuthConfig::LEN,
        seeds=[AUTH_SEEDS],
        bump
    )]
    pub auth_config: Account<'info, AuthConfig>,

    // 👇 fee_config 同理，用 FeeConfig::LEN
    #[account(
        init_if_needed,
        payer = signer,
        space = FeeConfig::LEN,
        seeds=[FEE_SEEDS],
        bump
    )]
    pub fee_config: Account<'info, FeeConfig>,

    pub system_program: Program<'info, System>,
}

// 👇 空结构体，只占 1 字节，避免空间不匹配
#[account]
pub struct EmptyMeta {
    pub _dummy: u8,
}

pub fn init_storage(ctx: Context<InitStorage>) -> Result<()> {
    // 初始化空账户，不做任何复杂操作
    ctx.accounts.meta._dummy = 0;
    ctx.accounts.head._dummy = 0;
    // 👇 初始化 auth_config（如果你的结构体需要默认值）
    ctx.accounts.auth_config.paused = false;
    // 👇 初始化 fee_config（根据你的业务设置默认值）
    ctx.accounts.fee_config.base_fee = 1000;
    ctx.accounts.fee_config.fee_per_byte = 10;
    Ok(())
}