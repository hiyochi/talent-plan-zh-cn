use failure::Fail;
use std::io;

/// KVS 的错误类型。
#[derive(Fail, Debug)]
pub enum KvsError {
    /// IO 错误。
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    /// 序列化或反序列化错误。
    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),
    /// 删除不存在的键的错误。
    #[fail(display = "Key not found")]
    KeyNotFound,
    /// 意外的命令类型错误。
    /// 表示日志损坏或程序存在 bug。
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> KvsError {
        KvsError::Io(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> KvsError {
        KvsError::Serde(err)
    }
}

/// KVS 的结果类型。
pub type Result<T> = std::result::Result<T, KvsError>;