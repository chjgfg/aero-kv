use super::structs::AuthConfig;
use crate::{constants::AUTH_SEEDS, error::Error};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [AUTH_SEEDS], 
        // ✅ 把权限校验写在账户约束里，彻底避免 require! 宏报错
        // constraint = auth_config.admin == signer.key() @ Error::Unauthorized,
        bump
    )]
    pub auth_config: Account<'info, AuthConfig>,
}

/// 更换管理员
pub fn set_admin(ctx: Context<SetAdmin>, new_admin: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.auth_config;

    require!(
        ctx.accounts.signer.key() == config.admin,
        Error::Unauthorized
    );

    config.admin = new_admin;
    Ok(())
}
