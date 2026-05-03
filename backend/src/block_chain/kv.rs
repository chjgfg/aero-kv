// # upsert / get / scan / delete

// # set_fee, collect_fee, 费用计算

// # set_admin, set_pause, 权限校验

use crate::{
    block_chain::{
        client::ChainClient,
        types::{KVEvent, ValueAccount},
    },
    constants::{
        AUTH_SEEDS, COUNTER_SEEDS, FEE_SEEDS, HEAD_SEEDS, META_SEEDS, SYS_BASE_FEE, SYS_PAUSED,
        VALUE_SEEDS,
    },
    error::{Error, Result},
    storage::engine::DiskClient,
    utils::bytes_to_str,
};
use anchor_client::anchor_lang::prelude::borsh::BorshDeserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

use base64::{Engine, engine::general_purpose};
use contract::accounts; // 👈 用你的合约名
use contract::instruction;
use log::{error, info, warn};
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Signature, Signer},
    system_program,
};

/// 初始化存储
pub async fn init_storage(chain: Arc<ChainClient>) -> Result<Signature> {
    info!("backend init storage...");
    // 新版合约需要的 PDA
    let (meta_pda, _) = chain.find_pda(META_SEEDS);
    let (head_pda, _) = chain.find_pda(HEAD_SEEDS);
    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    let (fee_pda, _) = chain.find_pda(FEE_SEEDS);

    let admin = chain.payer.pubkey();

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
            let program = chain.program()?;

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
                if let Some(Ok(_)) = chain.rpc_client.get_signature_status(&sig).unwrap_or(None) {
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
    let signature = signature_result?;
    info!("初始化完成，签名: {}", signature);
    Ok(signature)
}

pub async fn init_counter(chain: Arc<ChainClient>) -> Result<Signature> {
    info!("backend init counter...");
    // 新版合约需要的 PDA
    let (counter_pda, _) = chain.find_pda(COUNTER_SEEDS);
    info!("backend upsert counter pda: {}", counter_pda);

    let admin = chain.payer.pubkey();

    // 账户完全匹配新版合约
    let accounts = accounts::InitCounter {
        signer: admin,
        counter: counter_pda,
        system_program: system_program::ID,
    };

    let args = instruction::InitCounter {};

    let signature_result: Result<solana_sdk::signature::Signature> =
        tokio::task::spawn_blocking(move || {
            let program = chain.program()?;

            let sig = program
                .request()
                .args(args)
                .accounts(accounts)
                .send()
                .map_err(|e| {
                    log::error!("init counter 失败: {:?}", e);
                    Error::RpcError(format!("init 失败: {e}"))
                })?;

            // 等待交易确认
            let mut retries = 0;
            while retries < 10 {
                if let Some(Ok(_)) = chain.rpc_client.get_signature_status(&sig).unwrap_or(None) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
                retries += 1;
            }

            log::info!("init_counter 成功: {}", sig);
            Ok(sig)
        })
        .await
        .map_err(|e| Error::RpcError(format!("任务执行失败: {e}")))?;
    let signature = signature_result?;
    info!("初始化完成，签名: {}", signature);
    Ok(signature)
}

/// 插入或者修改
pub async fn upsert(
    chain: Arc<ChainClient>,
    storage: Arc<Mutex<DiskClient>>,
    key: Vec<u8>,
    value: Vec<u8>,
) -> Result<Signature> {
    info!("backend upsert key: {:?}, value: {:?}", key, value);

    // 只保留合约真正需要的 PDA
    let (value_pda, _) = chain.find_pda(&[VALUE_SEEDS, key.as_slice()]);
    info!("backend upsert value pda: {}", value_pda);

    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    info!("backend upsert auth pda: {}", auth_pda);
    let (fee_pda, _) = chain.find_pda(FEE_SEEDS);
    info!("backend upsert fee pda: {}", fee_pda);
    // ✅ 新增：获取计数器的 PDA
    let (counter_pda, _) = chain.find_pda(COUNTER_SEEDS);
    info!("backend upsert counter pda: {}", counter_pda);

    {
        let mut disk = storage.lock().await;
        let _ = disk.set(key.clone(), value_pda.to_bytes().to_vec());
    }

    let admin = chain.payer.pubkey();

    // 🔥 账户结构完全匹配新版合约
    let accounts = accounts::Upsert {
        signer: admin,
        value_account: value_pda,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: chain.treasury,
        counter: counter_pda,
        system_program: system_program::ID,
    };

    let args = instruction::Upsert { key, value };

    // 发送交易（不再需要 remaining_accounts！）
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = chain.program()?;

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
            let status = chain
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
    Ok(signature)
}

pub async fn get(chain: Arc<ChainClient>, key: Vec<u8>) -> Result<String> {
    if key == SYS_PAUSED || key == SYS_BASE_FEE {
        info!("跳过系统配置项: {:?}", key);
        return Err(Error::InvalidKey);
    }
    info!("backend get key: {:?}", key);

    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    info!("backend get auth pda: {}", auth_pda);
    let (value_pda, _) = chain.find_pda(&[VALUE_SEEDS, key.as_slice()]);
    info!("backend get value pda: {}", value_pda);
    let accounts = accounts::Get {
        auth_config: auth_pda,
        value_account: value_pda,
    };
    let args = instruction::Get { key };
    // --- 🔥 关键修复开始：先克隆，再 move ---
    // 在所有权被 move 之前，先克隆两份 Arc
    let client_for_tx = Arc::clone(&chain);
    let client_for_read = Arc::clone(&chain);
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
            return Ok("not_found".to_string());
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
            Ok(value_str.to_string())
        }
        _ => {
            // 账户不存在（返回 404 语义，但不崩溃）
            Ok("not_found".to_string())
        }
    }
}

/// 删除
pub async fn delete(
    chain: Arc<ChainClient>,
    storage: Arc<Mutex<DiskClient>>,
    key: Vec<u8>,
) -> Result<Signature> {
    if key == SYS_PAUSED || key == SYS_BASE_FEE {
        info!("跳过系统配置项: {:?}", key);
        return Err(Error::InvalidKey);
    }

    info!("backend delete key: {:?}", key);

    let (value_pda, _) = chain.find_pda(&[VALUE_SEEDS, key.as_slice()]);
    info!("backend delete value pda: {}", value_pda);
    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    info!("backend delete auth pda: {}", auth_pda);
    let (fee_pda, _) = chain.find_pda(FEE_SEEDS);
    info!("backend delete fee pda: {}", fee_pda);
    // ✅ 新增：获取计数器的 PDA
    let (counter_pda, _) = chain.find_pda(COUNTER_SEEDS);
    info!("backend delete counter pda: {}", counter_pda);

    {
        let mut disk = storage.lock().await;
        let _ = disk.delete(key.clone());
    }

    let admin = chain.payer.pubkey();

    let accounts = accounts::Delete {
        signer: admin,
        value_account: value_pda,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: chain.treasury,
        counter: counter_pda,
        system_program: system_program::ID,
    };

    let args = instruction::Delete { key };

    let sig_result = tokio::task::spawn_blocking(move || {
        let program = chain.program()?;
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
            if let Some(Ok(_)) = chain.rpc_client.get_signature_status(&sig).unwrap_or(None) {
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
    Ok(signature)
}

pub async fn scan(
    chain: Arc<ChainClient>,
    storage: Arc<Mutex<DiskClient>>,
    start: Vec<u8>,
    limit: u64,
) -> Result<Vec<(String, String)>> {
    if start.is_empty() {
        return Err(Error::InvalidKey);
    }
    info!("backend scan start: {:?}, limit: {}", start, limit);

    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    info!("backend scan auth pda: {}", auth_pda);
    let (fee_pda, _) = chain.find_pda(FEE_SEEDS);
    info!("backend scan fee pda: {}", fee_pda);
    let admin = chain.payer.pubkey();

    let client_for_tx = Arc::clone(&chain);

    // 账户匹配新版合约
    let accounts = accounts::Scan {
        signer: admin,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: chain.treasury,
        system_program: system_program::ID,
    };

    // ======================================================================
    // 🔥 核心：scan 只需要把【你要扫描的 ValueAccount PDA】放进 remaining_accounts
    // 这里我给你一个通用可用的版本：从 start 前缀批量生成 PDA（可直接用）
    // ======================================================================
    // ==============================
    // 🔥 从后端拿到所有 key
    // ==============================
    // 1. 先在外层声明 k_v 变量
    let k_v: Vec<(Vec<u8>, Vec<u8>)>;
    {
        let mut disk = storage.lock().await;
        let iter = disk.scan_prefix(start.clone());
        k_v = iter
            .skip(0)
            .take(limit as usize)
            .collect::<Result<Vec<_>>>()?;
    }

    // 把 key 列表通过指令数据传给合约
    let keys: Vec<Vec<u8>> = k_v
        .iter()
        .filter(|(k, _)| k != &*SYS_PAUSED && k != &*SYS_BASE_FEE)
        .map(|(k, _)| k.clone())
        .collect();
    let args = instruction::Scan {
        start: start.clone(),
        limit,
        keys,
    };
    info!("从bitcask取的数据: {:?}", k_v);
    // ==============================
    // 生成所有 ValueAccount PDA
    // 传给合约做范围查询
    // ==============================
    let mut remaining_accounts = Vec::new();
    for (key, pda) in k_v {
        // 过滤掉所有以 "sys_" 开头的内部配置键
        // --- 关键过滤逻辑：跳过系统配置项 ---
        if key == SYS_PAUSED || key == SYS_BASE_FEE {
            info!("跳过系统配置项: {:?}", key);
            continue;
        }

        // 也可以采用更健壮的长度判断：PDA 必须是 32 字节
        if pda.len() != 32 {
            warn!("跳过长度非 32 字节的数据 (Key: {:?})", key);
            continue;
        }
        info!("key: {:?}", key);
        // let (pda, _) = chain.find_pda(&[VALUE_SEEDS, &key]);
        let value_pda = match Pubkey::try_from(pda) {
            Ok(v) => v,
            Err(e) => {
                error!("从节点查询出错: {:?}", e);
                // 将 Vec<u8> 转换为字符串，方便查看
                let msg = String::from_utf8_lossy(&e).to_string();
                // 包装进你的错误枚举中返回[cite: 1]
                return Err(Error::InternalError(msg));
            }
        };
        remaining_accounts.push(AccountMeta::new_readonly(value_pda, false));
    }

    // 发送交易
    let sig_result = tokio::task::spawn_blocking(move || {
        let program = chain.program()?;
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
            if let Some(Ok(_)) = chain.rpc_client.get_signature_status(&sig).unwrap_or(None) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
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

    // ==============================
    // 用你的 RpcClient 直接获取交易日志（零报错版）
    // ==============================
    let tx = client_for_tx
        .rpc_client
        .get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                // 关键：commitment 要包在 Some() 里
                commitment: Some(CommitmentConfig::confirmed()),
                ..Default::default()
            },
        )
        .map_err(|e| Error::RpcError(format!("获取交易失败: {}", e)))?;

    // 方式 A：先转成标准的 Option，再使用 ok_or_else
    let meta = tx
        .transaction
        .meta
        .ok_or_else(|| Error::RpcError("交易没有 meta 信息".to_string()))?;
    let logs: Vec<String> = Option::from(meta.log_messages)
        .ok_or_else(|| Error::RpcError("交易没有日志".to_string()))?;

    info!("=== 交易日志（RPC 获取）===");
    for log in &logs {
        info!("{}", log);
    }

    // 解析事件
    let mut results: Vec<(String, String)> = Vec::new();
    for log in logs {
        // 这里给 log 加个类型注解，解决 cannot infer type
        let log: String = log;
        // 匹配 Program data: 开头的日志
        if log.starts_with("Program data: ") {
            info!("匹配到 Program data 日志: {:?}", log);
            let data = log.replace("Program data: ", "");

            // 替换为推荐的 base64 解码方式，解决弃用警告
            let bytes = general_purpose::STANDARD.decode(data).unwrap_or_default();

            info!("解码后的字节长度: {}", bytes.len());

            // Anchor 事件的前 8 字节是 discriminator，必须跳过
            if bytes.len() > 8 {
                match KVEvent::try_from_slice(&bytes[8..]) {
                    Ok(event) => {
                        info!(
                            "✅ 解析成功 -> key: {:?}, value: {:?}",
                            event.key, event.value
                        );
                        let kv = bytes_to_str(&event)
                            .map_err(|e| Error::ParseEmitError(e.to_string()))?;
                        results.push(kv);
                    }
                    Err(e) => {
                        info!("❌ 解析失败: {:?}, data: {:?}", e, &bytes[8..]);
                    }
                }
            }
        }
    }

    info!("扫描完成，{:?}", results);
    info!("扫描完成，共 {} 条数据", results.len());

    Ok(results)
}

pub async fn page(
    chain: Arc<ChainClient>,
    storage: Arc<Mutex<DiskClient>>,
    page: usize,
    limit: usize,
) -> Result<(Vec<(String, String)>, u64)> {
    info!("backend page offset: {:?}, limit: {}", page, limit);

    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    info!("backend page auth pda: {}", auth_pda);
    let (fee_pda, _) = chain.find_pda(FEE_SEEDS);
    info!("backend page fee pda: {}", fee_pda);
    let (counter_pda, _) = chain.find_pda(COUNTER_SEEDS); // 👈 加 counter
    info!("backend page counter pda: {}", counter_pda);
    // ✅ 新增：获取计数器的 PDA
    let (counter_pda, _) = chain.find_pda(COUNTER_SEEDS);
    info!("backend delete counter pda: {}", counter_pda);
    let admin = chain.payer.pubkey();

    let client_for_tx = Arc::new(chain.clone());
    // let client_for_counter = Arc::new(chain.clone());

    // 账户匹配新版合约
    let accounts = accounts::Page {
        signer: admin,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: chain.treasury,
        counter: counter_pda,
        system_program: system_program::ID,
    };

    // ======================================================================
    // 🔥 核心：scan 只需要把【你要扫描的 ValueAccount PDA】放进 remaining_accounts
    // 这里我给你一个通用可用的版本：从 start 前缀批量生成 PDA（可直接用）
    // ======================================================================
    // ==============================
    // 🔥 从后端拿到所有 key
    // ==============================
    let page_data: Vec<(Vec<u8>, Vec<u8>)>;
    {
        let mut disk = storage.lock().await;
        page_data = disk.page(limit, page)?;
    }

    // 把 key 列表通过指令数据传给合约
    let keys: Vec<Vec<u8>> = page_data
        .iter()
        .filter(|(k, _)| k != &*SYS_PAUSED && k != &*SYS_BASE_FEE)
        .map(|(k, _)| k.clone())
        .collect();
    let args = instruction::Page { keys };
    info!("从bitcask取的数据: {:?}", page_data);
    // ==============================
    // 生成所有 ValueAccount PDA
    // 传给合约做范围查询
    // ==============================
    let mut remaining_accounts = Vec::new();
    for (key, pda) in page_data {
        // 过滤掉所有以 "sys_" 开头的内部配置键
        // --- 关键过滤逻辑：跳过系统配置项 ---
        if key == SYS_PAUSED || key == SYS_BASE_FEE {
            info!("跳过系统配置项: {:?}", key);
            continue;
        }

        // 也可以采用更健壮的长度判断：PDA 必须是 32 字节
        if pda.len() != 32 {
            warn!("跳过长度非 32 字节的数据 (Key: {:?})", key);
            continue;
        }
        info!(
            "Key: {}, PDA Raw Data (Hex): {}",
            String::from_utf8_lossy(&key),
            hex::encode(&pda)
        );
        // let (pda, _) = chain.find_pda(&[VALUE_SEEDS, &key]);
        let value_pda = match Pubkey::try_from(pda) {
            Ok(v) => v,
            Err(e) => {
                error!("从节点查询出错: {:?}", e);
                // 将 Vec<u8> 转换为字符串，方便查看
                let msg = String::from_utf8_lossy(&e).to_string();
                // 包装进你的错误枚举中返回[cite: 1]
                return Err(Error::InternalError(msg));
            }
        };
        remaining_accounts.push(AccountMeta::new_readonly(value_pda, false));
    }

    // 发送交易
    let sig_result = tokio::task::spawn_blocking(move || {
        let program = chain.program()?;
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
            if let Some(Ok(_)) = chain.rpc_client.get_signature_status(&sig).unwrap_or(None) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
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

    // ==============================
    // 用你的 RpcClient 直接获取交易日志（零报错版）
    // ==============================
    let tx = client_for_tx
        .rpc_client
        .get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                // 关键：commitment 要包在 Some() 里
                commitment: Some(CommitmentConfig::confirmed()),
                ..Default::default()
            },
        )
        .map_err(|e| Error::RpcError(format!("获取交易失败: {}", e)))?;

    // 方式 A：先转成标准的 Option，再使用 ok_or_else
    let meta = tx
        .transaction
        .meta
        .ok_or_else(|| Error::RpcError("交易没有 meta 信息".to_string()))?;
    let logs: Vec<String> = Option::from(meta.log_messages)
        .ok_or_else(|| Error::RpcError("交易没有日志".to_string()))?;

    info!("=== 交易日志（RPC 获取）===");
    for log in &logs {
        info!("{}", log);
    }

    let mut total_from_log: Option<u64> = None;

    // 解析事件
    let mut results: Vec<(String, String)> = Vec::new();
    for log in logs {
        // 这里给 log 加个类型注解，解决 cannot infer type
        let log: String = log;
        // 匹配 Program data: 开头的日志
        if log.starts_with("Program data: ") {
            info!("匹配到 Program data 日志: {:?}", log);
            let data = log.replace("Program data: ", "");

            // 替换为推荐的 base64 解码方式，解决弃用警告
            let bytes = general_purpose::STANDARD.decode(data).unwrap_or_default();

            info!("解码后的字节长度: {}", bytes.len());

            // Anchor 事件的前 8 字节是 discriminator，必须跳过
            if bytes.len() > 8 {
                match KVEvent::try_from_slice(&bytes[8..]) {
                    Ok(event) => {
                        info!(
                            "✅ 解析成功 -> key: {:?}, value: {:?}",
                            event.key, event.value
                        );
                        let kv = bytes_to_str(&event)
                            .map_err(|e| Error::ParseEmitError(e.to_string()))?;
                        results.push(kv);
                    }
                    Err(e) => {
                        info!("❌ 解析失败: {:?}, data: {:?}", e, &bytes[8..]);
                    }
                }
            }
        }

        // 2. 新增：解析我们在合约里手动打印的 total
        if log.contains("FINAL_TOTAL: ") {
            if let Some(total_str) = log.split("FINAL_TOTAL: ").last() {
                if let Ok(t) = total_str.trim().parse::<u64>() {
                    total_from_log = Some(t);
                    info!("从日志中同步获取到最新 Total: {}", t);
                }
            }
        }
    }

    info!("扫描完成，{:?}", results);
    info!("扫描完成，共 {} 条数据", results.len());
    info!("扫描完成，共 {:?} 条数据", total_from_log);

    // ======================================================
    // 🔥 关键：每次读取 total 都新建客户端，彻底避免缓存
    // ======================================================
    let total =
        total_from_log.ok_or_else(|| Error::RpcError("counter account not found".to_string()))?;
    Ok((results, total))
}
