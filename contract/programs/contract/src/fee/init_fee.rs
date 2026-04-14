use anchor_lang::prelude::*;

use crate::{constants::FEE_SEEDS, fee::FeeConfig};

#[derive(Accounts)]
pub struct InitFee<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    // ✅ 初始化 PDA 账户
    #[account(
        init,
        payer = signer,
        space = 8 + 8 + 8 + 8 + 32,
        seeds = [FEE_SEEDS],
        bump
    )]
    pub fee_config: Account<'info, FeeConfig>,

    pub system_program: Program<'info, System>,
}

pub fn init_fee(ctx: Context<InitFee>) -> Result<()> {
    let fee = &mut ctx.accounts.fee_config;
    fee.base_fee = 0;
    fee.fee_per_byte = 0;
    fee.scan_fee_per_item = 0;
    // ✅ 初始国库 = 部署者
    fee.treasury = ctx.accounts.signer.key();
    Ok(())
}
