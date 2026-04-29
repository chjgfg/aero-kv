use crate::raft::to_storage_error;
use crate::raft::types::RaftConfig;
use crate::storage::engine::DiskClient;
use openraft::storage::{LogState, RaftLogReader, RaftLogStorage};
use openraft::{LogId, OptionalSend, Vote};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MyLogStorage {
    pub db: Arc<Mutex<DiskClient>>,
}

// 必须实现 Reader (解决 E0277)
// 必须实现 Reader (解决 E0277)
impl RaftLogReader<RaftConfig> for MyLogStorage {
    async fn try_get_log_entries<R>(
        &mut self,
        range: R,
    ) -> Result<Vec<openraft::Entry<RaftConfig>>, openraft::StorageError<u64>>
    where
        R: std::ops::RangeBounds<u64> + Send,
    {
        let mut db = self.db.lock().await;
        let mut entries = Vec::new();

        // 这里的逻辑搬运你之前的扫描代码
        for item in db.scan(..) {
            if let Ok((_k, v)) = item {
                if let Ok(entry) = serde_json::from_slice::<openraft::Entry<RaftConfig>>(&v) {
                    if range.contains(&entry.log_id.index) {
                        entries.push(entry);
                    }
                }
            }
        }
        Ok(entries)
    }
}

impl RaftLogStorage<RaftConfig> for MyLogStorage {
    type LogReader = Self;

    async fn get_log_state(&mut self) -> Result<LogState<RaftConfig>, openraft::StorageError<u64>> {
        Ok(LogState {
            last_purged_log_id: None,
            last_log_id: None,
        })
    }

    // --- 投票相关持久化 ---
    async fn save_vote(&mut self, vote: &Vote<u64>) -> Result<(), openraft::StorageError<u64>> {
        let mut db = self.db.lock().await;
        // 使用一个固定的 Key 存储当前节点的投票信息
        let val = serde_json::to_vec(vote).map_err(to_storage_error)?;
        db.set(b"raft_vote".to_vec(), val)
            .map_err(to_storage_error)?;
        Ok(())
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<u64>>, openraft::StorageError<u64>> {
        let mut db = self.db.lock().await;
        let val = db.get(b"raft_vote".to_vec()).map_err(to_storage_error)?;
        match val {
            Some(v) => Ok(Some(serde_json::from_slice(&v).map_err(to_storage_error)?)),
            None => Ok(None),
        }
    }

    // --- 日志追加 (最关键) ---
    async fn append<I>(
        &mut self,
        entries: I,
        callback: openraft::storage::LogFlushed<RaftConfig>,
    ) -> Result<(), openraft::StorageError<u64>>
    where
        I: IntoIterator<Item = openraft::Entry<RaftConfig>> + OptionalSend,
        I::IntoIter: OptionalSend,
    {
        let mut db = self.db.lock().await; // 配合 Arc<Mutex<DiskClient>>
        for entry in entries {
            // Key 设计：log_{index}，由于 Bitcask scan 是按顺序的，大端序数字字符串比较稳妥
            let key = format!("log_{:020}", entry.log_id.index).into_bytes();
            let val = serde_json::to_vec(&entry).map_err(to_storage_error)?;
            db.set(key, val).map_err(to_storage_error)?;
        }

        // 告诉 Raft 引擎日志已经成功刷盘
        callback.log_io_completed(Ok(()));
        Ok(())
    }

    async fn truncate(&mut self, _log_id: LogId<u64>) -> Result<(), openraft::StorageError<u64>> {
        Ok(())
    }
    async fn purge(&mut self, _log_id: LogId<u64>) -> Result<(), openraft::StorageError<u64>> {
        Ok(())
    }
    async fn get_log_reader(&mut self) -> Self::LogReader {
        MyLogStorage {
            db: self.db.clone(),
        }
    }
}
