// # upsert / get / scan / delete

// # set_fee, collect_fee, 费用计算

// # set_admin, set_pause, 权限校验

use std::sync::Arc;

use crate::{
    chain::{client::ChainClient, types::ValueAccount},
    constants::{
        AUTH_SEEDS, FEE_SEEDS, HEAD_SEEDS, MAX_LEVEL, META_SEEDS, NODE_SEEDS, VALUE_SEEDS,
    },
    error::{Error, Result},
};
use anchor_client::anchor_lang::prelude::borsh::BorshDeserialize;
use axum::Json;
use contract::accounts; // 👈 用你的合约名
use contract::instruction;
use log::info;
use serde_json::{Value, json};
use solana_sdk::{
    instruction::AccountMeta,
    signature::{Keypair, Signer},
    system_program,
};

/// 初始化存储
pub async fn init_storage(client: Arc<ChainClient>) -> Result<()> {
    let (meta_pda, _) = client.find_pda(META_SEEDS);
    let (head_pda, _) = client.find_pda(HEAD_SEEDS);
    let admin = client.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::InitStorage {
        signer: admin,
        meta: meta_pda,
        head: head_pda,
        system_program: system_program::ID,
    };
    let signature = tokio::task::spawn_blocking(move || {
        let program = client.program()?;
        program
            .request()
            .args(instruction::InitStorage {}) // Anchor 调用必须传 args，哪怕是空
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("init_storage 失败: {e}")))
    })
    .await
    .map_err(|e| Error::RpcError(format!("任务执行失败: {e}")))?; // 处理任务 panic
    info!("init storage 成功，签名: {}", signature?);
    Ok(())
}

/// 插入或者修改
pub async fn upsert(client: Arc<ChainClient>, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
    info!("upsert key: {:?}, value: {:?}", key, value);
    let (meta_pda, _) = client.find_pda(META_SEEDS);
    // info!("Meta PDA: {}", meta_pda);
    let (node_pda, _) = client.find_pda(&[NODE_SEEDS, key.as_slice()]);
    info!("Node PDA: {}", node_pda);
    let (value_pda, _) = client.find_pda(&[VALUE_SEEDS, key.as_slice()]);
    info!("Value PDA: {}", value_pda);
    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    // info!("Auth PDA: {}", auth_pda);
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);
    // info!("Fee PDA: {}", fee_pda);
    let (head_pda, _) = client.find_pda(HEAD_SEEDS);
    // info!("Head PDA: {}", head_pda);
    let admin = client.payer.pubkey();
    // info!("Signer: {}", admin);
    let treasury = Keypair::new();
    let treasury_pubkey = treasury.pubkey();
    // info!("Treasury: {}", treasury_pubkey);
    let accounts = accounts::Upsert {
        signer: admin,
        meta: meta_pda,
        new_node: node_pda,
        value_account: value_pda,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: treasury_pubkey,
        system_program: system_program::ID,
    };
    let args = instruction::Upsert { key, value };

    // 3. 🔥 核心修复：把同步阻塞调用放到 tokio::task::spawn_blocking 里
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = client.program()?;
        // 重点：在这里手动添加剩余账户
        // const MAX_LEVEL: usize = 8; // 必须和合约一致
        let remaining_metas: Vec<AccountMeta> = (0..MAX_LEVEL)
            .map(|_| AccountMeta::new(head_pda, false)) // true = is_writable, false = is_signer
            .collect();
        let sig = program
            .request()
            .args(args)
            .accounts(accounts)
            .accounts(remaining_metas)
            .send()
            .map_err(|e| {
                // 这里直接打印错误
                log::error!("upsert 交易发送失败: {:?}", e);
                Error::RpcError(format!("upsert 失败: {e}"))
            })?;

        // 💡 关键：阻塞直到交易被确认
        let mut confirmation_retries = 0;
        while confirmation_retries < 10 {
            let status = client
                .rpc_client
                .get_signature_status(&sig)
                .map_err(|_| Error::IoError("()".to_string()))?;
            if let Some(Ok(_)) = status {
                break; // 交易已确认，可以安全返回了
            }
            let _ = tokio::time::sleep(std::time::Duration::from_millis(500));
            confirmation_retries += 1;
        }

        log::info!("交易签名: {}", sig);
        Ok((sig, value_pda))
    })
    .await; // 处理任务 panic
    let signature = match signature_result {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => {
            log::error!("交易执行失败: {:?}", e);
            return Err(e);
        }
        Err(e) => {
            log::error!("spawn_blocking 任务失败: {:?}", e);
            return Err(Error::RpcError(format!("任务执行失败: {e}")));
        }
    };
    // 🔥 关键修复：解构元组，分别打印
    // let (sig, value_pda) = signature?;
    // 现在可以分别打印了！
    log::info!("upsert 成功，签名: {:?}", signature);
    // log::info!("upsert 成功，签名: {}", sig);
    // log::info!("创建的 value PDA: {}", value_pda);
    Ok(())
}

/// 删除
pub async fn delete(client: Arc<ChainClient>, key: Vec<u8>) -> Result<()> {
    info!("delete key: {:?}", key);
    let (meta_pda, _) = client.find_pda(META_SEEDS);
    let (node_pda, _) = client.find_pda(&[NODE_SEEDS, key.as_slice()]);
    info!("Node PDA: {}", node_pda);
    let (value_pda, _) = client.find_pda(&[VALUE_SEEDS, key.as_slice()]);
    info!("Value PDA: {}", value_pda);
    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);
    let (head_pda, _) = client.find_pda(HEAD_SEEDS);
    // info!("Head PDA: {}", head_pda);
    let admin = client.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let treasury = Keypair::new();
    let treasury_pubkey = treasury.pubkey();
    // info!("Treasury: {}", treasury_pubkey);
    let accounts = accounts::Delete {
        signer: admin,
        meta: meta_pda,
        target: node_pda,
        value_account: value_pda,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: treasury_pubkey,
        system_program: system_program::ID,
    };
    let args = instruction::Delete { key };
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = client.program()?;
        // 1️⃣ 必须传入 remaining_accounts
        // 在测试/简单阶段，如果你还没实现查找前驱逻辑，
        // 至少要传入 MAX_LEVEL 个 head_pda 占位（参考你测试脚本的逻辑）
        // const MAX_LEVEL: usize = 16;
        let remaining_accounts_vec: Vec<AccountMeta> = (0..MAX_LEVEL)
            .map(|_| AccountMeta::new(head_pda, false))
            .collect();
        program
            .request()
            .args(args)
            .accounts(accounts)
            .accounts(remaining_accounts_vec)
            .send()
            .map_err(|e| Error::RpcError(format!("delete 失败: {e}")))
    })
    .await; // 处理任务 panic
    let signature = match signature_result {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => {
            log::error!("交易执行失败: {:?}", e);
            return Err(e);
        }
        Err(e) => {
            log::error!("spawn_blocking 任务失败: {:?}", e);
            return Err(Error::RpcError(format!("任务执行失败: {e}")));
        }
    };
    info!("delete 成功，签名: {}", signature);
    Ok(())
}

pub async fn get(client: Arc<ChainClient>, key: Vec<u8>) -> Result<Json<Value>> {
    info!("backend get key: {:?}", key);
    let (meta_pda, _) = client.find_pda(META_SEEDS);
    info!("backend get mete pda: {}", meta_pda);
    let (head_pda, _) = client.find_pda(HEAD_SEEDS);
    info!("backend get head pda: {}", head_pda);
    let (node_pda, _) = client.find_pda(&[NODE_SEEDS, key.as_slice()]);
    info!("backend get node pda: {}", node_pda);
    let (value_pda, _) = client.find_pda(&[VALUE_SEEDS, key.as_slice()]);
    info!("backend get value pda: {}", value_pda);
    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    info!("backend get auth pda: {}", auth_pda);
    let accounts = accounts::Get {
        meta: meta_pda,
        head: head_pda,
        target: node_pda,
        auth_config: auth_pda,
        value_account: value_pda,
    };
    let args = instruction::Get { key };

    // --- 🔥 关键修复开始：先克隆，再 move ---
    // 在所有权被 move 之前，先克隆两份 Arc
    let client_for_tx = Arc::clone(&client);
    let client_for_read = Arc::clone(&client);
    let sig_result = tokio::task::spawn_blocking(move || {
        let program = client_for_tx.program()?;
        program
            .request()
            .args(args)
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("get 失败: {e}")))
    })
    .await
    .map_err(|e| Error::RpcError(format!("任务执行失败: {e}")))?; // 处理任务 panic
    let signature = match sig_result {
        Ok(sig) => sig,
        Err(e) => {
            // 如果是账户不存在，这里可以拦截并返回友好的提示
            log::warn!("⚠️ 链上执行失败（可能账户已被删除）: {:?}", e);
            return Ok(Json(json!({
                "status": "not_found",
                "message": "账户已删除或不存在"
            })));
        }
    };
    info!("get 成功，签名: {}", signature);

    let raw_data_res: Result<Vec<u8>> = tokio::task::spawn_blocking(move || {
        client_for_read
            .rpc_client
            .get_account_data(&value_pda)
            .map_err(|e| Error::RpcError(format!("读取链上数据失败: {e}")))
    })
    .await
    .map_err(|e| Error::RpcError(format!("读取任务执行失败: {e}")))?;
    match raw_data_res {
        Ok(data) => {
            // 调用时手动跳过 8 字节判别码
            let mut data_slice = &data[8..]; // 直接跳过前 8 位
            // 使用 Anchor 的反序列化方法，它会自动处理那 8 字节并解析 Vec<u8>
            let value_acc = ValueAccount::deserialize(&mut data_slice)
                .map_err(|e| Error::RpcError(format!("反序列化失败: {e}")))?;

            // 解析出真正的字符串内容
            let value_str = String::from_utf8_lossy(&value_acc.data);
            log::info!("✅ 最终解析出的值: {}", value_str);
            Ok(Json(json!({ "status": "success", "data": value_str })))
        }
        _ => {
            // 账户不存在（返回 404 语义，但不崩溃）
            Ok(Json(
                json!({ "status": "not_found", "message": "Key does not exist or has been deleted" }),
            ))
        }
    }
}

pub async fn scan(client: Arc<ChainClient>, start: Vec<u8>, limit: u64) -> Result<()> {
    let (meta_pda, _) = client.find_pda(META_SEEDS);
    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);
    let (head_pda, _) = client.find_pda(HEAD_SEEDS);
    let admin = client.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::Scan {
        signer: admin,
        meta: meta_pda,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: system_program::ID,
    };
    let args = instruction::Scan { start, limit };
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = client.program()?;
        let remaining_accounts_vec: Vec<AccountMeta> = (0..MAX_LEVEL)
            .map(|_| AccountMeta::new(head_pda, false))
            .collect();
        program
            .request()
            .args(args)
            .accounts(accounts)
            .accounts(remaining_accounts_vec)
            .send()
            .map_err(|e| Error::RpcError(format!("scan 失败: {e}")))
    })
    .await; // 处理任务 panic
    let signature = match signature_result {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => {
            log::error!("交易执行失败: {:?}", e);
            return Err(e);
        }
        Err(e) => {
            log::error!("spawn_blocking 任务失败: {:?}", e);
            return Err(Error::RpcError(format!("任务执行失败: {e}")));
        }
    };
    info!("scan 成功，签名: {}", signature);
    Ok(())
}
