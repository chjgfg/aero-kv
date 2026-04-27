// # set_fee, collect_fee, 费用计算

// # set_admin, set_pause, 权限校验

use std::sync::Arc;

use crate::{
    block_chain::client::ChainClient,
    constants::{AUTH_SEEDS, FEE_SEEDS},
    error::{Error, Result},
};
use contract::accounts; // 👈 用你的合约名
use contract::instruction;
use solana_sdk::{signature::{Signature, Signer}, system_program};

/// 初始化费用
pub async fn init_fee(chain: Arc<ChainClient>) -> Result<Signature> {
    let (fee_pda, _) = chain.find_pda(FEE_SEEDS);
    let admin = chain.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::InitFee {
        signer: admin,
        fee_config: fee_pda,
        system_program: system_program::ID,
    };
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = chain.program()?;
        let sig = program
            .request()
            .args(instruction::InitFee {}) // Anchor 调用必须传 args，哪怕是空
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("init_fee 失败: {e}")))?;
        log::info!("交易签名: {}", sig);
        Ok((sig, fee_pda))
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
    log::info!("init_fee 成功，签名: {:?}", signature);
    Ok(signature.0)
}

/// 重置权限
pub async fn set_fee(
    chain: Arc<ChainClient>,
    base_fee: u64,
    fee_per_byte: u64,
    scan_fee_per_item: u64,
) -> Result<Signature> {
    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    let (fee_pda, _) = chain.find_pda(FEE_SEEDS);
    let admin = chain.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::SetFee {
        signer: admin,
        auth_config: auth_pda,
        fee_config: fee_pda,
    };
    let args = instruction::SetFee {
        base_fee,
        fee_per_byte,
        scan_fee_per_item,
    };
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = chain.program()?;
        let sig = program
            .request()
            .args(args)
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("set_fee 失败: {e}")))?;
        log::info!("交易签名: {}", sig);
        Ok((sig, fee_pda))
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
    log::info!("set_fee 成功，签名: {:?}", signature);
    Ok(signature.0)
}
