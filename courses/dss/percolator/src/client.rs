use labrpc::*;

use crate::service::{TSOClient, TransactionClient};

// BACKOFF_TIME_MS 是重试发送请求前的等待时间。
// 它应该呈指数增长。例如：
//|  重试次数  |  退避时间  |
//|------------|------------|
//|      1     |    100     |
//|      2     |    200     |
//|      3     |    400     |
const BACKOFF_TIME_MS: u64 = 100;
// RETRY_TIMES 是客户端尝试发送请求的最大次数。
const RETRY_TIMES: usize = 3;

/// 客户端主要有两个用途：
/// 一是从 TSO（时间戳预言机）获取单调递增的时间戳。
/// 二是执行事务逻辑。
#[derive(Clone)]
pub struct Client {
    // 你的定义写在这里。
}

impl Client {
    /// 创建一个新的客户端。
    pub fn new(tso_client: TSOClient, txn_client: TransactionClient) -> Client {
        // 你的代码写在这里。
        Client {}
    }

    /// 从 TSO 获取一个时间戳。
    pub fn get_timestamp(&self) -> Result<u64> {
        // 你的代码写在这里。
        unimplemented!()
    }

    /// 开始一个新的事务。
    pub fn begin(&mut self) {
        // 你的代码写在这里。
        unimplemented!()
    }

    /// 获取给定键的值。
    pub fn get(&self, key: Vec<u8>) -> Result<Vec<u8>> {
        // 你的代码写在这里。
        unimplemented!()
    }

    /// 将键值对暂存到缓冲区，直到提交时再处理。
    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        // 你的代码写在这里。
        unimplemented!()
    }

    /// 提交一个事务。
    pub fn commit(&self) -> Result<bool> {
        // 你的代码写在这里。
        unimplemented!()
    }
}