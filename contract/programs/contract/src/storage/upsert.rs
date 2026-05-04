use anchor_lang::prelude::*;
use crate::{
    auth::structs::AuthConfig,
    constants::{
        AUTH_SEEDS, COUNTER_SEEDS, FEE_SEEDS, MAX_KEY_LEN, MAX_KEY_LENGTH, MAX_VALUE_LEN, MAX_VALUE_LENGTH, VALUE_SEEDS
    },
    error::Error,
    fee::FeeConfig,
    storage::structs::{KvCounter, ValueAccount},
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

    // ✅ 新增计数器账号
    #[account(
        mut,
        seeds = [COUNTER_SEEDS], // 和后端的 COUNTER_SEEDS 完全一致
        bump
    )]
    pub counter: Account<'info, KvCounter>,

    pub system_program: Program<'info, System>,

}

pub fn upsert(ctx: Context<Upsert>, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
    msg!("contract upsert key: {:?}, value: {:?}", key, value);
    msg!("contract upsert value pda: {:?}", ctx.accounts.value_account.key());
    require!(key.len() <= MAX_KEY_LENGTH, Error::KeyTooLong);
    require!(value.len() <= MAX_VALUE_LENGTH, Error::ValueTooLong);
    // 权限控制
    require!(!ctx.accounts.auth_config.paused, Error::Paused);

    // 校验长度
    require!(key.len() <= MAX_KEY_LEN, Error::KeyTooLong);
    require!(value.len() <= MAX_VALUE_LEN, Error::ValueTooLarge);

    // 收费逻辑（保留原有）
    let fee_config = &ctx.accounts.fee_config;
    // 基础费用+字节数*一字节的费用
    let fee = fee_config.base_fee + (value.len() as u64 * fee_config.fee_per_byte);
    charge(
        &ctx.accounts.signer,
        &ctx.accounts.treasury.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        fee,
    )?;

    // ==============================================
    // 🔴 关键：判断 key 是否是新增（通过 value_account 的状态）
    // ==============================================
    let is_new = ctx.accounts.value_account.data.is_empty();

    // 直接写入 ValueAccount
    let value_acc = &mut ctx.accounts.value_account;
    value_acc.data = value;

    // ==============================================
    // 🟢 如果是新增，计数器 +1
    // ==============================================
    msg!("is_new: {}", is_new);
    msg!("counter pda: {}", ctx.accounts.counter.key());
    if is_new {
        let counter = &mut ctx.accounts.counter;
        counter.total_count = counter.total_count.saturating_add(1);
        msg!("upsert total_count: {}", counter.total_count);
    }

    Ok(())
}