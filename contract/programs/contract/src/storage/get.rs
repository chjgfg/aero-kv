use anchor_lang::prelude::*;

use crate::{
    constants::{HEAD_SEEDS, META_SEEDS, NODE_SEEDS},
    error::Error,
    storage::{
        storage_structs::{SkipListMeta, SkipNode, ValueAccount},
        utils::key_cmp,
    },
};

#[derive(Accounts)]
pub struct Get<'info> {
    #[account(seeds=[META_SEEDS], bump)]
    pub meta: Account<'info, SkipListMeta>,

    #[account(seeds=[HEAD_SEEDS], bump)]
    pub head: Account<'info, SkipNode>,

    #[account(seeds=[NODE_SEEDS, target.key.as_slice()], bump)]
    pub target: Account<'info, SkipNode>,

    pub value_account: Account<'info, ValueAccount>,
}

pub fn get(ctx: Context<Get>, key: Vec<u8>) -> Result<Vec<u8>> {
    msg!("Greetings from: {:?}", ctx.program_id);
    // 1. 用一个变量标记最后匹配成功的“前驱节点”在哪里
    // -1 表示 head，0 及以上表示 remaining_accounts 的索引
    let mut last_found_idx: i32 = -1;

    // 使用普通的 range 循环，通过下标访问
    for i in 0..ctx.remaining_accounts.len() {
        let acc = &ctx.remaining_accounts[i];

        // 1. 获取数据进行比较 (这里只读，不需要长寿命引用)
        let node_data = acc.try_borrow_data()?;
        let node = SkipNode::deserialize(&mut &node_data[8..])?;

        // 2. 比较逻辑
        if key_cmp(&node.key, &key) == core::cmp::Ordering::Less {
            last_found_idx = i as i32;
        }
    }

    // 1. 获取原始数据字节
    let current_node = if last_found_idx == -1 {
        // 直接从 head 借用并反序列化，注意链式调用中间不留引用变量
        SkipNode::deserialize(&mut &ctx.accounts.head.to_account_info().data.borrow()[8..])?
    } else {
        // 直接从 remaining_accounts 借用
        SkipNode::deserialize(
            &mut &ctx.remaining_accounts[last_found_idx as usize]
                .data
                .borrow()[8..],
        )?
    };

    // 2. 验证
    let next_key = current_node.forward[0];
    if next_key != ctx.accounts.target.key() {
        return err!(Error::NotFound);
    }

    let target = &ctx.accounts.target;
    if target.key != key {
        return err!(Error::NotFound);
    }

    Ok(ctx.accounts.value_account.data.clone())
}
