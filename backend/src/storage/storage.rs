use crate::error::Result;
use std::{
    collections::BTreeMap,
    fs,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

/// 索引
pub type KeyDir = BTreeMap<Vec<u8>, (u64, u32)>; // key | (offset, value-len)

const LOG_HEADER_SIZE: u32 = 8;

pub struct Log {
    pub file_path: PathBuf, // 日志存储路径
    pub file: File,         //日志存储文件
}

impl Log {
    /// 日志初始化
    pub fn new(file_path: PathBuf) -> Self {
        if let Some(dir) = file_path.parent() {
            if !dir.exists() {
                fs::create_dir(dir).unwrap();
            }
        }
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(file_path.clone())
            .unwrap();
        Self { file_path, file }
    }

    /// 构建索引
    pub fn build_index(&mut self) -> Result<KeyDir> {
        let mut key_dir = KeyDir::new();
        let file_len = self.file.metadata().unwrap().len();
        let mut read_buf = BufReader::new(&self.file);
        let mut offset: u64 = 0;
        loop {
            if offset >= file_len {
                break;
            }
            let (key, value_size) = Self::read_entry(&mut read_buf, offset)?;
            let key_size = key.len() as u64;
            if value_size == -1 {
                key_dir.remove(&key);
                offset += key_size + LOG_HEADER_SIZE as u64;
            } else {
                key_dir.insert(key, (offset + LOG_HEADER_SIZE as u64 + key_size, value_size as u32,), );
                offset += key_size + value_size as u64 + LOG_HEADER_SIZE as u64;
            }
        }
        Ok(key_dir)
    }

    /// 从数据文件读取数据方便构建索引
    fn read_entry(read_buf: &mut BufReader<&File>, offset: u64) -> Result<(Vec<u8>, i32)> {
        read_buf.seek(SeekFrom::Start(offset))?;
        let mut buf = [0; 4];
        // 读取 key size
        read_buf.read_exact(&mut buf)?;
        // from_le_bytes 小端读出, to_le_bytes 小端写入
        // from_be_bytes 大端读出, to_be_bytes 大端写入
        let key_size = u32::from_be_bytes(buf);
        // 读取 key value
        read_buf.read_exact(&mut buf)?;
        let value_size = i32::from_be_bytes(buf); // value_len 可能是 -1，所以是i64
        // 读取 key
        let mut key = vec![0; key_size as usize];
        read_buf.read_exact(&mut key)?;
        Ok((key, value_size))
    }

    /// +-------------+-------------+----------------+----------------+
    /// | key len(4)    val len(4)     key(varint)       val(varint)  |
    /// +-------------+-------------+----------------+----------------+
    pub fn write_entry(&mut self, key: &Vec<u8>, value: Option<&Vec<u8>>) -> Result<(u64, u32)> {
        // 传引用是为了避免数据拷贝，这个函数直接返回 (offset, size) 即可
        let offset = self.file.seek(SeekFrom::End(0))?;
        let key_size = key.len() as u32;
        let value_size = value.map_or(0, |v| v.len() as u32);
        let total_size = key_size + value_size + LOG_HEADER_SIZE;
        let mut writer_buf = BufWriter::with_capacity(total_size as usize, &mut self.file); // 得到了一个写缓冲器, (缓冲区大小，文件)
        writer_buf.write_all(&key_size.to_be_bytes())?; // write_all 保证必须将内容全部写入，否则会报错
        writer_buf.write_all(&value.map_or(-1, |v| v.len() as i32).to_be_bytes())?; // value为None则value_size = -1
        writer_buf.write_all(&key)?;
        if let Some(v) = value {
            writer_buf.write_all(v)?;
        }
        writer_buf.flush()?;
        Ok((offset, total_size))
    }

    pub fn read_value(&mut self, offset: u64, size: u32) -> Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(offset))?;
        // 仅分配容量但长度为0，无法容纳读取的数据
        // let mut read_buf = Vec::with_capacity(size as usize);
        let mut read_buf = vec![0; size as usize];
        self.file.read_exact(&mut read_buf)?; // 和write_all() 一样，read_exact()保证必须将内容全部读完，否则会报错
        Ok(read_buf) // buffer是大小为value长度的01字符流
    }
}
