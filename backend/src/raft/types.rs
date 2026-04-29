use openraft::{BasicNode, declare_raft_types};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

pub type NodeId = u64;

// 你的 KV 写操作请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KvOp {
    Upsert {
        key: String,
        value: String,
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
}

// 写操作的响应（可选）
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
