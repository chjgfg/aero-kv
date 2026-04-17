use anchor_lang::prelude::*;
use crate::{
    auth::structs::AuthConfig,
    constants::{
        AUTH_SEEDS, FEE_SEEDS, MAX_KEY_LEN, MAX_VALUE_LEN, VALUE_SEEDS,
    },
    error::Error,
    fee::FeeConfig,
    storage::structs::ValueAccount,
    utils::charge,
};

#[derive(Accounts)]
#[instruction(key: Vec<u8>, value: Vec<u8>)]
pub struct Upsert<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        space = ValueAccount::LEN,
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

pub fn upsert(ctx: Context<Upsert>, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
    msg!("contract upsert key: {:?}, value: {:?}", key, value);
    msg!("contract upsert value pda: {:?}", ctx.accounts.value_account.key());
    
    // 权限控制
    require!(!ctx.accounts.auth_config.paused, Error::Paused);

    // 校验长度
    require!(key.len() <= MAX_KEY_LEN, Error::KeyTooLong);
    require!(value.len() <= MAX_VALUE_LEN, Error::ValueTooLarge);

    // 收费逻辑（保留原有）
    let fee_config = &ctx.accounts.fee_config;
    let fee = fee_config.base_fee + (value.len() as u64 * fee_config.fee_per_byte);
    charge(
        &ctx.accounts.signer,
        &ctx.accounts.treasury.to_account_info(),
        fee,
    )?;

    // 直接写入 ValueAccount
    let value_acc = &mut ctx.accounts.value_account;
    value_acc.data = value;

    Ok(())
}