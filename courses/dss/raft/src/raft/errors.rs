```rust
use std::{error, fmt, result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    // 编码错误
    Encode(labcodec::EncodeError),
    // 解码错误
    Decode(labcodec::DecodeError),
    // RPC 错误
    Rpc(labrpc::Error),
    // 非领导者错误
    NotLeader,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 使用 Debug 格式化输出错误
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // 返回错误的根本原因
        match *self {
            // 如果是编码错误，返回其根本原因
            Error::Encode(ref e) => Some(e),
            // 如果是解码错误，返回其根本原因
            Error::Decode(ref e) => Some(e),
            // 如果是 RPC 错误，返回其根本原因
            Error::Rpc(ref e) => Some(e),
            // 其他情况没有根本原因
            _ => None,
        }
    }
}

// 定义 Result 类型，用于返回结果或错误
pub type Result<T> = result::Result<T, Error>;
```