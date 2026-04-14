use super::structs::FeeConfig;
use crate::auth::structs::AuthConfig;
use crate::constants::{AUTH_SEEDS, FEE_SEEDS};
use crate::error::Error;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetFee<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(seeds = [AUTH_SEEDS], bump)]
    pub auth_config: Account<'info, AuthConfig>,

    #[account(seeds = [FEE_SEEDS], bump)]
    pub fee_config: Account<'info, FeeConfig>,
}

pub fn set_fee(
    ctx: Context<SetFee>,
    base_fee: u64,
    fee_per_byte: u64,
    scan_fee_per_item: u64,
) -> Result<()> {
    let auth = &ctx.accounts.auth_config;

    require!(ctx.accounts.signer.key() == auth.admin, Error::Unauthorized);

    let fee = &mut ctx.accounts.fee_config;

    fee.base_fee = base_fee;
    fee.fee_per_byte = fee_per_byte;
    fee.scan_fee_per_item = scan_fee_per_item;

    Ok(())
}
