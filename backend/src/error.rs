// # 后端自定义错误

use std::{fmt::Display};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    InvalidProgramId(String),
    RpcError(String),
    RpcUrlError(String),
    ConfigError(String),
    ProgramIdError(String),
    PayerKeypairError(String),
    PubKeyError,
    LogError(String),
    InvalidKey,
    ServerPortError(String),
    IoError(String),
    TreasuryError(String),
    ParseEmitError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidProgramId(msg) => write!(f, "无效的程序ID: {msg}"),
            Error::RpcError(msg) => write!(f, "RPC 错误: {msg}"),
            Error::ConfigError(msg) => write!(f, "配置错误: {msg}"),
            Error::RpcUrlError(msg) => write!(f, "RPC 路径错误: {msg}"),
            Error::ProgramIdError(msg) => write!(f, "Program Id 错误: {msg}"),
            Error::PayerKeypairError(msg) => write!(f, "Payer Keypair 错误: {msg}"),
            Error::PubKeyError => write!(f, "Pubkey 错误"),
            Error::LogError(msg) => write!(f, "Log 错误: {msg}"),
            Error::InvalidKey => write!(f, "无效的 Key"),
            Error::ServerPortError(msg) => write!(f, "服务端口 错误: {msg}"),
            Error::IoError(msg) => write!(f, "IO 错误: {msg}"),
            Error::TreasuryError(msg) => write!(f, "Treasury 错误: {msg}"),
            Error::ParseEmitError(msg) => write!(f, "解析 Emit 错误: {msg}"),
        }
    }
}

// 只保留这一个！自动兼容 tokio + std io::Error
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(format!("{}", e))
    }
}

impl From<dotenv::Error> for Error {
    fn from(e: dotenv::Error) -> Self {
        Error::ConfigError(format!("{}", e))
    }
}

