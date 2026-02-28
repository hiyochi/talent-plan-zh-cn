use std::thread;

use super::ThreadPool;
use crate::Result;

use crossbeam::channel::{self, Receiver, Sender};

use log::{debug, error};

// 注意：在 Rust 培训课程中，线程池未使用 `catch_unwind` 实现，因为这要求任务实现 `UnwindSafe`。

/// 一个使用内部共享队列的线程池。
///
/// 如果某个任务发生恐慌，旧线程将被销毁并创建一个新线程。
/// 在线程池创建后，如果操作系统层面创建线程失败，将静默忽略。
/// 因此，线程池中的线程数量可能减少到零，此时向线程池提交任务将导致恐慌。
pub struct SharedQueueThreadPool {
    tx: Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self> {
        let (tx, rx) = channel::unbounded::<Box<dyn FnOnce() + Send + 'static>>();
        for _ in 0..threads {
            let rx = TaskReceiver(rx.clone());
            thread::Builder::new().spawn(move || run_tasks(rx))?;
        }
        Ok(SharedQueueThreadPool { tx })
    }

    /// 将一个函数提交到线程池中执行。
    ///
    /// # 潘克（Panics）
    ///
    /// 如果线程池中没有线程，则会恐慌。
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx
            .send(Box::new(job))
            .expect("The thread pool has no thread.");
    }
}

#[derive(Clone)]
struct TaskReceiver(Receiver<Box<dyn FnOnce() + Send + 'static>>);

impl Drop for TaskReceiver {
    fn drop(&mut self) {
        if thread::panicking() {
            let rx = self.clone();
            if let Err(e) = thread::Builder::new().spawn(move || run_tasks(rx)) {
                error!("Failed to spawn a thread: {}", e);
            }
        }
    }
}

fn run_tasks(rx: TaskReceiver) {
    loop {
        match rx.0.recv() {
            Ok(task) => {
                task();
            }
            Err(_) => debug!("Thread exits because the thread pool is destroyed."),
        }
    }
}