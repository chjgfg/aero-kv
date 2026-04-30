use openraft::error::{InstallSnapshotError, NetworkError, RaftError, RPCError};
use openraft::network::{RaftNetwork, RaftNetworkFactory, RPCOption};
use openraft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest,
    InstallSnapshotResponse, VoteRequest, VoteResponse,
};
use openraft::BasicNode;
use reqwest::Client;

use crate::raft::types::{NodeId, RaftConfig};

pub struct Network {}

// 添加下面这段代码
impl Network {
    pub fn new() -> Self {
        Self {}
    }
}

// 1. 移除 #[async_trait]，改用原生的实现方式
impl RaftNetworkFactory<RaftConfig> for Network {
    type Network = NetworkConnection;

    // 注意：这里没有 #[async_trait] 宏，直接写 async fn
    async fn new_client(&mut self, target: NodeId, node: &BasicNode) -> Self::Network {
        NetworkConnection {
            _target: target,
            addr: node.addr.clone(),
            client: Client::new(),
        }
    }
}

pub struct NetworkConnection {
    _target: NodeId,
    addr: String,
    client: Client,
}

// 2. 同样移除这里的 #[async_trait]
impl RaftNetwork<RaftConfig> for NetworkConnection {
    async fn append_entries(
        &mut self,
        req: AppendEntriesRequest<RaftConfig>,
        _option: RPCOption,
    ) -> Result<AppendEntriesResponse<NodeId>, RPCError<NodeId, BasicNode, RaftError<NodeId>>> {
        let url = format!("http://{}/raft/append", self.addr);
        let resp = self.client.post(&url).json(&req).send().await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        let res = resp.json::<AppendEntriesResponse<NodeId>>().await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;
        Ok(res)
    }

    async fn install_snapshot(
        &mut self,
        req: InstallSnapshotRequest<RaftConfig>,
        _option: RPCOption,
    ) -> Result<
        InstallSnapshotResponse<NodeId>, 
        RPCError<NodeId, BasicNode, RaftError<NodeId, InstallSnapshotError>> // 关键修改点：外层嵌套 RaftError
    > {
        let url = format!("http://{}/raft/snapshot", self.addr);
        let resp = self.client.post(&url).json(&req).send().await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        let res = resp.json::<InstallSnapshotResponse<NodeId>>().await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;
        Ok(res)
    }

    async fn vote(
        &mut self,
        req: VoteRequest<NodeId>,
        _option: RPCOption,
    ) -> Result<VoteResponse<NodeId>, RPCError<NodeId, BasicNode, RaftError<NodeId>>> {
        let url = format!("http://{}/raft/vote", self.addr);
        let resp = self.client.post(&url).json(&req).send().await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;

        let res = resp.json::<VoteResponse<NodeId>>().await
            .map_err(|e| RPCError::Network(NetworkError::new(&e)))?;
        Ok(res)
    }
}