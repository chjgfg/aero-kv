use crate::{
    auth::structs::AuthConfig,
    constants::{AUTH_SEEDS, COUNTER_SEEDS, FEE_SEEDS},
    error::Error,
    fee::FeeConfig,
    storage::structs::{KVEvent, KvCounter, ValueAccount},
    utils::charge,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Page<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
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

pub fn page(ctx: Context<Page>, keys: Vec<Vec<u8>>) -> Result<()> {
    require!(!ctx.accounts.auth_config.paused, Error::Paused);

    let mut count = 0;
    // 用 zip 把 remaining_accounts 和 keys 一一对应
    for (acc, key) in ctx.remaining_accounts.iter().zip(keys) {
        // 1. 读取账户数据
        let data = match acc.try_borrow_data() {
            Ok(d) => d,
            Err(_) => continue,
        };
        if data.len() < 8 {
            continue;
        }

        // 2. 反序列化 ValueAccount
        let value_acc = match ValueAccount::deserialize(&mut &data[8..]) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // 3. 范围过滤（key 直接从指令数据里拿，不用反推）

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

    // 收费逻辑不变
    let fee = ctx.accounts.fee_config.base_fee * count as u64;
    charge(&ctx.accounts.signer, &ctx.accounts.treasury, &ctx.accounts.system_program.to_account_info(), fee)?;

    // 1. 获取 counter 账户（假设你已经在 Context 里引入了它）
    let counter = &ctx.accounts.counter;
    // 2. 打印一个特定格式的日志，方便后端抓取
    msg!("FINAL_TOTAL: {}", counter.total_count);

    Ok(())
}
