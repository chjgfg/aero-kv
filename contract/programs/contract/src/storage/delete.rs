use anchor_lang::prelude::*;

use crate::{constants::{KVData, SEED}, error::Error};


#[derive(Accounts)]
#[instruction(id:String, )]
pub struct DeleteKV<'info>{

    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED, id.as_bytes()],
        bump,  
        close = signer,

    )]
    pub kv_account:Account<'info, KVData>,
}

pub fn delete_kv(ctx: Context<DeleteKV>, _id: String)->Result<()>{
    // 权限校验：只有 Owner 才能删除
    require_keys_eq!(ctx.accounts.kv_account.owner, ctx.accounts.signer.key(), Error::Unauthorized);
    // 关闭账户并回收租金给 Signer
    Ok(())
}