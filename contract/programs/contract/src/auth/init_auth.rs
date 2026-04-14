use anchor_lang::prelude::*;

use crate::{auth::structs::AuthConfig, constants::AUTH_SEEDS};

#[derive(Accounts)]
pub struct InitAuth<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        space = AuthConfig::LEN,
        seeds = [AUTH_SEEDS], 
        bump
    )]
    pub auth_config: Account<'info, AuthConfig>,

    pub system_program: Program<'info, System>,
}

pub fn init_auth(ctx: Context<InitAuth>) -> Result<()> {
    let auth = &mut ctx.accounts.auth_config;
    auth.admin = ctx.accounts.signer.key(); // 第一次赋值
    auth.paused = false; // 初始不暂停
    Ok(())
}
