use anchor_lang::prelude::*;

use crate::{constants::MAX_LEVEL, storage::storage_structs::{SkipListMeta, SkipNode}};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = signer, 
        space = SkipListMeta::LEN, 
        seeds=[b"meta"], 
        bump
    )]
    pub meta: Account<'info, SkipListMeta>,

    #[account(
        init, 
        payer = signer, 
        space = SkipNode::LEN, 
        seeds=[b"head"], 
        bump
    )]
    pub head: Account<'info, SkipNode>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
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
