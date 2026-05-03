use anchor_lang::prelude::*;
use crate::{
    auth::structs::AuthConfig,
    constants::{AUTH_SEEDS, SYS_BASE_FEE, SYS_PAUSED, VALUE_SEEDS},
    error::Error,
    storage::structs::ValueAccount,
};

#[derive(Accounts)]
#[instruction(key: Vec<u8>)]
pub struct Get<'info> {
    #[account(seeds = [AUTH_SEEDS], bump)]
    pub auth_config: Account<'info, AuthConfig>,

    #[account(
        seeds=[VALUE_SEEDS, key.as_slice()],
        bump
    )]
    pub value_account: Account<'info, ValueAccount>,
}

pub fn get(ctx: Context<Get>, key: Vec<u8>) -> Result<Vec<u8>> {
    // 1. 系统键访问控制：防止外部用户读取手续费或暂停状态的原始字节
    // 这种直接对比字节数组的方式在合约中是最高效的
    if key == SYS_PAUSED || key == SYS_BASE_FEE {
        msg!("Security Alert: Unauthorized access to system key");
        return Err(Error::UnauthorizedAccess.into()); 
    }
    msg!("contract get key: {:?}", key);
    msg!("contract get value pda: {:?}", ctx.accounts.value_account.key());
    msg!("contract get auth pda: {:?}", ctx.accounts.auth_config.key());
    
    // 权限控制
    require!(!ctx.accounts.auth_config.paused, Error::Paused);

    // 直接返回 ValueAccount 的数据
    Ok(ctx.accounts.value_account.data.clone())
}