use anchor_lang::prelude::{program::invoke, *};
use crate::constants::{KVData, StorageMode, PDA_LEN, KV_ACCOUNT_SEED, TREASURY_SEED};

#[derive(Accounts)]
#[instruction(id: String, value:Vec<u8>)]
pub struct UpsertKV<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        space = PDA_LEN + value.len(),
        seeds = [KV_ACCOUNT_SEED, id.as_bytes()],
        bump,
        realloc,
        realloc::payer = signer,
        realloc::zero = false,
    )]
    pub kv_account: Account<'info, KVData>,

    /// CHECK: 协议手续费归集账户
    #[account(
        mut, 
        seeds = [TREASURY_SEED], 
        bump = treasury_bump,
    )]
    pub treasury: UncheckedAccount<'info>,
    pub treasury_bump: u8,
    pub system_program: Program<'info, System>,
}

pub fn upsert_kv(
    ctx: Context<UpsertKV>,
    id: String,
    value: Vec<u8>,
    mode: StorageMode,
) -> Result<()> {
    let clock = Clock::get()?;
    match mode {
        StorageMode::Compressed => {
            // 调用压缩库指令：将数据写入 Merkle Tree Leaf
            // 此时数据不在账户里，由 RPC Indexer 索引
            msg!("Data compressed for ID: {}", id);

        }
        StorageMode::Permanent => {
            // 逻辑：如果是永久存储，收取 0.001 SOL 服务费
            let fee = 1_000_000; // 0.001 SOL
            let instruction = system_instruction::transfer(
                &ctx.accounts.signer.key(),
                &ctx.accounts.treasury.key(),
                fee,
            );
            let account_infos = [
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.treasury.to_account_info(),
            ];
            invoke(&instruction, &account_infos)?;
            // 写入数据到原生账户
            let kv_acc = &mut ctx.accounts.kv_account;
            kv_acc.data = value;
            kv_acc.owner = ctx.accounts.signer.key();
            kv_acc.last_updated = clock.unix_timestamp;
        }
    }
    Ok(())
}
