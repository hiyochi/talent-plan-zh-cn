use kvs::{KvStore, KvsEngine, Result};
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;
use walkdir::WalkDir;

// 应该获取之前存储的值
#[test]
fn get_stored_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;

    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    // 从磁盘重新打开并检查持久化数据
    drop(store);
    let store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    Ok(())
}

// 应该覆盖已存在的值
#[test]
fn overwrite_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    store.set("key1".to_owned(), "value2".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

    // 从磁盘重新打开并检查持久化数据
    drop(store);
    let store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));
    store.set("key1".to_owned(), "value3".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value3".to_owned()));

    Ok(())
}

// 当获取不存在的键时，应返回 `None`
#[test]
fn get_non_existent_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    // 从磁盘重新打开并检查持久化数据
    drop(store);
    let store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    Ok(())
}

#[test]
fn remove_non_existent_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;
    assert!(store.remove("key1".to_owned()).is_err());
    Ok(())
}

#[test]
fn remove_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;
    store.set("key1".to_owned(), "value1".to_owned())?;
    assert!(store.remove("key1".to_owned()).is_ok());
    assert_eq!(store.get("key1".to_owned())?, None);
    Ok(())
}

// 插入数据直到目录总大小减小。
// 测试压缩后数据的正确性。
#[test]
fn compaction() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;

    let dir_size = || {
        let entries = WalkDir::new(temp_dir.path()).into_iter();
        let len: walkdir::Result<u64> = entries
            .map(|res| {
                res.and_then(|entry| entry.metadata())
                    .map(|metadata| metadata.len())
            })
            .sum();
        len.expect("fail to get directory size")
    };

    let mut current_size = dir_size();
    for iter in 0..1000 {
        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            let value = format!("{}", iter);
            store.set(key, value)?;
        }

        let new_size = dir_size();
        if new_size > current_size {
            current_size = new_size;
            continue;
        }
        // 触发压缩

        drop(store);
        // 重新打开并检查内容
        let store = KvStore::open(temp_dir.path())?;
        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            assert_eq!(store.get(key)?, Some(format!("{}", iter)));
        }
        return Ok(());
    }

    panic!("No compaction detected");
}

#[test]
fn concurrent_set() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;
    let barrier = Arc::new(Barrier::new(1001));
    for i in 0..1000 {
        let store = store.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            store
                .set(format!("key{}", i), format!("value{}", i))
                .unwrap();
            barrier.wait();
        });
    }
    barrier.wait();

    for i in 0..1000 {
        assert_eq!(store.get(format!("key{}", i))?, Some(format!("value{}", i)));
    }

    // 从磁盘重新打开并检查持久化数据
    drop(store);
    let store = KvStore::open(temp_dir.path())?;
    for i in 0..1000 {
        assert_eq!(store.get(format!("key{}", i))?, Some(format!("value{}", i)));
    }

    Ok(())
}

#[test]
fn concurrent_get() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::open(temp_dir.path())?;
    for i in 0..100 {
        store
            .set(format!("key{}", i), format!("value{}", i))
            .unwrap();
    }

    let mut handles = Vec::new();
    for thread_id in 0..100 {
        let store = store.clone();
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let key_id = (i + thread_id) % 100;
                assert_eq!(
                    store.get(format!("key{}", key_id)).unwrap(),
                    Some(format!("value{}", key_id))
                );
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    // 从磁盘重新打开并检查持久化数据
    drop(store);
    let store = KvStore::open(temp_dir.path())?;
    let mut handles = Vec::new();
    for thread_id in 0..100 {
        let store = store.clone();
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let key_id = (i + thread_id) % 100;
                assert_eq!(
                    store.get(format!("key{}", key_id)).unwrap(),
                    Some(format!("value{}", key_id))
                );
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}