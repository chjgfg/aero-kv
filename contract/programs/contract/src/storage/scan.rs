use anchor_lang::prelude::*;
use crate::{
    auth::structs::AuthConfig,
    constants::{AUTH_SEEDS, FEE_SEEDS},
    error::Error,
    fee::FeeConfig,
    storage::structs::ValueAccount,
    utils::charge,
};

#[derive(Debug)]
#[event]
pub struct KVEvent {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Accounts)]
pub struct Scan<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(seeds = [AUTH_SEEDS], bump)]
    pub auth_config: Account<'info, AuthConfig>,
    #[account(seeds = [FEE_SEEDS], bump)]
    pub fee_config: Account<'info, FeeConfig>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}


pub fn scan(ctx: Context<Scan>, start_key: Vec<u8>, limit: u64, keys: Vec<Vec<u8>>) -> Result<()> {
    require!(!ctx.accounts.auth_config.paused, Error::Paused);

    let mut count = 0;
    // 用 zip 把 remaining_accounts 和 keys 一一对应
    for (acc, key) in ctx.remaining_accounts.iter().zip(keys) {
        if count >= limit { break; }

        // 1. 读取账户数据
        let data = match acc.try_borrow_data() {
            Ok(d) => d, Err(_) => continue,
        };
        if data.len() < 8 { continue; }

        // 2. 反序列化 ValueAccount
        let value_acc = match ValueAccount::deserialize(&mut &data[8..]) {
            Ok(v) => v, Err(_) => continue,
        };

        // 3. 范围过滤（key 直接从指令数据里拿，不用反推）
        if key.as_slice() >= start_key.as_slice() {
            let kv = KVEvent {
                key,
                value: value_acc.data.clone(),
            };
            msg!("kv: {:?}", kv);
            msg!("before emit");
            emit!(kv);
            msg!("after emit");
            count += 1;
        }
    }

    // 收费逻辑不变
    let fee = ctx.accounts.fee_config.base_fee * count as u64;
    charge(&ctx.accounts.signer, &ctx.accounts.treasury, fee)?;

    Ok(())
}