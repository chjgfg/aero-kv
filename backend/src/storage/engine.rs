use crate::{
    error::Result,
    storage::{
        iter::DiskIterator,
        storage::{KeyDir, Log},
    },
};
use std::{collections::Bound, fs, ops::RangeBounds, path::PathBuf};

pub struct DiskClient {
    keydir: KeyDir,
    log: Log,
}

impl DiskClient {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let mut log = Log::new(file_path);
        let keydir = log.build_index()?;
        Ok(Self { keydir, log })
    }

    pub fn compact(file_path: PathBuf) -> Result<Self> {
        let mut eng = Self::new(file_path)?;
        eng.new_compact()?;
        Ok(eng)
    }

    pub fn new_compact(&mut self) -> Result<()> {
        let mut new_path = self.log.file_path.clone();
        new_path.set_extension(".compact");
        let mut new_log = Log::new(new_path);
        let mut new_key_dir = KeyDir::new();
        for (key, (offset, value_size)) in self.keydir.iter() {
            let value = self.log.read_value(*offset, *value_size)?;
            let (new_offset, new_size) = new_log.write_entry(key, Some(&value))?;
            new_key_dir.insert(
                key.clone(),
                (
                    new_offset + new_size as u64 - *value_size as u64,
                    *value_size,
                ),
            );
        }
        fs::rename(new_log.file_path, &self.log.file_path).unwrap(); // compact_log.file_path 变成 self.log.file_path
        new_log.file_path = self.log.file_path.clone();
        self.log = new_log;
        self.keydir = new_key_dir;
        Ok(())
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let (offset, size) = self.log.write_entry(&key, Some(&value))?;
        let value_size = value.len() as u32;
        self.keydir
            .insert(key, (offset + size as u64 - value_size as u64, value_size));
        Ok(())
    }

    pub fn get(&mut self, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        match self.keydir.get(&key) {
            Some((offset, val_size)) => {
                let val = self.log.read_value(*offset, *val_size)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    pub fn delete(&mut self, key: Vec<u8>) -> Result<()> {
        self.log.write_entry(&key, None)?;
        self.keydir.remove(&key);
        Ok(())
    }

    pub fn scan(&mut self, range: impl RangeBounds<Vec<u8>>) -> DiskIterator<'_> {
        DiskIterator {
            inner: self.keydir.range(range),
            log: &mut self.log,
        }
    }

    pub fn scan_prefix(&mut self, prefix: Vec<u8>) -> DiskIterator<'_> {
        let start = Bound::Included(prefix.clone());
        let mut bound_prefix = prefix.clone();
        let end = match bound_prefix.iter().rposition(|b| *b != 255) {
            Some(pos) => {
                bound_prefix[pos] += 1;
                bound_prefix.truncate(pos + 1); // 从255开始向后丢弃
                Bound::Excluded(bound_prefix)
            }
            None => Bound::Unbounded,
        };
        self.scan((start, end))
    }

    pub fn count(&self) -> Result<usize> {
        Ok(self.keydir.len())
    }

    pub fn page(&mut self, limit: usize, offset: usize) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        // 边界检查：offset 超过总条数时返回空
        let mut data: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        if offset >= self.keydir.len() {
            return Ok(data.clone());
        }
        // 1. 跳过 offset 条，取 limit 条
        let page_data: Vec<(Vec<u8>, (u64, u32))> = self
            .keydir
            .iter()
            // skip(offset) 跳过前面 offset 个元素
            .skip(offset)
            // take(limit) 只取最多 limit 个元素
            .take(limit)
            // 克隆数据（因为 iter() 是引用，我们要返回所有权）
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        for (key, (offset, val_size)) in page_data {
            let val = self.log.read_value(offset, val_size)?;
            data.push((key, val));
        }
        Ok(data)
    }
}
