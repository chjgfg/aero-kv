// # 后端自定义错误

use std::fmt::Display;
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
    LimitError,
    InvalidPageError,
    InvalidParam(String),
    InternalError(String),
    UnsupportedCharacter(String),
    UserDoesNotExistError,
    PermissionDoesNotExistError,
    UserLogout,
    KeyTooLong,
    ValueTooLong,
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
            Error::LimitError => write!(f, "Limit 输入 错误"),
            Error::InvalidPageError => write!(f, "输入 页码 错误"),
            Error::InvalidParam(msg) => write!(f, "无效参数: {msg}"),
            Error::InternalError(msg) => write!(f, "从bitcask取数据报错: {msg}"),
            Error::UnsupportedCharacter(msg) => write!(f, "无效的字符: {msg}"),
            Error::UserDoesNotExistError => write!(f, "用户不存在"),
            Error::PermissionDoesNotExistError => write!(f, "权限不存在"),
            Error::UserLogout => write!(f, "用户未登录"),
            Error::KeyTooLong => write!(f, "key 太大了"),
            Error::ValueTooLong => write!(f, "value 太大了"),
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
