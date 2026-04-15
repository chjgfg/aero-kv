// # Anchor 客户端：连接链、发送交易

use std::str::FromStr;

use crate::{
    config::Config,
    error::{Error, Result},
};
use anchor_client::{Client, Cluster, Program};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

pub struct ChainClient<'a> {
    pub client: Client<'a, Keypair>,
    pub program_id: Pubkey,
    pub payer: &'a Keypair,
}

impl ChainClient {
    pub fn new(config: &Config) -> Result<Self> {
        // 1. 解析 payer 私钥
        let payer = Keypair::from_base58_string(config.payer_keypair.as_str());

        // 2. Cluster 从 RPC URL 创建
        let cluster = Cluster::Custom(config.rpc_url.clone(), config.rpc_url.clone());

        // 3. 【核心修复：删掉类型注解！】
        // 原来的错误代码：let client: Client<&Keypair> = ...
        // 现在直接不写类型，让 Rust 自动推成 Client<Keypair>
        let client = Client::new_with_options(cluster, &payer, CommitmentConfig::confirmed());

        // 4. 解析程序 ID
        let program_id = Pubkey::from_str(&config.program_id)
            .map_err(|e| Error::InvalidProgramId(format!("解析程序ID失败: {}", e)))?;

        // 5. 类型完全匹配，直接返回
        Ok(Self {
            client,
            program_id,
            payer,
        })
    }

    // 获取 Program 实例
    pub fn program(&self) -> Program<Keypair> {
        self.client.program(self.program_id)
    }

    // 通用 PDA 计算
    pub fn find_pda(&self, seeds: &[&[u8]]) -> (Pubkey, u8) {
        Pubkey::find_program_address(seeds, &self.program_id)
    }
}
