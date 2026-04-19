use anchor_lang::prelude::*;
use crate::{
    auth::structs::AuthConfig,
    constants::{AUTH_SEEDS, FEE_SEEDS, VALUE_SEEDS},
    error::Error,
    fee::FeeConfig,
    storage::structs::ValueAccount,
    utils::charge,
};

#[derive(Accounts)]
#[instruction(key: Vec<u8>)]
pub struct Delete<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds=[VALUE_SEEDS, key.as_slice()],
        bump
    )]
    pub value_account: Account<'info, ValueAccount>,

    #[account(seeds = [AUTH_SEEDS], bump)]
    pub auth_config: Account<'info, AuthConfig>,

    #[account(seeds = [FEE_SEEDS], bump)]
    pub fee_config: Account<'info, FeeConfig>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn delete(ctx: Context<Delete>, _key: Vec<u8>) -> Result<()> {
    msg!("Value PDA: {:?}", ctx.accounts.value_account.key());
    // 权限控制
    require!(!ctx.accounts.auth_config.paused, Error::Paused);

    // ✅ 只清空当前这个 key 的数据，绝对安全
    let value_acc = &mut ctx.accounts.value_account;
    value_acc.data = vec![];

    // 收费（保留原有）
    let fee = ctx.accounts.fee_config.base_fee;
    charge(
        &ctx.accounts.signer,
        &ctx.accounts.treasury.to_account_info(),
        fee,
    )?;

    // 直接关闭 ValueAccount
    ctx.accounts
        .value_account
        .close(ctx.accounts.signer.to_account_info())?;

    Ok(())
}