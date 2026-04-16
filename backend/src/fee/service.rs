// # set_fee, collect_fee, 费用计算

// # set_admin, set_pause, 权限校验

use crate::{
    chain::client::ChainClient,
    constants::{AUTH_SEEDS, FEE_SEEDS},
    error::{Error, Result},
};
use contract::accounts; // 👈 用你的合约名
use contract::instruction;
use solana_sdk::{signature::Signer, system_program};

/// 初始化费用
pub fn init_fee(client: &ChainClient) -> Result<()> {
    let program = client.program()?;
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);
    let admin = client.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::InitFee {
        signer: admin,
        fee_config: fee_pda,
        system_program: system_program::ID,
    };
    program
        .request()
        .args(instruction::InitFee {}) // Anchor 调用必须传 args，哪怕是空
        .accounts(accounts)
        .send()
        .map_err(|e| Error::RpcError(format!("init_fee 失败: {e}")))?;
    Ok(())
}

/// 重置权限
pub fn set_fee(
    client: &ChainClient,
    base_fee: u64,
    fee_per_byte: u64,
    scan_fee_per_item: u64,
) -> Result<()> {
    let program = client.program()?;
    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);
    let admin = client.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
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
    program
        .request()
        .args(args)
        .accounts(accounts)
        .send()
        .map_err(|e| Error::RpcError(format!("set_fee 失败: {e}")))?;
    Ok(())
}
