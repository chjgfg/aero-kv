use crate::{
    error::Error,
    raft::types::{Entry, Envelope, Message, Role},
};

/// 最核心的 Raft 节点结构
pub struct Node {
    pub id: u64,
    pub term: u64,
    pub log_index: u64,
    pub log_term: u64,
    pub role: Role,
    pub peers: Vec<u64>,
    _node_tx: crossbeam::channel::Sender<Envelope>,
}

impl Node {
    pub fn new(id: u64, peers: Vec<u64>, node_tx: crossbeam::channel::Sender<Envelope>) -> Self {
        // 如果没有队友，直接当 Leader
        let role = if peers.is_empty() {
            println!("🚀 Single node detected. Node {} starting as Leader.", id);
            Role::Leader {
                _heartbeat_ticks: 0,
            }
        } else {
            println!("⏳ Cluster mode. Node {} starting as Follower.", id);
            Role::Follower {
                leader: None,
                voted_for: None,
            }
        };

        Self {
            id,
            term: 0,
            log_index: 0,
            log_term: 0,
            role,
            peers,
            _node_tx: node_tx,
        }
    }

    pub fn step(&mut self, msg: Message, from_id: u64) -> Result<Vec<Envelope>, Error> {
        // ✅ 修复点：定义 msg_term
        let msg_term = msg.term();
        // 只要消息里的 term 比我大，我无条件变回 Follower 并更新 term
        if msg_term > self.term {
            println!("Node {} term (current:{}) < msg_term ({}), stepping down", self.id, self.term, msg_term);
            self.term = msg_term;
            self.role = Role::Follower {
                leader: None,
                voted_for: None,
            };
            println!("Node {} term updated to {}, stepping down to Follower", self.id, self.term);
        }

        let mut responses = Vec::new();

        match &mut self.role {
            // --- Follower 逻辑 ---
            Role::Follower { leader, voted_for } => {
                match msg {
                    // 如果有人拉票
                    Message::Campaign { last_index, .. } => {
                        // 简化版：只要我没投过票，就投给你
                        if voted_for.is_none() {
                            *voted_for = Some(last_index); // 记录投给了谁
                            responses.push(Envelope {
                                from: self.id,
                                to: last_index, // 回复给候选人
                                term: self.term,
                                message: Message::CampaignResponse {
                                    vote: true,
                                    term: self.term,
                                },
                            });
                        }
                    }
                    // 收到 ClientRequest，转发给 Leader
                    Message::ClientRequest { id, request } => {
                        if let Some(leader_id) = *leader {
                            // 🚀 转发：我不是 Leader，但我知道谁是，我帮你发给它
                            println!("Node {} forwarding request {} to Leader {}", self.id, id, leader_id);
                            responses.push(Envelope {
                                from: self.id,
                                to: leader_id,
                                term: self.term,
                                message: Message::ClientRequest { id, request },
                            });
                        }
                    }
                    // 假设后续我们会加 Append 消息作为心跳
                    Message::Append { base_term, .. } => {
                        *leader = Some(from_id); // 认老大，这样就不会再触发选举了
                        // 确保 term 同步，防止 Follower 因为 term 滞后而意外发起选举
                        if base_term >= self.term {
                            self.term = base_term;
                        }
                    }
                    _ => {}
                }
            }

            Role::Leader { .. } => {
                match msg {
                    // ✅ 收到客户端请求：
                    Message::ClientRequest { id, request } => {
                        // 1. 给自己发回执 (为了让本地 Leader 写入内存并回复 HTTP)
                        responses.push(Envelope {
                            from: self.id,
                            to: from_id, // 🚀 关键修改：发还给发送者，实现响应闭环
                            term: self.term,
                            message: Message::ClientResponse {
                                id,
                                response: Ok(request.clone()),
                            },
                        });

                        // 2. 🚀 关键：广播给所有 Follower，这样从节点才能同步数据
                        for peer in &self.peers {
                            responses.push(Envelope {
                                from: self.id,
                                to: *peer,
                                term: self.term,
                                message: Message::Append {
                                    base_index: self.log_index,
                                    base_term: self.log_term,
                                    entries: vec![Entry {
                                        index: self.log_index + 1,
                                        term: self.term,
                                        command: Some(request.clone()), // 👈 这里一定要带上原始请求数据
                                    }],
                                },
                            });
                        }
                    }

                    // 🚨 收到拉票，退位逻辑保持不变
                    Message::Campaign { .. } => {
                        println!("Node {} received campaign, stepping down.", self.id);
                        self.role = Role::Follower {
                            leader: None,
                            voted_for: None,
                        };
                    }

                    _ => {}
                }
            }

            // 修改 Candidate 逻辑部分
            Role::Candidate { votes } => {
                if let Message::CampaignResponse { vote: true, term: _, } = msg {
                    // ❌ 不要只写 self.id
                    // ✅ 这里需要知道是谁回的消息。
                    // 为了方便，我们在 step 函数里增加一个参数，或者通过 Envelope 传进来。
                    // 假设我们把 step 改为 step(&mut self, msg: Message, from_id: u64)

                    votes.insert(from_id); // 记录实际投票人的 ID
                    votes.insert(self.id); // 别忘了自己那一票

                    println!("Node {} got vote from {}, total votes: {}", self.id, from_id, votes.len());

                    if votes.len() >= 2 {
                        self.role = Role::Leader {
                            _heartbeat_ticks: 0,
                        };
                        println!("🏆 Node {} became Leader!", self.id);
                    }
                }
            }
        }
        Ok(responses)
    }

    pub fn tick(&mut self) -> Result<Vec<Envelope>, Error> {
        let mut responses = Vec::new();

        match &mut self.role {
            // --- Leader 发送心跳 ---
            Role::Leader { .. } => {
                // println!("Node {} (Leader) sending heartbeats...", self.id);
                for peer in &self.peers {
                    responses.push(Envelope {
                        from: self.id,
                        to: *peer,
                        term: self.term,
                        message: Message::Append {
                            base_index: self.log_index,
                            base_term: self.log_term,
                            entries: vec![], // 空 entries 代表心跳
                        },
                    });
                }
            }

            // --- Follower 竞选计时 ---
            Role::Follower { leader, .. } => {
                // 💡 只有没有 Leader 时才竞选
                if leader.is_none() {
                    // 降低选举频率：20个tick才可能有一次选举
                    if rand::random::<u8>() % 20 == 0 {
                        self.term += 1;
                        self.role = Role::Candidate {
                            votes: std::collections::HashSet::new(),
                        };
                        println!("Node {} timeout! Campaigning for Term {}...", self.id, self.term);
                        // ... 这里保持你之前的拉票逻辑 (Campaign) ...
                        for peer in &self.peers {
                            responses.push(Envelope {
                                from: self.id,
                                to: *peer,
                                term: self.term,
                                message: Message::Campaign {
                                    last_index: self.id,
                                    last_term: self.term,
                                },
                            });
                        }
                    }
                } else {
                    // 💡 如果有 Leader，每个 tick 都要重置 Leader 状态，模拟心跳过期
                    // 简单处理：每个 tick 都把 leader 设为 None，除非下一秒收到 Append 消息
                    *leader = None;
                }
            }
            _ => {}
        }

        Ok(responses)
    }
}
