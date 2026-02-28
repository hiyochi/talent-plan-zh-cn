//! 支持 Raft 和 kvraft 持久化
//! Raft 状态（日志等）和 k/v 服务器快照。
//!
//! 我们将使用原始的 persister.rs 来测试你的代码以进行评分。
//! 因此，虽然你可以修改此代码以帮助调试，
//! 但在提交前请使用原始版本进行测试。

use std::sync::{Arc, Mutex};

pub trait Persister: Send + 'static {
    fn raft_state(&self) -> Vec<u8>;
    fn save_raft_state(&self, state: Vec<u8>);
    fn save_state_and_snapshot(&self, state: Vec<u8>, snapshot: Vec<u8>);
    fn snapshot(&self) -> Vec<u8>;
}

impl<T: ?Sized + Persister> Persister for Box<T> {
    fn raft_state(&self) -> Vec<u8> {
        (**self).raft_state()
    }
    fn save_raft_state(&self, state: Vec<u8>) {
        (**self).save_raft_state(state)
    }
    fn save_state_and_snapshot(&self, state: Vec<u8>, snapshot: Vec<u8>) {
        (**self).save_state_and_snapshot(state, snapshot)
    }
    fn snapshot(&self) -> Vec<u8> {
        (**self).snapshot()
    }
}

impl<T: ?Sized + Sync + Persister> Persister for Arc<T> {
    fn raft_state(&self) -> Vec<u8> {
        (**self).raft_state()
    }
    fn save_raft_state(&self, state: Vec<u8>) {
        (**self).save_raft_state(state)
    }
    fn save_state_and_snapshot(&self, state: Vec<u8>, snapshot: Vec<u8>) {
        (**self).save_state_and_snapshot(state, snapshot)
    }
    fn snapshot(&self) -> Vec<u8> {
        (**self).snapshot()
    }
}

#[derive(Default)]
pub struct SimplePersister {
    states: Mutex<(
        Vec<u8>, // raft 状态
        Vec<u8>, // 快照
    )>,
}

impl SimplePersister {
    pub fn new() -> SimplePersister {
        SimplePersister {
            states: Mutex::default(),
        }
    }
}

impl Persister for SimplePersister {
    fn raft_state(&self) -> Vec<u8> {
        self.states.lock().unwrap().0.clone()
    }

    fn save_raft_state(&self, state: Vec<u8>) {
        self.states.lock().unwrap().0 = state;
    }

    fn save_state_and_snapshot(&self, state: Vec<u8>, snapshot: Vec<u8>) {
        self.states.lock().unwrap().0 = state;
        self.states.lock().unwrap().1 = snapshot;
    }

    fn snapshot(&self) -> Vec<u8> {
        self.states.lock().unwrap().1.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_safety() {
        let sp = SimplePersister::new();
        sp.save_raft_state(vec![111]);
        let obj: Box<dyn Persister + Sync> = Box::new(sp);
        assert_eq!(obj.raft_state(), vec![111]);
        obj.save_state_and_snapshot(vec![222], vec![123]);
        assert_eq!(obj.raft_state(), vec![222]);
        assert_eq!(obj.snapshot(), vec![123]);

        let cloneable_obj: Arc<dyn Persister> = Arc::new(obj);
        assert_eq!(cloneable_obj.raft_state(), vec![222]);
        assert_eq!(cloneable_obj.snapshot(), vec![123]);

        let cloneable_obj_ = cloneable_obj.clone();
        cloneable_obj.save_raft_state(vec![233]);
        assert_eq!(cloneable_obj_.raft_state(), vec![233]);
        assert_eq!(cloneable_obj_.snapshot(), vec![123]);

        let sp = SimplePersister::new();
        let obj: Arc<dyn Persister + Sync> = Arc::new(sp);
        let _box_obj: Box<dyn Persister> = Box::new(obj);
    }
}