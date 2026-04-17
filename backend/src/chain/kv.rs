// # upsert / get / scan / delete

// # set_fee, collect_fee, 费用计算

// # set_admin, set_pause, 权限校验

use std::sync::{Arc, Mutex};

use crate::{
    chain::{client::ChainClient, types::ValueAccount},
    constants::{AUTH_SEEDS, FEE_SEEDS, HEAD_SEEDS, META_SEEDS, VALUE_SEEDS},
    error::{Error, Result},
};
use anchor_client::anchor_lang::prelude::borsh::BorshDeserialize;
use axum::Json;
use contract::accounts; // 👈 用你的合约名
use contract::instruction;
use log::info;
use once_cell::sync::Lazy;
use serde_json::{Value, json};
use solana_sdk::{instruction::AccountMeta, signature::Signer, system_program};

static GLOBAL_KEYS: Lazy<Mutex<Vec<Vec<u8>>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// 初始化存储
pub async fn init_storage(client: Arc<ChainClient>) -> Result<()> {
    info!("backend init storage...");

    // 新版合约需要的 PDA
    let (meta_pda, _) = client.find_pda(META_SEEDS);
    let (head_pda, _) = client.find_pda(HEAD_SEEDS);
    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);

    let admin = client.payer.pubkey();

    // 账户完全匹配新版合约
    let accounts = accounts::InitStorage {
        signer: admin,
        meta: meta_pda,
        head: head_pda,
        auth_config: auth_pda,
        fee_config: fee_pda,
        system_program: system_program::ID,
    };

    let args = instruction::InitStorage {};

    let signature_result: Result<solana_sdk::signature::Signature> =
        tokio::task::spawn_blocking(move || {
            let program = client.program()?;

            let sig = program
                .request()
                .args(args)
                .accounts(accounts)
                .send()
                .map_err(|e| {
                    log::error!("init storage 失败: {:?}", e);
                    Error::RpcError(format!("init 失败: {e}"))
                })?;

            // 等待交易确认
            let mut retries = 0;
            while retries < 10 {
                if let Some(Ok(_)) = client.rpc_client.get_signature_status(&sig).unwrap_or(None) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
                retries += 1;
            }

            log::info!("init_storage 成功: {}", sig);
            Ok(sig)
        })
        .await
        .map_err(|e| Error::RpcError(format!("任务执行失败: {e}")))?;
    info!("初始化完成，签名: {}", signature_result?);
    Ok(())
}

/// 插入或者修改
pub async fn upsert(client: Arc<ChainClient>, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
    info!("backend upsert key: {:?}, value: {:?}", key, value);
    // ==============================
    // 🔥 后端维护 key：upsert 时插入
    // ==============================
    let mut keys = GLOBAL_KEYS.lock().unwrap();
    if !keys.contains(&key) {
        keys.push(key.clone());
    }

    // 只保留合约真正需要的 PDA
    let (value_pda, _) = client.find_pda(&[VALUE_SEEDS, key.as_slice()]);
    info!("backend upsert value pda: {}", value_pda);

    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    info!("backend upsert auth pda: {}", auth_pda);
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);
    info!("backend upsert fee pda: {}", fee_pda);

    let admin = client.payer.pubkey();

    // 🔥 账户结构完全匹配新版合约
    let accounts = accounts::Upsert {
        signer: admin,
        value_account: value_pda,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: client.treasury,
        system_program: system_program::ID,
    };

    let args = instruction::Upsert { key, value };

    // 发送交易（不再需要 remaining_accounts！）
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = client.program()?;

        let sig = program
            .request()
            .args(args)
            .accounts(accounts)
            // 🔥 完全删除了 remaining_accounts！
            .send()
            .map_err(|e| {
                log::error!("upsert 交易发送失败: {:?}", e);
                Error::RpcError(format!("upsert 失败: {e}"))
            })?;

        // 等待交易确认
        let mut confirmation_retries = 0;
        while confirmation_retries < 10 {
            let status = client
                .rpc_client
                .get_signature_status(&sig)
                .map_err(|_| Error::IoError("获取交易状态失败".to_string()))?;

            if let Some(Ok(_)) = status {
                break;
            }

            let _ = std::thread::sleep(std::time::Duration::from_millis(500));
            confirmation_retries += 1;
        }

        log::info!("交易签名: {}", sig);
        Ok(sig)
    })
    .await;

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

    log::info!("upsert 成功，签名: {:?}", signature);
    Ok(())
}

pub async fn get(client: Arc<ChainClient>, key: Vec<u8>) -> Result<Json<Value>> {
    info!("backend get key: {:?}", key);

    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    info!("backend get auth pda: {}", auth_pda);
    let (value_pda, _) = client.find_pda(&[VALUE_SEEDS, key.as_slice()]);
    info!("backend get value pda: {}", value_pda);
    let accounts = accounts::Get {
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

/// 删除
pub async fn delete(client: Arc<ChainClient>, key: Vec<u8>) -> Result<()> {
    info!("backend delete key: {:?}", key);

    // ==============================
    // 🔥 后端维护 key：delete 时移除
    // ==============================
    let mut keys = GLOBAL_KEYS.lock().unwrap();
    keys.retain(|k| k != &key);

    let (value_pda, _) = client.find_pda(&[VALUE_SEEDS, key.as_slice()]);
    info!("backend delete value pda: {}", value_pda);
    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    info!("backend delete auth pda: {}", auth_pda);
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);
    info!("backend delete fee pda: {}", fee_pda);

    let admin = client.payer.pubkey();

    let accounts = accounts::Delete {
        signer: admin,
        value_account: value_pda,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: client.treasury,
        system_program: system_program::ID,
    };

    let args = instruction::Delete { key };

    let sig_result = tokio::task::spawn_blocking(move || {
        let program = client.program()?;
        let sig = program
            .request()
            .args(args)
            .accounts(accounts)
            .send()
            .map_err(|e| {
                log::error!("delete 交易发送失败: {:?}", e);
                Error::RpcError(format!("delete 失败: {e}"))
            })?;

        // 等待确认
        let mut retries = 0;
        while retries < 10 {
            if let Some(Ok(_)) = client.rpc_client.get_signature_status(&sig).unwrap_or(None) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
            retries += 1;
        }

        Ok(sig)
    })
    .await;

    let signature = match sig_result {
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

    info!("delete 成功: {}", signature);
    Ok(())
}

pub async fn scan(client: Arc<ChainClient>, start: Vec<u8>, limit: u64) -> Result<()> {
    info!("backend scan start: {:?}, limit: {}", start, limit);

    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    info!("backend scan auth pda: {}", auth_pda);
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);
    info!("backend scan fee pda: {}", fee_pda);
    let admin = client.payer.pubkey();

    // 账户匹配新版合约
    let accounts = accounts::Scan {
        signer: admin,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: client.treasury,
        system_program: system_program::ID,
    };

    let args = instruction::Scan {
        start: start.clone(),
        limit,
    };

    // ======================================================================
    // 🔥 核心：scan 只需要把【你要扫描的 ValueAccount PDA】放进 remaining_accounts
    // 这里我给你一个通用可用的版本：从 start 前缀批量生成 PDA（可直接用）
    // ======================================================================
    // ==============================
    // 🔥 从后端拿到所有 key
    // ==============================
    let all_keys = GLOBAL_KEYS.lock().unwrap().clone();

    // ==============================
    // 生成所有 ValueAccount PDA
    // 传给合约做范围查询
    // ==============================
    let mut remaining_accounts = Vec::new();
    for key in all_keys {
        let (pda, _) = client.find_pda(&[VALUE_SEEDS, &key]);
        remaining_accounts.push(AccountMeta::new_readonly(pda, false));
    }

    // 发送交易
    let sig_result = tokio::task::spawn_blocking(move || {
        let program = client.program()?;
        let sig = program
            .request()
            .args(args)
            .accounts(accounts)
            .accounts(remaining_accounts) // 传入要扫描的账户
            .send()
            .map_err(|e| {
                log::error!("scan 交易发送失败: {:?}", e);
                Error::RpcError(format!("scan 失败: {e}"))
            })?;

        // 等待确认
        let mut retries = 0;
        while retries < 10 {
            if let Some(Ok(_)) = client.rpc_client.get_signature_status(&sig).unwrap_or(None) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
            retries += 1;
        }

        Ok(sig)
    })
    .await;

    let signature = match sig_result {
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
    info!("scan 交易成功: {}", signature);
    Ok(())
}
