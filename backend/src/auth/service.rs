// # set_admin, set_pause, 权限校验

use crate::{
    chain::client::ChainClient,
    constants::AUTH_SEEDS,
    error::{Error, Result},
};
use contract::accounts; // 👈 用你的合约名
use contract::instruction;
use solana_sdk::{pubkey::Pubkey, signature::Signer, system_program};

impl ChainClient {
    /// 初始化权限
    pub async fn init_auth(&self) -> Result<()> {
        let program = self.program()?;
        let (auth_pda, _) = self.find_pda(AUTH_SEEDS);
        let admin = self.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
        let accounts = accounts::InitAuth {
            signer: admin,
            auth_config: auth_pda,
            system_program: system_program::ID,
        };
        program
            .request()
            .args(instruction::InitAuth {}) // Anchor 调用必须传 args，哪怕是空
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("init_auth 失败: {e}")))?;
        Ok(())
    }

    /// 重置权限
    pub async fn set_admin(&self, new_admin: Pubkey) -> Result<()> {
        let program = self.program()?;
        let (auth_pda, _) = self.find_pda(AUTH_SEEDS);
        let admin = self.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
        let accounts = accounts::SetAdmin {
            signer: admin,
            auth_config: auth_pda,
        };
        let args = instruction::SetAdmin { new_admin };
        program
            .request()
            .args(args)
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("set_admin 失败: {e}")))?;
        Ok(())
    }

    /// 暂停/开启合约
    pub async fn set_pause(&self, paused: bool) -> Result<()> {
        let program = self.program()?;
        let (auth_pda, _) = self.find_pda(AUTH_SEEDS);
        let admin = self.payer.try_pubkey().map_err(|_| Error::PubKeyError)?;
        let accounts = accounts::SetPause {
            signer: admin,
            auth_config: auth_pda,
        };
        let args = instruction::SetPause { paused };
        program
            .request()
            .args(args)
            .accounts(accounts)
            .send()
            .map_err(|e| Error::RpcError(format!("set_pause 失败: {e}")))?;
        Ok(())
    }
}
