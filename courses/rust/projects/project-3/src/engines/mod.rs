//! 该模块提供了多种键值存储引擎。

use crate::Result;

/// 键值存储引擎的 trait。
pub trait KvsEngine {
    /// 将字符串键的值设置为字符串。
    ///
    /// 如果键已存在，则会覆盖之前的值。
    fn set(&mut self, key: String, value: String) -> Result<()>;

    /// 获取给定字符串键的字符串值。
    ///
    /// 如果给定的键不存在，则返回 `None`。
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// 删除给定的键。
    ///
    /// # 错误
    ///
    /// 如果未找到给定的键，则返回 `KvsError::KeyNotFound`。
    fn remove(&mut self, key: String) -> Result<()>;
}

mod kvs;
mod sled;

pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;