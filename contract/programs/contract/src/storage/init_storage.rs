use anchor_lang::prelude::*;

use crate::{constants::{HEAD_SEEDS, MAX_LEVEL, META_SEEDS}, storage::structs::{SkipListMeta, SkipNode}};

#[derive(Accounts)]
pub struct InitStorage<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed, 
        payer = signer, 
        space = SkipListMeta::LEN, 
        seeds=[META_SEEDS], 
        bump
    )]
    pub meta: Account<'info, SkipListMeta>,

    #[account(
        init_if_needed, 
        payer = signer, 
        space = SkipNode::LEN, 
        seeds=[HEAD_SEEDS], 
        bump
    )]
    pub head: Account<'info, SkipNode>,

    pub system_program: Program<'info, System>,
}

pub fn init_storage(ctx: Context<InitStorage>) -> Result<()> {
    // msg!("Greetings from: {:?}", ctx.program_id);
    let meta = &mut ctx.accounts.meta;
    let head = &mut ctx.accounts.head;

    meta.head = head.key();
    meta.max_level = MAX_LEVEL as u8;

    head.key = vec![]; // sentinel
    head.value = Pubkey::default();
    head.level = MAX_LEVEL as u8;
    head.forward = [Pubkey::default(); MAX_LEVEL];

    Ok(())
}
