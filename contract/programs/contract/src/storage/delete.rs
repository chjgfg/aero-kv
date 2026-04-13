use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_LEVEL, META_SEEDS, NODE_SEEDS, VALUE_SEEDS},
    error::Error,
    storage::storage_structs::{SkipListMeta, SkipNode, ValueAccount},
};

#[derive(Accounts)]
#[instruction(key: Vec<u8>)]
pub struct Delete<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(seeds=[META_SEEDS], bump)]
    pub meta: Account<'info, SkipListMeta>,

    #[account(
        mut,
        seeds=[NODE_SEEDS, key.as_slice()],
        bump
    )]
    pub target: Account<'info, SkipNode>,

    #[account(
        mut,
        seeds=[VALUE_SEEDS, key.as_slice()],
        bump
    )]
    pub value_account: Account<'info, ValueAccount>,

    pub system_program: Program<'info, System>,
}

pub fn delete(ctx: Context<Delete>, key: Vec<u8>) -> Result<()> {
    let target = &ctx.accounts.target;

    require!(target.key == key, Error::NotFound);

    if ctx.remaining_accounts.len() != MAX_LEVEL {
        return err!(Error::InvalidRemaining);
    }

    for i in 0..(target.level as usize) {
        // 1. 获取 AccountInfo 的引用
        let prev_info = &ctx.remaining_accounts[i];

        // 2. 手动反序列化数据
        // 这样不会产生复杂的生命周期绑定，直接从 data 借用
        let mut prev_data = prev_info.try_borrow_mut_data()?;

        // 假设你的 SkipNode 结构体已经 derive 了 AnchorSerialize/Deserialize
        // 跳过 Anchor 的 8 字节 discriminator
        let mut prev = SkipNode::deserialize(&mut &prev_data[8..])?;

        // if prev.forward[i] == target.key() {
        //     prev.forward[i] = target.forward[i];
        //     prev.exit(ctx.program_id)?;
        // }
        // 3. 执行逻辑
        if prev.forward[i] == target.key() {
            prev.forward[i] = target.forward[i];

            // 4. 手动写回数据 (这一步非常重要，替代了 exit)
            let mut writer = &mut prev_data[8..];
            prev.serialize(&mut writer)?;
        }
    }

    // ✅ 正确关闭
    ctx.accounts
        .value_account
        .close(ctx.accounts.signer.to_account_info())?;
    ctx.accounts
        .target
        .close(ctx.accounts.signer.to_account_info())?;

    Ok(())
}
