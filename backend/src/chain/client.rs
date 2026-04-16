// # Anchor 客户端：连接链、发送交易

use std::str::FromStr;

use crate::{
    config::env_config::Config,
    error::{Error, Result},
    utils::parse_keypair_array,
};
use anchor_client::{Client, Cluster, Program};
use log::info;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair};

use std::sync::Arc; // 引入 Arc

pub struct ChainClient {
    // 这里的 C 现在是 Arc<Keypair>, 只写个 Client<Keypair> 会报错
    pub client: Client<Arc<Keypair>>,
    pub rpc_client: Arc<RpcClient>, // 🔥 关键：新增 RpcClient 字段
    pub program_id: Pubkey,
    pub payer: Arc<Keypair>,
}

impl ChainClient {
    pub fn new(config: &Config) -> Result<Self> {
        // 1. 创建拥有所有权的 Keypair，并直接塞进 Arc 里
        // let payer = Arc::new(Keypair::from_base58_string(config.payer_keypair.as_str()));
        let key_bytes = parse_keypair_array(config.payer_keypair.as_str())?;
        let payer = Arc::new(Keypair::from_bytes(&key_bytes).map_err(|_| Error::InvalidKey)?); // 用 from_bytes 解析数组
        // info!("payer: {:?}", payer);
        // 2. 这里的 .clone() 只是拷贝了智能指针，符合 Client<C: Clone> 的要求
        let payer_for_client = payer.clone();
        let cluster = Cluster::Custom(config.rpc_url.clone(), config.rpc_url.clone());
        // 3. 此时 payer_for_client 类型是 Arc<Keypair>
        // 它满足 Clone 且满足 Deref<Target = Keypair> (Keypair 实现了 Signer)
        let client =
            Client::new_with_options(cluster, payer_for_client, CommitmentConfig::confirmed());
        let program_id = Pubkey::from_str(&config.program_id)
            .map_err(|e| Error::InvalidProgramId(format!("{}", e)))?;
        // 2. 创建 RpcClient（同步，可在 spawn_blocking 中使用）
        let rpc_client = Arc::new(RpcClient::new(config.rpc_url.clone()));
        info!("program_id: {}", program_id);
        Ok(Self {
            client,
            program_id,
            payer,
            rpc_client,
        })
    }

    /// 和 Solana 合约对话的工具
    /// 你要发交易、读数据、调用合约，必须用这个工具
    /// 这个函数就是给你提供这个工具
    pub fn program(&self) -> Result<Program<Arc<Keypair>>> {
        self.client
            .program(self.program_id)
            .map_err(|e| Error::RpcError(format!("获取 program 失败: {e}")))
    }

    /// 通用 PDA 计算
    pub fn find_pda(&self, seeds: &[&[u8]]) -> (Pubkey, u8) {
        Pubkey::find_program_address(seeds, &self.program_id)
    }
}
