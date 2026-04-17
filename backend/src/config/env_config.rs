// # 全局配置：RPC、程序ID、PDA种子、钱包

use crate::error::{Error, Result};
use dotenv::dotenv;
// use log::info;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub payer_keypair: String,
    pub rpc_url: String,
    pub program_id: String,
    pub server_port: String,
    pub treasury: String,
}

impl Config {
    pub fn from_env() -> Result<Config> {
        dotenv().ok();
        let rpc_url: String =
            env::var("RPC_URL").map_err(|e| Error::RpcUrlError(format!("RPC 路径错误: {}", e)))?;
        // info!("rpc_url: {}", rpc_url);
        let program_id = env::var("PROGRAM_ID")
            .map_err(|e| Error::ProgramIdError(format!("Program Id 错误: {}", e)))?;
        // info!("program_id: {}", program_id);
        let payer_keypair = env::var("PAYER_KEYPAIR")
            .map_err(|e| Error::PayerKeypairError(format!("Payer Keypair 错误: {}", e)))?;
        // info!("payer_keypair: {}", payer_keypair);
        let server_port = env::var("SERVER_PORT")
        .map_err(|e| Error::ServerPortError(format!("Program Id 错误: {}", e)))?;
        // info!("payer_keypair: {}", payer_keypair);
        let treasury = env::var("TREASURY")
            .map_err(|e| Error::TreasuryError(format!("treasury 错误: {}", e)))?;

        Ok(Self {
            rpc_url,
            program_id,
            payer_keypair,
            server_port: server_port,
            treasury,
        })
    }
}
