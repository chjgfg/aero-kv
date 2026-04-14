use super::structs::AuthConfig;
use crate::{constants::AUTH_SEEDS, error::Error};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(seeds = [AUTH_SEEDS], bump)]
    pub auth_config: Account<'info, AuthConfig>,
}

pub fn set_admin(ctx: Context<SetAdmin>, new_admin: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.auth_config;

    require!(
        ctx.accounts.signer.key() == config.admin,
        Error::Unauthorized
    );

    config.admin = new_admin;
    Ok(())
}
