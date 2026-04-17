use anchor_lang::prelude::*;
use crate::{
    auth::structs::AuthConfig,
    constants::{AUTH_SEEDS, FEE_SEEDS, VALUE_SEEDS},
    error::Error,
    fee::FeeConfig,
    storage::structs::ValueAccount,
    utils::charge,
};

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

pub fn scan(ctx: Context<Scan>, start_key: Vec<u8>, limit: u64) -> Result<()> {
    require!(!ctx.accounts.auth_config.paused, Error::Paused);

    let mut count = 0;
    for acc in ctx.remaining_accounts.iter() {
        if count >= limit { break; }

        // 安全读取
        let data = match acc.try_borrow_data() {
            Ok(d) => d, Err(_) => continue,
        };
        if data.len() < 8 { continue; }

        // 反序列化
        let value_acc = match ValueAccount::deserialize(&mut &data[8..]) {
            Ok(v) => v, Err(_) => continue,
        };

        // 从 PDA 反推 key（最关键！）
        let (key, _) = Pubkey::find_program_address(
            &[VALUE_SEEDS, acc.key().as_ref()], // 用 PDA 反推 key
            ctx.program_id,
        );
        let key = key.to_bytes();

        // 范围过滤
        if key.as_slice() >= start_key.as_slice() {
            emit!(KVEvent {
                key: key.to_vec(),
                value: value_acc.data.clone(),
            });
            count += 1;
        }
    }

    // 收费
    let fee = ctx.accounts.fee_config.base_fee * count as u64;
    charge(&ctx.accounts.signer, &ctx.accounts.treasury, fee)?;

    Ok(())
}