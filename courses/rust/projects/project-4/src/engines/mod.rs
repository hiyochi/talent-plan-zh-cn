pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;
use crate::Result;

mod kvs;
mod sled;

/// 键值存储引擎的 trait。
pub trait KvsEngine: Clone + Send + 'static {
    /// 将字符串键的值设置为字符串。
    ///
    /// 如果键已存在，则会覆盖之前的值。
    fn set(&self, key: String, value: String) -> Result<()>;

    /// 获取给定字符串键的字符串值。
    ///
    /// 如果给定的键不存在，则返回 `None`。
    fn get(&self, key: String) -> Result<Option<String>>;

    /// 删除给定的键。
    ///
    /// # 错误
    ///
    /// 如果未找到给定的键，则返回 `KvsError::KeyNotFound`。
    fn remove(&self, key: String) -> Result<()>;
}