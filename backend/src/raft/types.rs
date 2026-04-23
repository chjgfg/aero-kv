use std::sync::{Arc, Mutex, RwLock};

use crossbeam::channel::Sender;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::{block_chain::client::ChainClient, error::Error, storage::engine::DiskClient};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub op: String, // "SET", "DELETE"
    pub key: String,
    pub value: Option<String>, // DELETE 时为 None
}

#[derive(Deserialize, Debug)]
pub struct GetParams {
    pub key: String,
}

// 全局上下文
pub struct AppContext {
    pub node_id: u64,
    pub task_sender: Sender<RaftTask>,
    pub peers_sender: Sender<Envelope>, // 用于接收外部 Raft 消息
    pub kv_store: Arc<RwLock<DiskClient>>,
    pub chain: Arc<ChainClient>,
    pub storage: Arc<Mutex<DiskClient>>,
}

/// 发送到 Raft 线程的任务包装
#[derive(Debug)]
pub struct RaftTask {
    /// 这里的 ID 必须和 Message::ClientRequest 里的 ID 一致
    pub id: u64,
    /// 包含具体的指令（Set/Get 等）
    pub message: Message,
    /// 用于将处理结果从 Raft 线程传回 HTTP 线程
    pub response_tx: oneshot::Sender<Result<Vec<u8>, Error>>,
}

/// 所有的网络消息包装
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    pub from: u64,
    pub to: u64,
    pub term: u64,
    pub message: Message,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Message {
    /// 拉票请求
    Campaign { last_index: u64, last_term: u64 },
    /// 投票响应
    CampaignResponse {
        term: u64, // 💡 补上 term
        vote: bool,
    },
    /// 日志追加/心跳
    Append {
        base_index: u64,
        base_term: u64,
        entries: Vec<Entry>,
    },
    /// 日志响应
    AppendResponse {
        term: u64, // 💡 补上 term
        last_index: u64,
        success: bool,
    },
    /// 客户端请求
    ClientRequest { id: u64, request: Vec<u8> },
    /// 客户端响应
    ClientResponse {
        id: u64,
        response: Result<Vec<u8>, Error>,
    },
}

impl Message {
    pub fn term(&self) -> u64 {
        match self {
            Message::Campaign { last_term, .. } => *last_term,
            Message::CampaignResponse { term, .. } => *term,
            Message::Append { base_term, .. } => *base_term,
            Message::AppendResponse { term, .. } => *term,
            // 客户端请求/响应不携带集群任期
            Message::ClientRequest { .. } => 0,
            Message::ClientResponse { .. } => 0,
        }
    }
}

/// 日志条目
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub index: u64,
    pub term: u64,
    pub command: Option<Vec<u8>>,
}

/// Raft 节点状态
pub enum Role {
    Follower {
        leader: Option<u64>,
        voted_for: Option<u64>,
    },
    Candidate {
        votes: std::collections::HashSet<u64>,
    },
    Leader {
        _heartbeat_ticks: u64,
    },
}
