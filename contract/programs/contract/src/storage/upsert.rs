use crate::{
    auth::structs::AuthConfig,
    constants::{
        AUTH_SEEDS, FEE_SEEDS, MAX_KEY_LEN, MAX_LEVEL, MAX_VALUE_LEN, META_SEEDS, NODE_SEEDS,
        VALUE_SEEDS,
    },
    error::Error,
    fee::FeeConfig,
    storage::structs::{SkipListMeta, SkipNode, ValueAccount},
    utils::{calc_level, charge},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(key: Vec<u8>)]
pub struct Upsert<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[META_SEEDS], bump)]
    pub meta: Account<'info, SkipListMeta>,

    #[account(
        init_if_needed,
        payer = signer,
        space = SkipNode::LEN,
        seeds=[NODE_SEEDS, key.as_slice()],
        bump
    )]
    pub new_node: Account<'info, SkipNode>,

    #[account(
        init_if_needed,
        payer = signer,
        space = ValueAccount::LEN,
        seeds=[VALUE_SEEDS, key.as_slice()],
        bump
    )]
    pub value_account: Account<'info, ValueAccount>,

    // ✅ 正确：带 seeds 的 PDA，全局唯一
    #[account(seeds = [AUTH_SEEDS], bump)]
    pub auth_config: Account<'info, AuthConfig>,

    // ✅ 正确：带 seeds 的 PDA，全局唯一
    #[account(seeds = [FEE_SEEDS], bump)]
    pub fee_config: Account<'info, FeeConfig>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// upsert:
/// 需要通过 remaining_accounts 传入“沿途会被更新 forward 的节点账户”
/// 顺序：从高层到低层的 prev 节点（长度 = MAX_LEVEL，找不到就传 head）
pub fn upsert(ctx: Context<Upsert>, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
    // 权限控制
    require!(!ctx.accounts.auth_config.paused, Error::Paused);

    require!(key.len() <= MAX_KEY_LEN, Error::KeyTooLong);
    require!(value.len() <= MAX_VALUE_LEN, Error::ValueTooLarge);

    // 2️⃣ 收费（按数据大小）
    let fee_config = &ctx.accounts.fee_config;

    let fee = fee_config.base_fee + (value.len() as u64 * fee_config.fee_per_byte);

    charge(
        &ctx.accounts.signer,
        &ctx.accounts.treasury.to_account_info(),
        fee,
    )?;

    let _meta = &ctx.accounts.meta;
    let new_node = &mut ctx.accounts.new_node;
    let value_acc = &mut ctx.accounts.value_account;

    // 1️⃣ 计算 level
    let lvl = calc_level(&key);

    // 2️⃣ 填充 new node
    new_node.key = key.clone();
    new_node.value = value_acc.key();
    new_node.level = lvl;
    new_node.forward = [Pubkey::default(); MAX_LEVEL];

    // 3️⃣ 写 value
    value_acc.data = value;

    // 4️⃣ 处理 prev（来自 remaining_accounts）
    // 约定：remaining_accounts 长度 = MAX_LEVEL
    if ctx.remaining_accounts.len() != MAX_LEVEL {
        return err!(Error::InvalidRemaining);
    }

    // 5️⃣ 更新 forward
    for i in 0..(lvl as usize) {
        // 1. 获取 AccountInfo 的引用
        let prev_info = &ctx.remaining_accounts[i];

        // 2. 手动反序列化数据
        // 这样不会产生复杂的生命周期绑定，直接从 data 借用
        let mut prev_data = prev_info.try_borrow_mut_data()?;

        // 假设你的 SkipNode 结构体已经 derive 了 AnchorSerialize/Deserialize
        // 跳过 Anchor 的 8 字节 discriminator
        let mut prev = SkipNode::deserialize(&mut &prev_data[8..])?;

        // new.forward[i] = prev.forward[i]
        new_node.forward[i] = prev.forward[i];

        // prev.forward[i] = new
        prev.forward[i] = new_node.key();

        // 4. 手动写回数据 (这一步非常重要，替代了 exit)
        let mut writer = &mut prev_data[8..];
        prev.serialize(&mut writer)?;
    }

    Ok(())
}
