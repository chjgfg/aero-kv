// # upsert / get / scan / delete

// # set_fee, collect_fee, 费用计算

// # set_admin, set_pause, 权限校验

use crate::{
    chain::client::ChainClient,
    constants::{AUTH_SEEDS, FEE_SEEDS, HEAD_SEEDS, META_SEEDS, NODE_SEEDS, VALUE_SEEDS},
    error::{Error, Result},
};
use contract::accounts; // 👈 用你的合约名
use contract::instruction;
use solana_sdk::{pubkey::Pubkey, signature::Signer, system_program};

/// 初始化存储
pub fn init_storage(client: &ChainClient) -> Result<()> {
    let program = client.program()?;
    let (meta_pda, _) = client.find_pda(META_SEEDS);
    let (head_pda, _) = client.find_pda(HEAD_SEEDS);
    let admin = client.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::InitStorage {
        signer: admin,
        meta: meta_pda,
        head: head_pda,
        system_program: system_program::ID,
    };
    program
        .request()
        .args(instruction::InitStorage {}) // Anchor 调用必须传 args，哪怕是空
        .accounts(accounts)
        .send()
        .map_err(|e| Error::RpcError(format!("init_storage 失败: {e}")))?;
    Ok(())
}

/// 重置权限
pub fn upsert(client: &ChainClient, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
    let program = client.program()?;
    let (meta_pda, _) = client.find_pda(META_SEEDS);
    let (node_pda, _) = client.find_pda(NODE_SEEDS);
    let (value_pda, _) = client.find_pda(VALUE_SEEDS);
    let (auth_pda, _) = client.find_pda(AUTH_SEEDS);
    let (fee_pda, _) = client.find_pda(FEE_SEEDS);
    let admin = client.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
    let accounts = accounts::Upsert {
        signer: admin,
        meta: meta_pda,
        new_node: node_pda,
        value_account: value_pda,
        auth_config: auth_pda,
        fee_config: fee_pda,
        treasury: system_program::ID,
        system_program: system_program::ID,
    };
    let args = instruction::Upsert { key, value };
    program
        .request()
        .args(args)
        .accounts(accounts)
        .send()
        .map_err(|e| Error::RpcError(format!("upsert 失败: {e}")))?;
    Ok(())
}
