use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Get { key: String },           // 获取指定键的值
    Set { key: String, value: String }, // 设置键值对
    Remove { key: String },        // 删除指定键
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GetResponse {
    Ok(Option<String>),            // 获取成功，返回值（可能为None）
    Err(String),                   // 获取失败，返回错误信息
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SetResponse {
    Ok(()),                        // 设置成功
    Err(String),                   // 设置失败，返回错误信息
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RemoveResponse {
    Ok(()),                        // 删除成功
    Err(String),                   // 删除失败，返回错误信息
}