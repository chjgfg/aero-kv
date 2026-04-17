use anchor_lang::prelude::*;

use crate::{
    auth::structs::AuthConfig,
    constants::{AUTH_SEEDS, FEE_SEEDS, META_SEEDS},
    error::Error,
    fee::FeeConfig,
    storage::structs::{SkipListMeta, SkipNode, ValueAccount},
    utils::charge,
};

#[event]
pub struct KVPair {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}
#[derive(Accounts)]
pub struct Scan<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // ⭐ scan 也要收费

    #[account(seeds=[META_SEEDS], bump)]
    pub meta: Account<'info, SkipListMeta>,

    #[account(seeds = [AUTH_SEEDS], bump)]
    pub auth_config: Account<'info, AuthConfig>,

    #[account(seeds = [FEE_SEEDS], bump)]
    pub fee_config: Account<'info, FeeConfig>,

    #[account(mut)]
    pub treasury: SystemAccount<'info>,
}

pub fn scan(ctx: Context<Scan>, start: Vec<u8>, limit: u64) -> Result<()> {
    // 权限控制
    require!(!ctx.accounts.auth_config.paused, Error::Paused);

    msg!("SCAN_START: limit={}, start_len={}", limit, start.len()); // 加上这一行

    let fee_config = &ctx.accounts.fee_config;

    let mut count: u64 = 0;
    let mut i: usize = 0;

    // // 直接使用 ctx.remaining_accounts，不要赋值给中间变量
    // while i + 1 < ctx.remaining_accounts.len() && count < limit {
    //     // 1. 获取 AccountInfo 的引用
    //     let node_info = &ctx.remaining_accounts[i];
    //     let value_info = &ctx.remaining_accounts[i + 1];

    //     // --- 使用独立的作用域块来处理 Node 借用 ---
    //     let should_emit = {
    //         // 2. 手动反序列化 Node 数据
    //         // 直接 borrow 字节流，不涉及 Account 类的生命周期绑定
    //         // 1. 借用数据
    //         let node_data = node_info.try_borrow_data()?; // 如果只是读，用 try_borrow_data
    //                                                       // 2. 解析逻辑
    //         let node = SkipNode::deserialize(&mut &node_data[8..])?;
    //         // 仅仅在这里使用 node.key，不把 node 带出这个块
    //         if node.key >= start {
    //             true
    //         } else {
    //             false
    //         }
    //     }; // node_data 会在这里被百分之百释放

    //     // 3. 过滤逻辑
    //     if should_emit {
    //         // --- 使用独立的作用域块来处理 Value 借用 ---
    //         {
    //             // 手动反序列化 Value 数据
    //             let value_data = value_info.try_borrow_data()?;
    //             let value_acc = ValueAccount::deserialize(&mut &value_data[8..])?;
    //             msg!("DEBUG: About to emit KVPair for key {:?}", start);
    //             emit!(KVPair {
    //                 key: start.clone(),
    //                 value: value_acc.data.clone(),
    //             });
    //             count += 1;
    //         } // value_data 会在这里自动 drop
    //     }
    //     i += 2;
    // }

    // 在 scan 循环中，确保每一次迭代的借用都完全释放
    while i + 1 < ctx.remaining_accounts.len() && count < limit {
        let pair = {
            // 这一对大括号是生命周期的铁闸门
            let node_info = &ctx.remaining_accounts[i];
            let node_data = node_info.try_borrow_data()?;
            let node = SkipNode::deserialize(&mut &node_data[8..])?;
            
            if node.key >= start {
                let value_info = &ctx.remaining_accounts[i + 1];
                let value_data = value_info.try_borrow_data()?;
                let value_acc = ValueAccount::deserialize(&mut &value_data[8..])?;
                
                // 将需要的数据克隆出来，退出这个作用域，从而释放借用
                Some(KVPair {
                    key: node.key.clone(),
                    value: value_acc.data.clone(),
                })
            } else {
                None
            }
        }; // <--- 执行到这里，node_data 和 value_data 都会被强制 drop

        if let Some(p) = pair {
            emit!(p);
            count += 1;
        }
        i += 2;
    }

    // 🔥 按返回数量收费
    let fee = fee_config.scan_fee_per_item * count;

    charge(
        &ctx.accounts.signer,
        &ctx.accounts.treasury.to_account_info(),
        fee,
    )?;

    Ok(())
}
