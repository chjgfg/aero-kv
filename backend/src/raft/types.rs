use openraft::{BasicNode, declare_raft_types};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::auth::types::Action;

pub type NodeId = u64;

// 你的 KV 写操作请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KvOp {
    Upsert {
        key: String,
        value: String,
        pda: Vec<u8>, // 新增：由 API 层计算好传进来
    },
    Delete {
        key: String,
    },
    // --- 新增变体 ---
    SetPause {
        paused: bool,
    },
    SetFee {
        base_fee: u64,
        fee_per_byte: u64,
        scan_fee_per_item: u64,
    },
    SyncLogin {
        pubkey: String,
        permissions: Vec<Action>,
    },  
    SyncLogout {
        pubkey: String,
    }, // 同步登出状态
    SyncGrant {
        user_pubkey: String,
        permissions: Vec<Action>,
    }, // 同步授权变更
}

// 写操作的响应（可选）
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KvResponse {
    pub value: Option<String>,
}

// 快照数据类型：由于 Bitcask 是字节流，用 Cursor<Vec<u8>> 最方便
pub type SnapshotData = Cursor<Vec<u8>>;

declare_raft_types!(
    pub RaftConfig:
        D = KvOp,
        R = (), // 响应暂时设为空
        NodeId = NodeId,
        Node = BasicNode,
        Entry = openraft::Entry<RaftConfig>
);
