```rust
use std::{error, fmt, result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    NoLeader, // 无领导者
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::NoLeader => None, // 无领导者错误没有源错误
        }
    }
}

pub type Result<T> = result::Result<T, Error>; // 定义结果类型，使用自定义错误类型
```