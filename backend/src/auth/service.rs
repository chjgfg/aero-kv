// # set_admin, set_pause, 权限校验

use std::sync::Arc;

use crate::{
    block_chain::client::ChainClient,
    constants::AUTH_SEEDS,
    error::{Error, Result},
};
use contract::accounts; // 👈 用你的合约名
use contract::instruction;
use solana_sdk::{pubkey::Pubkey, signature::Signer, system_program};

/// 初始化权限
pub async fn init_auth(chain: Arc<ChainClient>,) -> Result<()> {
    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    let admin = chain.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::InitAuth {
        signer: admin,
        auth_config: auth_pda,
        system_program: system_program::ID,
    };
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = chain.program()?;
        let sig = program
            .request()
            .args(instruction::InitAuth {}) // Anchor 调用必须传 args，哪怕是空
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("init_auth 失败: {e}")))?;
        log::info!("交易签名: {}", sig);
        Ok((sig, auth_pda))
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
    log::info!("init_auth 成功，签名: {:?}", signature);
    Ok(())
}

/// 重置权限
pub async fn set_admin(chain: Arc<ChainClient>, new_admin: Pubkey) -> Result<()> {
    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    let admin = chain.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::SetAdmin {
        signer: admin,
        auth_config: auth_pda,
    };
    let args = instruction::SetAdmin { new_admin };
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = chain.program()?;
        let sig = program
            .request()
            .args(args)
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("set_admin 失败: {e}")))?;
        log::info!("交易签名: {}", sig);
        Ok((sig, auth_pda))
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
    log::info!("set_admin 成功，签名: {:?}", signature);
    Ok(())
}

/// 暂停/开启合约
pub async fn set_pause(chain: Arc<ChainClient>, paused: bool) -> Result<()> {
    let (auth_pda, _) = chain.find_pda(AUTH_SEEDS);
    let admin = chain.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::SetPause {
        signer: admin,
        auth_config: auth_pda,
    };
    let args = instruction::SetPause { paused };
    let signature_result = tokio::task::spawn_blocking(move || {
        let program = chain.program()?;
        let sig = program
            .request()
            .args(args)
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("set_pause 失败: {e}")))?;
        log::info!("交易签名: {}", sig);
        Ok((sig, auth_pda))
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
    log::info!("set_pause 成功，签名: {:?}", signature);
    Ok(())
}
