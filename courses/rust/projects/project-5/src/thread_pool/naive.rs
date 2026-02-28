use std::thread;

use super::ThreadPool;
use crate::Result;

/// 实际上它不是一个线程池，每次调用 `spawn` 方法时都会创建一个新线程。
#[derive(Clone)]
pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    fn new(_threads: u32) -> Result<Self> {
        Ok(NaiveThreadPool)
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(job);
    }
}