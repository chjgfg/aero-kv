// # 全局配置：RPC、程序ID、PDA种子、钱包

use crate::error::{Error, Result};
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub payer_keypair: String,
    pub rpc_url: String,
    pub program_id: String,
}

impl Config {
    pub fn from_env() -> Result<Config> {
        dotenv().ok();
        let rpc_url: String =
            env::var("RPC_URL").map_err(|e| Error::RpcUrlError(format!("RPC 路径错误: {}", e)))?;
        let program_id = env::var("PROGRAM_ID")
            .map_err(|e| Error::ProgramIdError(format!("Program Id 错误: {}", e)))?;
        let payer_keypair = env::var("PAYER_KEYPAIR")
            .map_err(|e| Error::PayerKeypairError(format!("Payer Keypair 错误: {}", e)))?;
        Ok(Self {
            rpc_url,
            program_id,
            payer_keypair,
        })
    }
}
