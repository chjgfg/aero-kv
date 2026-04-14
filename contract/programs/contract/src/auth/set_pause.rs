use super::structs::AuthConfig;
use crate::error::Error;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(mut)]
    pub config: Account<'info, AuthConfig>,

    pub signer: Signer<'info>,
}

pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    let config = &mut ctx.accounts.config;

    require!(
        ctx.accounts.signer.key() == config.admin,
        Error::Unauthorized
    );

    config.paused = paused;

    Ok(())
}
