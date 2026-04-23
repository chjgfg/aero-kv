use crate::{
    error::Error, raft::{raft::Node, types::{Command, Envelope, Message, RaftTask}}, storage::engine::DiskClient
};

use crossbeam::channel::{Receiver, select};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Instant,
};
use tokio::sync::oneshot;

pub struct Server {
    node: Node,                  // 你的 Raft 状态机节点
    node_rx: Receiver<Envelope>, // Raft 内部输出通道
    // 核心：暂存正在等待共识结果的 HTTP 请求
    pending_responses: HashMap<u64, oneshot::Sender<Result<Vec<u8>, Error>>>,
    pub storage: Arc<RwLock<DiskClient>>,
}


impl Server {
    pub fn new(id: u64, peers: Vec<u64>, storage: Arc<RwLock<DiskClient>>) -> Self {
        // 1. 创建一个通道，用于接收来自 Raft 内部的消息
        let (node_tx, node_rx) = crossbeam::channel::unbounded();

        Self {
            // 2. 初始化 Node，并把 node_tx 给它（假设 Node 需要它发消息）
            // 如果你的 Node::new 目前还不支持传入 tx，你需要修改 Node::new
            node: Node::new(id, peers, node_tx),
            // 3. 把 node_rx 存进 Server，用于 loop 里的 recv(self.node_rx)
            node_rx,
            pending_responses: HashMap::new(),
            storage, // 👈 确保这里正确传入
        }
    }

    // 💡 解决 forward_to_peers 报错
    // 你需要补上这个方法，逻辑是：把消息发给真正的 Peer（其他节点）
    // server.rs 里的 forward_to_peers 实现
    fn forward_to_peers(&self, envelope: Envelope) {
        let target_id = envelope.to;
        let target_port = 3000 + target_id; // 简单约定
        let url = format!("http://localhost:{}/raft/message", target_port);

        // 使用 reqwest 异步发送 (注意：这里在同步线程，可以用 blocking 客户端)
        std::thread::spawn(move || {
            let client = reqwest::blocking::Client::new();
            let _ = client.post(url).json(&envelope).send();
        });
    }

    fn handle_envelopes(&mut self, envelopes: Vec<Envelope>) {
        for envelope in envelopes {
            match &envelope.message {
                // 情况 A：Leader 确认共识达成，产生响应
                // 此时 Leader 会进入这里并更新本地存储
                Message::ClientResponse { id: _, response } => {
                    if let Ok(data) = response {
                        if let Ok(cmd) = serde_json::from_slice::<Command>(data) {
                            let k = cmd.key.as_bytes();
                            let mut store = self.storage.write().unwrap();
                            match cmd.op.as_str() {
                                "upsert" => {
                                    // 🚀 这里是关键：打印出每个字节的十进制值
                                    println!("📍 Node {} WRITING KEY: {:?} (len: {})", self.node.id, k, k.len());
                                    let Some(value) = cmd.value else {
                                        println!("⚠️ Warn: SET operation missing value for key: {}", cmd.key);
                                        return;
                                    };
                                    store.set(k.to_vec(), value.as_bytes().to_vec()).expect("Storage set failed");
                                    println!("💾 [Leader Apply] Node {}: {} = {}", self.node.id, cmd.key, value);
                                }
                                "delete" => {
                                    store.delete(cmd.key.as_bytes().to_vec()).ok(); // 👈 调用 memory.rs 的删除逻辑
                                    println!("🗑️ [Node {}] Deleted key: {}", self.node.id, cmd.key);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // 情况 B：Follower 收到来自 Leader 的日志同步（Append）
                // 此时 Follower 必须解析日志并更新本地存储
                Message::Append { entries, .. } => {
                    for entry in entries {
                        // 如果 entry 中有 command（即指令），则应用到存储
                        if let Some(command) = &entry.command {
                            if let Ok(req) = serde_json::from_slice::<Command>(command) {
                                let k = req.key.as_bytes();
                                // 🚀 这里是关键：打印出每个字节的十进制值
                                println!("📍 Node {} WRITING KEY: {:?} (len: {})", self.node.id, k, k.len());
                                let mut store = self.storage.write().unwrap();
                                let Some(value) = req.value else {
                                    return;
                                };
                                store.set(k.to_vec(), value.as_bytes().to_vec()).expect("Storage set failed");
                                println!("🔥 REAL WRITE: key={}, val={}", req.key, value);
                                println!("💾 [Follower Apply] Node {}: {} = {}", self.node.id, req.key, value);
                                // 强行读取确认
                                let check = store.get(req.key.as_bytes().to_vec()).ok();
                                println!("🔍 Instant Check: {:?}", check);
                            } else {
                                // 💡 如果解析失败，这里会报错
                                println!("❌ FAILED TO PARSE COMMAND");
                            }
                        }
                    }
                }
                _ => {}
            }

            // --- 原有的分发逻辑 (保持不变) ---
            if envelope.to == self.node.id {
                if let Message::ClientResponse { id, response } = envelope.message {
                    if let Some(tx) = self.pending_responses.remove(&id) {
                        let _ = tx.send(response);
                        println!("✅ Sent response to HTTP for request {}", id);
                    }
                }
            } else {
                self.forward_to_peers(envelope);
            }
        }
    }

    pub fn run(
        mut self,
        ticker: Receiver<Instant>,
        peers_rx: Receiver<Envelope>,
        task_rx: Receiver<RaftTask>,
    ) {
        loop {
            select! {
                // 1. 处理来自 HTTP 接口的任务 (Leader 写入入口)
                recv(task_rx) -> result => {
                    if let Ok(task) = result {
                        self.pending_responses.insert(task.id, task.response_tx);
                        let envelopes = self.node.step(task.message, self.node.id).expect("Step failed");
                        self.handle_envelopes(envelopes);
                    }
                }

                // 2. 处理来自队友的消息 (Follower 同步入口)
                recv(peers_rx) -> result => {
                    if let Ok(envelope) = result {
                        // 🚀 重点：收到的 envelope 也要过一遍 handle_envelopes
                        // 这样 handle_envelopes 里的 Message::Append 分支才会触发写入存储！
                        self.handle_envelopes(vec![envelope.clone()]);

                        // 然后再让协议栈处理逻辑（比如回复确认包）
                        let envelopes = self.node.step(envelope.message, envelope.from).expect("Step failed");
                        self.handle_envelopes(envelopes);
                    }
                }

                // 3. 处理时钟信号 (心跳/选举触发)
                recv(ticker) -> _ => {
                    if let Ok(envelopes) = self.node.tick() {
                        self.handle_envelopes(envelopes);
                    }
                }

                // 4. 处理节点内部通知
                recv(self.node_rx) -> result => {
                    if let Ok(envelope) = result {
                        self.handle_envelopes(vec![envelope]);
                    }
                }
            }
        }
    }
}
