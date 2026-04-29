use openraft::storage::{RaftSnapshotBuilder, RaftStateMachine, Snapshot};
use openraft::{LogId, SnapshotMeta, StoredMembership};
use rs_merkle::MerkleTree;
use rs_merkle::algorithms::Sha256;
use std::io::Cursor;
use std::sync::Arc;

use crate::raft::to_storage_error;
use crate::raft::types::{KvOp, RaftConfig, SnapshotData};
use crate::storage::engine::DiskClient;
use crate::utils;

pub struct MyStateMachine {
    pub db: Arc<tokio::sync::Mutex<DiskClient>>,
    // 新增这两个字段，确保状态机能直接操作它们
    pub merkle_tree: Arc<tokio::sync::Mutex<MerkleTree<Sha256>>>,
    pub leaf_hashes: Arc<tokio::sync::Mutex<Vec<[u8; 32]>>>,
}

// 必须先实现这个 Trait (解决 E0277)
impl RaftSnapshotBuilder<RaftConfig> for MyStateMachine {
    async fn build_snapshot(
        &mut self,
    ) -> Result<Snapshot<RaftConfig>, openraft::StorageError<u64>> {
        // 这里获取锁的方式改成 Mutex 的方式
        let mut db = self.db.lock().await;
        let mut data = Vec::new();
        // 这里的 item 是 Result<(Vec<u8>, Vec<u8>), Error>
        for item in db.scan(..) {
            match item {
                Ok((k, v)) => {
                    // 只有成功拿到数据才 push
                    data.push((k, v));
                }
                Err(e) => {
                    // 如果扫描出错，直接返回 StorageError 终止快照构建
                    return Err(to_storage_error(e));
                }
            }
        }

        Ok(Snapshot {
            meta: SnapshotMeta {
                last_log_id: None, // 实际应从 DB 读取
                last_membership: StoredMembership::default(),
                snapshot_id: "snap_1".into(),
            },
            snapshot: Box::new(Cursor::new(serde_json::to_vec(&data).unwrap())),
        })
    }
}

impl RaftStateMachine<RaftConfig> for MyStateMachine {
    type SnapshotBuilder = Self;

    async fn applied_state(
        &mut self,
    ) -> Result<
        (
            Option<LogId<u64>>,
            StoredMembership<u64, openraft::BasicNode>,
        ),
        openraft::StorageError<u64>,
    > {
        Ok((None, StoredMembership::default()))
    }

    async fn apply<I>(&mut self, entries: I) -> Result<Vec<()>, openraft::StorageError<u64>>
    where
        I: IntoIterator<Item = openraft::Entry<RaftConfig>> + Send,
    {
        // 这里获取锁的方式改成 Mutex 的方式
        let mut db = self.db.lock().await;
        let mut res = Vec::new();
        let mut need_rebuild_tree = false; // 标记是否需要重建树
        for entry in entries {
            if let openraft::EntryPayload::Normal(op) = entry.payload {
                match op {
                    KvOp::Upsert { key, value } => {
                        let _ = db.set(key.into_bytes(), value.into_bytes());
                        need_rebuild_tree = true; // 只要有写入，就标记需要重建
                    }
                    KvOp::Delete { key } => {
                        let _ = db.delete(key.into_bytes());
                        need_rebuild_tree = true; // 只要有写入，就标记需要重建
                    }
                    // --- 处理新变体，将其持久化到本地存储 ---[cite: 1]
                    KvOp::SetPause { paused } => {
                        let _ = db.set(b"sys_paused".to_vec(), vec![paused as u8]);
                    }
                    KvOp::SetFee { base_fee, .. } => {
                        // 这里可以根据需要存储完整的手续费结构体
                        let val = serde_json::to_vec(&base_fee).unwrap();
                        let _ = db.set(b"sys_base_fee".to_vec(), val);
                    }
                }
            }
            res.push(());
        }
        // --- 核心修复：在所有节点同步完成后重建树 ---
        if need_rebuild_tree {
            // 注意：你需要确保 state 能够在这里被访问到，
            // 或者直接在这里实现类似的扫描 db 并重新计算 leaf_hashes 的逻辑。
            // 这样无论是在 Leader 还是 Follower，内存里的树都会同步更新。
            drop(db); // 先释放 db 锁，防止 calc_merkle_root 内部死锁
            // 调用你现有的重建逻辑
            // 传入三个 Arc 引用
            let _ = utils::rebuild_merkle_tree(
                &self.db.clone(),
                &self.merkle_tree.clone(),
                &self.leaf_hashes.clone(),
            ).await;
        }
        Ok(res)
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        MyStateMachine {
            db: self.db.clone(),
            merkle_tree: self.merkle_tree.clone(),
            leaf_hashes: self.leaf_hashes.clone(),
        }
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<SnapshotData>, openraft::StorageError<u64>> {
        Ok(Box::new(Cursor::new(Vec::new())))
    }

    async fn install_snapshot(
        &mut self,
        _meta: &SnapshotMeta<u64, openraft::BasicNode>,
        _snapshot: Box<SnapshotData>,
    ) -> Result<(), openraft::StorageError<u64>> {
        Ok(())
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<RaftConfig>>, openraft::StorageError<u64>> {
        let mut builder = self.get_snapshot_builder().await;
        builder.build_snapshot().await.map(Some)
    }
}
