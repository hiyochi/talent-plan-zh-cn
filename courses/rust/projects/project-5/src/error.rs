use failure::Fail;
use std::io;
use std::string::FromUtf8Error;

/// KVS 的错误类型
#[derive(Fail, Debug)]
pub enum KvsError {
    /// IO 错误
    #[fail(display = "IO error: {}", _0)]
    Io(#[cause] io::Error),
    /// 序列化或反序列化错误
    #[fail(display = "serde_json error: {}", _0)]
    Serde(#[cause] serde_json::Error),
    /// 删除不存在的键的错误
    #[fail(display = "Key not found")]
    KeyNotFound,
    /// 未知的命令类型错误。
    /// 表示日志损坏或程序存在 bug。
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,
    /// 键或值不是有效的 UTF-8 序列
    #[fail(display = "UTF-8 error: {}", _0)]
    Utf8(#[cause] FromUtf8Error),
    /// Sled 错误
    #[fail(display = "sled error: {}", _0)]
    Sled(#[cause] sled::Error),
    /// 带字符串消息的错误
    #[fail(display = "{}", _0)]
    StringError(String),
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

impl From<FromUtf8Error> for KvsError {
    fn from(err: FromUtf8Error) -> KvsError {
        KvsError::Utf8(err)
    }
}

impl From<sled::Error> for KvsError {
    fn from(err: sled::Error) -> KvsError {
        KvsError::Sled(err)
    }
}

/// KVS 的结果类型
pub type Result<T> = std::result::Result<T, KvsError>;