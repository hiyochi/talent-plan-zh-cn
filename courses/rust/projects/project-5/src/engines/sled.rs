use crate::thread_pool::ThreadPool;
use crate::{KvsEngine, KvsError, Result};
use sled::Db;
use tokio::prelude::*;
use tokio::sync::oneshot;

/// `sled::Db` 的包装器
#[derive(Clone)]
pub struct SledKvsEngine<P: ThreadPool> {
    pool: P,
    db: Db,
}

impl<P: ThreadPool> SledKvsEngine<P> {
    /// 从 `sled::Db` 创建一个 `SledKvsEngine`。
    ///
    /// 操作将在指定的线程池中运行。`concurrency` 指定线程池中的线程数量。
    pub fn new(db: Db, concurrency: u32) -> Result<Self> {
        let pool = P::new(concurrency)?;
        Ok(SledKvsEngine { pool, db })
    }
}

impl<P: ThreadPool> KvsEngine for SledKvsEngine<P> {
    fn set(&self, key: String, value: String) -> Box<dyn Future<Item = (), Error = KvsError> + Send> {
        let db = self.db.clone();
        let (tx, rx) = oneshot::channel();
        self.pool.spawn(move || {
            let res = db
                .set(key, value.into_bytes())
                .and_then(|_| db.flush())
                .map(|_| ())
                .map_err(KvsError::from);
            if tx.send(res).is_err() {
                error!("接收端已被丢弃");
            }
        });
        Box::new(
            rx.map_err(|e| KvsError::StringError(format!("{}", e)))
                .flatten(),
        )
    }

    fn get(&self, key: String) -> Box<dyn Future<Item = Option<String>, Error = KvsError> + Send> {
        let db = self.db.clone();
        let (tx, rx) = oneshot::channel();
        self.pool.spawn(move || {
            let res = (move || {
                Ok(db
                    .get(key)?
                    .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
                    .map(String::from_utf8)
                    .transpose()?)
            })();
            if tx.send(res).is_err() {
                error!("接收端已被丢弃");
            }
        });
        Box::new(
            rx.map_err(|e| KvsError::StringError(format!("{}", e)))
                .flatten(),
        )
    }

    fn remove(&self, key: String) -> Box<dyn Future<Item = (), Error = KvsError> + Send> {
        let db = self.db.clone();
        let (tx, rx) = oneshot::channel();
        self.pool.spawn(move || {
            let res = (|| {
                db.del(key)?.ok_or(KvsError::KeyNotFound)?;
                db.flush()?;
                Ok(())
            })();
            if tx.send(res).is_err() {
                error!("接收端已被丢弃");
            }
        });
        Box::new(
            rx.map_err(|e| KvsError::StringError(format!("{}", e)))
                .flatten(),
        )
    }
}