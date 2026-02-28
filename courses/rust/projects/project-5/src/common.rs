use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Get { key: String },           // 获取指定键的值
    Set { key: String, value: String }, // 设置键值对
    Remove { key: String },        // 删除指定键
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Get(Option<String>),           // 返回获取到的值，若不存在则为 None
    Set,                           // 设置成功
    Remove,                        // 删除成功
    Err(String),                   // 错误信息
}