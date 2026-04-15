// # Anchor 客户端：连接链、发送交易

use std::str::FromStr;

use crate::{
    config::Config,
    error::{Error, Result},
};
use anchor_client::{Client, Cluster};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair},
};

use std::sync::Arc; // 引入 Arc

pub struct ChainClient {
    // 这里的 C 现在是 Arc<Keypair>
    pub client: Client<Arc<Keypair>>,
    pub program_id: Pubkey,
    pub payer: Arc<Keypair>,
}
impl ChainClient {
    pub fn new(config: &Config) -> Result<Self> {
        // 1. 创建拥有所有权的 Keypair，并直接塞进 Arc 里
        let payer = Arc::new(Keypair::from_base58_string(config.payer_keypair.as_str()));

        // 2. 这里的 .clone() 只是拷贝了智能指针，符合 Client<C: Clone> 的要求
        let payer_for_client = payer.clone();

        let cluster = Cluster::Custom(config.rpc_url.clone(), config.rpc_url.clone());

        // 3. 此时 payer_for_client 类型是 Arc<Keypair>
        // 它满足 Clone 且满足 Deref<Target = Keypair> (Keypair 实现了 Signer)
        let client =
            Client::new_with_options(cluster, payer_for_client, CommitmentConfig::confirmed());

        let program_id = Pubkey::from_str(&config.program_id)
            .map_err(|e| Error::InvalidProgramId(format!("{}", e)))?;

        Ok(Self {
            client,
            program_id,
            payer,
        })
    }


}
