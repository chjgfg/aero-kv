use anchor_lang::prelude::*;

use crate::{constants::COUNTER_SEEDS, storage::structs::KvCounter};

#[derive(Accounts)]
pub struct InitCounter<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 8,
        seeds = [COUNTER_SEEDS],
        bump
    )]
    pub counter: Account<'info, KvCounter>,
    pub system_program: Program<'info, System>,
}

pub fn init_counter(ctx: Context<InitCounter>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;

    // 关键：只在计数器为 0 时才重置（新账户初始状态是 0）
    if counter.total_count == 0 {
        counter.total_count = 0;
    }

    Ok(())
}
