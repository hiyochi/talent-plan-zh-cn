//! 该模块提供了多种线程池。所有线程池都应实现 `ThreadPool` 特征。

use crate::Result;

mod naive;
mod rayon;
mod shared_queue;

pub use self::naive::NaiveThreadPool;
pub use self::rayon::RayonThreadPool;
pub use self::shared_queue::SharedQueueThreadPool;

/// 所有线程池都应实现的特征。
pub trait ThreadPool {
    /// 创建一个新的线程池，并立即启动指定数量的线程。
    ///
    /// 如果任何线程启动失败，则返回错误。所有已启动的线程将被终止。
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;

    /// 将一个函数提交到线程池中执行。
    ///
    /// 提交操作始终成功，但如果函数发生恐慌，线程池仍会继续运行，线程数量不会减少，
    /// 线程池也不会被销毁、损坏或失效。
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}