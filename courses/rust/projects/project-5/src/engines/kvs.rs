```rust
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam::queue::ArrayQueue;
use crossbeam_skiplist::SkipMap;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use tokio::prelude::*;
use tokio::sync::oneshot;

use super::KvsEngine;
use crate::thread_pool::ThreadPool;
use crate::{KvsError, Result};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// `KvStore` 存储字符串键值对。
///
/// 键值对以日志文件的形式持久化到磁盘。日志文件以单调递增的生成编号命名，扩展名为 `log`。
/// 内存中的跳表存储键及其值的位置，以实现快速查询。
///
/// ```rust
/// # use kvs::{KvStore, Result};
/// # use kvs::thread_pool::{ThreadPool, RayonThreadPool};
/// # use tokio::prelude::*;
/// # fn try_main() -> Result<()> {
/// use std::env::current_dir;
/// use kvs::KvsEngine;
/// let mut store: KvStore<RayonThreadPool> = KvStore::open(current_dir()?, 2)?;
/// store.set("key".to_owned(), "value".to_owned()).wait()?;
/// let val = store.get("key".to_owned()).wait()?;
/// assert_eq!(val, Some("value".to_owned()));
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct KvStore<P: ThreadPool> {
    // 日志和其他数据的目录
    path: Arc<PathBuf>,
    // 将生成编号映射到文件读取器
    index: Arc<SkipMap<String, CommandPos>>,
    writer: Arc<Mutex<KvStoreWriter>>,
    thread_pool: P,
    reader_pool: Arc<ArrayQueue<KvStoreReader>>,
}

impl<P: ThreadPool> KvStore<P> {
    /// 使用给定路径打开一个 `KvStore`。
    ///
    /// 如果指定的目录不存在，则会创建它。
    ///
    /// `concurrency` 指定最多可以同时读取数据库的线程数。
    ///
    /// # 错误
    ///
    /// 在日志重放期间，会传播 I/O 或反序列化错误。
    pub fn open(path: impl Into<PathBuf>, concurrency: u32) -> Result<Self> {
        let path = Arc::new(path.into());
        fs::create_dir_all(&*path)?;

        let mut readers = BTreeMap::new();
        let index = Arc::new(SkipMap::new());

        let gen_list = sorted_gen_list(&path)?;
        let mut uncompacted = 0;

        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, gen))?)?;
            uncompacted += load(gen, &mut reader, &*index)?;
            readers.insert(gen, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, current_gen)?;
        let safe_point = Arc::new(AtomicU64::new(0));

        let reader = KvStoreReader {
            path: Arc::clone(&path),
            safe_point,
            readers: RefCell::new(BTreeMap::new()),
        };

        let writer = KvStoreWriter {
            reader: reader.clone(),
            writer,
            current_gen,
            uncompacted,
            path: Arc::clone(&path),
            index: Arc::clone(&index),
        };

        let thread_pool = P::new(concurrency)?;
        let reader_pool = Arc::new(ArrayQueue::new(concurrency as usize));
        for _ in 1..concurrency {
            reader_pool.push(reader.clone()).unwrap();
        }
        reader_pool.push(reader).unwrap();

        Ok(KvStore {
            path,
            index,
            writer: Arc::new(Mutex::new(writer)),
            thread_pool,
            reader_pool,
        })
    }
}

impl<P: ThreadPool> KvsEngine for KvStore<P> {
    /// 设置字符串键的值为字符串。
    ///
    /// 如果键已存在，则会覆盖之前的值。
    ///
    /// # 错误
    ///
    /// 在写入日志时，会传播 I/O 或序列化错误。
    fn set(&self, key: String, value: String) -> Box<dyn Future<Item = (), Error = KvsError> + Send> {
        let writer = self.writer.clone();
        let (tx, rx) = oneshot::channel();
        self.thread_pool.spawn(move || {
            let res = writer.lock().unwrap().set(key, value);
            if tx.send(res).is_err() {
                error!("接收端已被丢弃");
            }
        });
        Box::new(
            rx.map_err(|e| KvsError::StringError(format!("{}", e)))
                .flatten(),
        )
    }

    /// 获取给定字符串键的字符串值。
    ///
    /// 如果给定的键不存在，则返回 `None`。
    fn get(&self, key: String) -> Box<dyn Future<Item = Option<String>, Error = KvsError> + Send> {
        let reader_pool = self.reader_pool.clone();
        let index = self.index.clone();
        let (tx, rx) = oneshot::channel();
        self.thread_pool.spawn(move || {
            let res = (|| {
                if let Some(cmd_pos) = index.get(&key) {
                    let reader = reader_pool.pop().unwrap();
                    let res = if let Command::Set { value, .. } =
                        reader.read_command(*cmd_pos.value())?
                    {
                        Ok(Some(value))
                    } else {
                        Err(KvsError::UnexpectedCommandType)
                    };
                    reader_pool.push(reader).unwrap();
                    res
                } else {
                    Ok(None)
                }
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

    /// 删除给定的键。
    ///
    /// # 错误
    ///
    /// 如果给定的键不存在，则返回 `KvsError::KeyNotFound`。
    ///
    /// 在写入日志时，会传播 I/O 或序列化错误。
    fn remove(&self, key: String) -> Box<dyn Future<Item = (), Error = KvsError> + Send> {
        let writer = self.writer.clone();
        let (tx, rx) = oneshot::channel();
        self.thread_pool.spawn(move || {
            let res = writer.lock().unwrap().remove(key);
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

/// 单线程读取器。
///
/// 每个 `KvStore` 实例都有自己的 `KvStoreReader`，
/// 并且 `KvStoreReader` 会分别打开相同的文件。因此，用户可以通过不同线程中的多个 `KvStore`
/// 并发读取。
struct KvStoreReader {
    path: Arc<PathBuf>,
    // 最新压缩文件的生成编号
    safe_point: Arc<AtomicU64>,
    readers: RefCell<BTreeMap<u64, BufReaderWithPos<File>>>,
}

impl KvStoreReader {
    /// 关闭生成编号小于 safe_point 的文件句柄。
    ///
    /// `safe_point` 在一次压缩完成后更新为最新的压缩生成编号。
    /// 压缩生成编号包含其之前的所有操作，内存索引中不再包含生成编号小于 safe_point 的条目。
    /// 因此，我们可以安全地关闭这些文件句柄，过时的文件可以被删除。
    fn close_stale_handles(&self) {
        let mut readers = self.readers.borrow_mut();
        while !readers.is_empty() {
            let first_gen = *readers.keys().next().unwrap();
            if self.safe_point.load(Ordering::SeqCst) <= first_gen {
                break;
            }
            readers.remove(&first_gen);
        }
    }

    /// 在给定的 `CommandPos` 处读取日志文件。
    fn read_and<F, R>(&self, cmd_pos: CommandPos, f: F) -> Result<R>
    where
        F: FnOnce(io::Take<&mut BufReaderWithPos<File>>) -> Result<R>,
    {
        self.close_stale_handles();

        let mut readers = self.readers.borrow_mut();
        // 如果我们尚未在该 `KvStoreReader` 中打开该文件，则打开它。
        // 我们不使用 entry API，因为我们希望传播错误。
        if !readers.contains_key(&cmd_pos.gen) {
            let reader = BufReaderWithPos::new(File::open(log_path(&self.path, cmd_pos.gen))?)?;
            readers.insert(cmd_pos.gen, reader);
        }
        let reader = readers.get_mut(&cmd_pos.gen).unwrap();
        reader.seek(SeekFrom::Start(cmd_pos.pos))?;
        let cmd_reader = reader.take(cmd_pos.len);
        f(cmd_reader)
    }

    // 在给定的 `CommandPos` 处读取日志文件并反序列化为 `Command`。
    fn read_command(&self, cmd_pos: CommandPos) -> Result<Command> {
        self.read_and(cmd_pos, |cmd_reader| {
            Ok(serde_json::from_reader(cmd_reader)?)
        })
    }
}

impl Clone for KvStoreReader {
    fn clone(&self) -> KvStoreReader {
        KvStoreReader {
            path: Arc::clone(&self.path),
            safe_point: Arc::clone(&self.safe_point),
            // 不使用其他 KvStoreReader 的 readers
            readers: RefCell::new(BTreeMap::new()),
        }
    }
}

struct KvStoreWriter {
    reader: KvStoreReader,
    writer: BufWriterWithPos<File>,
    current_gen: u64,
    // 在压缩期间可删除的“过时”命令所占的字节数
    uncompacted: u64,
    path: Arc<PathBuf>,
    index: Arc<SkipMap<String, CommandPos>>,
}

impl KvStoreWriter {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self.index.get(&key) {
                self.uncompacted += old_cmd.value().len;
            }
            self.index
                .insert(key, (self.current_gen, pos..self.writer.pos).into());
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            let pos = self.writer.pos;
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            if let Command::Remove { key } = cmd {
                let old_cmd = self.index.remove(&key).expect("键不存在");
                self.uncompacted += old_cmd.value().len;
                // “remove” 命令本身可以在下一次压缩中被删除，
                // 因此将其长度添加到 `uncompacted`
                self.uncompacted += self.writer.pos - pos;
            }

            if self.uncompacted > COMPACTION_THRESHOLD {
                self.compact()?;
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }

    /// 清除日志中的过时条目。
    fn compact(&mut self) -> Result<()> {
        // 将当前生成编号增加 2。current_gen + 1 用于压缩文件
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = new_log_file(&self.path, self.current_gen)?;

        let mut compaction_writer = new_log_file(&self.path, compaction_gen)?;

        let mut new_pos = 0; // 新日志文件中的位置
        for entry in self.index.iter() {
            let len = self.reader.read_and(*entry.value(), |mut entry_reader| {
                Ok(io::copy(&mut entry_reader, &mut compaction_writer)?)
            })?;
            self.index.insert(
                entry.key().clone(),
                (compaction_gen, new_pos..new_pos + len).into(),
            );
            new_pos += len;
        }
        compaction_writer.flush()?;

        self.reader
            .safe_point
            .store(compaction_gen, Ordering::SeqCst);
        self.reader.close_stale_handles();

        // 删除过时的日志文件
        // 注意：实际上这些文件不会立即被删除，因为 `KvStoreReader` 仍保持打开的文件句柄。
        // 当 `KvStoreReader` 下次使用时，它会清除其过时的文件句柄。
        // 在 Unix 上，当所有句柄关闭后，文件会被删除。
        // 在 Windows 上，以下删除操作会失败，预期在下一次压缩中删除过时文件。

        let stale_gens = sorted_gen_list(&self.path)?
            .into_iter()
            .filter(|&gen| gen < compaction_gen);
        for stale_gen in stale_gens {
            let file_path = log_path(&self.path, stale_gen);
            if let Err(e) = fs::remove_file(&file_path) {
                error!("{:?} 无法删除: {}", file_path, e);
            }
        }
        self.uncompacted = 0;

        Ok(())
    }
}

/// 使用给定的生成编号创建一个新的日志文件，并将读取器添加到 readers 映射中。
///
/// 返回日志的写入器。
fn new_log_file(path: &Path, gen: u64) -> Result<BufWriterWithPos<File>> {
    let path = log_path(&path, gen);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;
    Ok(writer)
}

/// 返回给定目录中排序的生成编号列表
fn sorted_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut gen_list: Vec<u64> = fs::read_dir(&path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    gen_list.sort_unstable();
    Ok(gen_list)
}

/// 加载整个日志文件并将值位置存储在索引映射中。
///
/// 返回压缩后可节省的字节数。
fn load(
    gen: u64,
    reader: &mut BufReaderWithPos<File>,
    index: &SkipMap<String, CommandPos>,
) -> Result<u64> {
    // 确保从文件开头开始读取
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0; // 压缩后可节省的字节数
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) = index.get(&key) {
                    uncompacted += old_cmd.value().len;
                }
                index.insert(key, (gen, pos..new_pos).into());
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.value().len;
                }
                // “remove” 命令本身可以在下一次压缩中被删除，
                // 因此将其长度添加到 `uncompacted`
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos;
    }
    Ok(uncompacted)
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{}.log", gen))
}

/// 表示命令的结构体
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

/// 表示日志中 JSON 序列化命令的位置和长度
#[derive(Debug, Clone, Copy)]
struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((gen, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            gen,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
```