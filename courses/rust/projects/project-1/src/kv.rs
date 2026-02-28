use std::collections::HashMap;

/// `KvStore` 存储字符串键值对。
///
/// 键值对存储在内存中的 `HashMap` 中，不会持久化到磁盘。
///
/// 示例：
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to_owned(), "value".to_owned());
/// let val = store.get("key".to_owned());
/// assert_eq!(val, Some("value".to_owned()));
/// ```
#[derive(Default)]
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    /// 创建一个 `KvStore`。
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
        }
    }

    /// 将字符串键的值设置为另一个字符串。
    ///
    /// 如果键已存在，则会覆盖之前的值。
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// 获取指定字符串键的字符串值。
    ///
    /// 如果给定的键不存在，则返回 `None`。
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }

    /// 删除指定的键。
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}