use super::structs::AuthConfig;
use crate::error::Error;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(mut)]
    pub auth_config: Account<'info, AuthConfig>,

    pub signer: Signer<'info>,
}

/// 暂停 开启合约
pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    let config = &mut ctx.accounts.auth_config;

    require!(
        ctx.accounts.signer.key() == config.admin,
        Error::Unauthorized
    );

    config.paused = paused;

    Ok(())
}
