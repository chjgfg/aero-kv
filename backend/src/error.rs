// # 后端自定义错误

use std::fmt::Display;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    WalletError(String),
    InvalidProgramId(String),
    RpcError(String),
    ConfigError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WalletError(msg) => write!(f, "钱包错误: {msg}"),
            Error::InvalidProgramId(msg) => write!(f, "无效的程序ID: {msg}"),
            Error::RpcError(msg) => write!(f, "RPC 错误: {msg}"),
            Error::ConfigError(msg) => write!(f, "配置错误: {msg}"),
        }
    }
}
