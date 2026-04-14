use anchor_lang::prelude::*;

use crate::{
    constants::META_SEEDS,
    storage::storage_structs::{SkipListMeta, SkipNode, ValueAccount},
};

#[event]
pub struct KVPair {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}
#[derive(Accounts)]
pub struct Scan<'info> {
    #[account(seeds=[META_SEEDS], bump)]
    pub meta: Account<'info, SkipListMeta>,
}

pub fn scan(ctx: Context<Scan>, start: Vec<u8>, limit: u64) -> Result<()> {
    msg!("SCAN_START: limit={}, start_len={}", limit, start.len()); // 加上这一行
    let mut count: u64 = 0;
    let mut i: usize = 0;

    // 直接使用 ctx.remaining_accounts，不要赋值给中间变量
    while i + 1 < ctx.remaining_accounts.len() && count < limit {
        // 1. 获取 AccountInfo 的引用
        let node_info = &ctx.remaining_accounts[i];
        let value_info = &ctx.remaining_accounts[i + 1];

        // 2. 手动反序列化 Node 数据
        // 直接 borrow 字节流，不涉及 Account 类的生命周期绑定
        let node_data = node_info.try_borrow_data()?;
        let node = SkipNode::deserialize(&mut &node_data[8..])?;

        // 3. 过滤逻辑
        if node.key >= start {
            // 手动反序列化 Value 数据
            let value_data = value_info.try_borrow_data()?;
            let value_acc = ValueAccount::deserialize(&mut &value_data[8..])?;
            msg!("DEBUG: About to emit KVPair for key {:?}", node.key);
            emit!(KVPair {
                key: node.key.clone(),
                value: value_acc.data.clone(),
            });
            count += 1;
        }

        i += 2;
    }

    Ok(())
}
