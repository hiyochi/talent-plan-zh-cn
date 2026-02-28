#![deny(missing_docs)]
//! 一个简单的键值存储。

pub use error::{KvsError, Result};
pub use kv::KvStore;

mod error;
mod kv;