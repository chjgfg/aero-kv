use super::structs::AuthConfig;
use crate::{constants::AUTH_SEEDS, error::Error};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [AUTH_SEEDS],
        constraint = auth_config.admin == signer.key() @ Error::Unauthorized,
        bump
    )]
    pub auth_config: Account<'info, AuthConfig>,
}

/// 暂停或开启合约
pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    let config = &mut ctx.accounts.auth_config;

    require!(
        ctx.accounts.signer.key() == config.admin,
        Error::Unauthorized
    );

    config.paused = paused;

    Ok(())
}
