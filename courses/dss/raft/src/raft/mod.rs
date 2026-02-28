```rust
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender;

#[cfg(test)]
pub mod config;
pub mod errors;
pub mod persister;
#[cfg(test)]
mod tests;

use self::errors::*;
use self::persister::*;
use crate::proto::raftpb::*;

/// 当每个 Raft 节点意识到连续的日志条目已被提交时，
/// 该节点应通过传递给 `Raft::new` 的 `apply_ch` 通道，
/// 向同一服务器上的服务（或测试器）发送一个 `ApplyMsg`。
pub enum ApplyMsg {
    Command {
        data: Vec<u8>,
        index: u64,
    },
    // 用于 2D 实验：
    Snapshot {
        data: Vec<u8>,
        term: u64,
        index: u64,
    },
}

/// Raft 节点的状态。
#[derive(Default, Clone, Debug)]
pub struct State {
    pub term: u64,
    pub is_leader: bool,
}

impl State {
    /// 获取该节点当前的任期（term）。
    pub fn term(&self) -> u64 {
        self.term
    }
    /// 判断该节点是否认为自己是领导者。
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
}

// 一个 Raft 节点。
pub struct Raft {
    // 所有节点的 RPC 端点
    peers: Vec<RaftClient>,
    // 用于保存该节点持久化状态的对象
    persister: Box<dyn Persister>,
    // 该节点在 peers[] 中的索引
    me: usize,
    state: Arc<State>,
    // 你的数据写在这里（2A, 2B, 2C）。
    // 请参考论文的 Figure 2，了解 Raft 服务器必须维护哪些状态。
}

impl Raft {
    /// 服务或测试器希望创建一个 Raft 服务器。
    /// 所有 Raft 服务器（包括当前服务器）的端口信息存储在 peers 中。
    /// 当前服务器的端口是 peers[me]。
    /// 所有服务器的 peers 数组顺序一致。
    /// persister 是该服务器保存其持久化状态的地方，
    /// 同时也初始保存最近一次保存的状态（如果有的话）。
    /// apply_ch 是一个通道，测试器或服务期望 Raft 通过该通道发送 ApplyMsg 消息。
    /// 此方法必须快速返回。
    pub fn new(
        peers: Vec<RaftClient>,
        me: usize,
        persister: Box<dyn Persister>,
        apply_ch: UnboundedSender<ApplyMsg>,
    ) -> Raft {
        let raft_state = persister.raft_state();

        // 你的初始化代码写在这里（2A, 2B, 2C）。
        let mut rf = Raft {
            peers,
            persister,
            me,
            state: Arc::default(),
        };

        // 从崩溃前持久化的状态中初始化
        rf.restore(&raft_state);

        crate::your_code_here((rf, apply_ch))
    }

    /// 将 Raft 的持久化状态保存到稳定存储中，
    /// 以便在崩溃和重启后可以恢复。
    /// 请参考论文的 Figure 2，了解哪些状态需要持久化。
    fn persist(&mut self) {
        // 你的代码写在这里（2C）。
        // 示例：
        // labcodec::encode(&self.xxx, &mut data).unwrap();
        // labcodec::encode(&self.yyy, &mut data).unwrap();
        // self.persister.save_raft_state(data);
    }

    /// 恢复之前持久化的状态。
    fn restore(&mut self, data: &[u8]) {
        if data.is_empty() {
            // 没有状态的情况下启动？
        }
        // 你的代码写在这里（2C）。
        // 示例：
        // match labcodec::decode(data) {
        //     Ok(o) => {
        //         self.xxx = o.xxx;
        //         self.yyy = o.yyy;
        //     }
        //     Err(e) => {
        //         panic!("{:?}", e);
        //     }
        // }
    }

    /// 示例代码：向某个服务器发送 RequestVote RPC。
    /// server 是目标服务器在 peers 中的索引。
    /// 期望 RPC 参数通过 args 传递。
    ///
    /// labrpc 包模拟了一个有损网络，其中服务器可能不可达，
    /// 请求和响应可能会丢失。
    /// 此方法发送请求并等待响应。如果在超时时间内收到响应，
    /// 此方法返回 Ok(_)；否则返回 Err(_)。
    /// 因此，此方法可能需要一段时间才能返回。
    /// Err(_) 的返回可能由以下原因引起：服务器宕机、无法访问的存活服务器、
    /// 请求丢失或响应丢失。
    ///
    /// 此方法保证会返回（可能延迟），除非服务器端的处理函数未返回。
    /// 因此，无需在此方法周围实现自己的超时机制。
    ///
    /// 更多细节请查看 ../labrpc/src/lib.rs 中的注释。
    fn send_request_vote(
        &self,
        server: usize,
        args: RequestVoteArgs,
    ) -> Receiver<Result<RequestVoteReply>> {
        // 如果你希望 RPC 变为异步，可以在这里添加代码。
        // 示例：
        // ```
        // let peer = &self.peers[server];
        // let peer_clone = peer.clone();
        // let (tx, rx) = channel();
        // peer.spawn(async move {
        //     let res = peer_clone.request_vote(&args).await.map_err(Error::Rpc);
        //     tx.send(res);
        // });
        // rx
        // ```
        let (tx, rx) = sync_channel::<Result<RequestVoteReply>>(1);
        crate::your_code_here((server, args, tx, rx))
    }

    fn start<M>(&self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        let index = 0;
        let term = 0;
        let is_leader = true;
        let mut buf = vec![];
        labcodec::encode(command, &mut buf).map_err(Error::Encode)?;
        // 你的代码写在这里（2B）。

        if is_leader {
            Ok((index, term))
        } else {
            Err(Error::NotLeader)
        }
    }

    fn cond_install_snapshot(
        &mut self,
        last_included_term: u64,
        last_included_index: u64,
        snapshot: &[u8],
    ) -> bool {
        // 你的代码写在这里（2D）。
        crate::your_code_here((last_included_term, last_included_index, snapshot));
    }

    fn snapshot(&mut self, index: u64, snapshot: &[u8]) {
        // 你的代码写在这里（2D）。
        crate::your_code_here((index, snapshot));
    }
}

impl Raft {
    /// 仅用于抑制未使用代码的警告。
    #[doc(hidden)]
    pub fn __suppress_deadcode(&mut self) {
        let _ = self.start(&0);
        let _ = self.cond_install_snapshot(0, 0, &[]);
        self.snapshot(0, &[]);
        let _ = self.send_request_vote(0, Default::default());
        self.persist();
        let _ = &self.state;
        let _ = &self.me;
        let _ = &self.persister;
        let _ = &self.peers;
    }
}

// 选择并发范式。
//
// 你可以通过 RPC 框架驱动 Raft 状态机，
//
// ```rust
// struct Node { raft: Arc<Mutex<Raft>> }
// ```
//
// 或者启动一个新线程运行 Raft 状态机，并通过通道通信。
//
// ```rust
// struct Node { sender: Sender<Msg> }
// ```
#[derive(Clone)]
pub struct Node {
    // 你的代码写在这里。
}

impl Node {
    /// 创建一个新的 Raft 服务。
    pub fn new(raft: Raft) -> Node {
        // 你的代码写在这里。
        crate::your_code_here(raft)
    }

    /// 使用 Raft 的服务（例如键值存储服务器）希望就下一个命令达成一致，
    /// 并将其追加到 Raft 日志中。如果当前服务器不是领导者，
    /// 则返回 [`Error::NotLeader`]。否则立即启动一致性协议并返回。
    /// 不保证该命令最终会被提交到 Raft 日志中，
    /// 因为领导者可能失败或输掉选举。
    /// 即使 Raft 实例已被终止，此函数也应优雅地返回。
    ///
    /// 元组的第一个值是命令最终被提交时的索引位置，
    /// 第二个值是当前任期。
    ///
    /// 此方法不得阻塞 Raft。
    pub fn start<M>(&self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        // 你的代码写在这里。
        // 示例：
        // self.raft.start(command)
        crate::your_code_here(command)
    }

    /// 获取该节点当前的任期。
    pub fn term(&self) -> u64 {
        // 你的代码写在这里。
        // 示例：
        // self.raft.term
        crate::your_code_here(())
    }

    /// 判断该节点是否认为自己是领导者。
    pub fn is_leader(&self) -> bool {
        // 你的代码写在这里。
        // 示例：
        // self.raft.leader_id == self.id
        crate::your_code_here(())
    }

    /// 获取该节点当前的状态。
    pub fn get_state(&self) -> State {
        State {
            term: self.term(),
            is_leader: self.is_leader(),
        }
    }

    /// 测试器在不再需要 Raft 实例时调用 kill()。
    /// 你不需要在 kill() 中做任何事情，但为了方便起见，
    /// 可以（例如）关闭此实例的调试输出。
    /// 在 Raft 论文中，服务器崩溃是物理崩溃，
    /// 即所有资源都被重置。但在测试器中我们模拟的是虚拟崩溃，
    /// 因此请注意清理由此 Raft 节点生成的后台线程。
    pub fn kill(&self) {
        // 如果需要，可以在这里添加代码。
    }

    /// 服务希望切换到快照。
    ///
    /// 仅在 Raft 没有更新信息的情况下执行此操作，
    /// 因为它已通过 `apply_ch` 通信了快照。
    pub fn cond_install_snapshot(
        &self,
        last_included_term: u64,
        last_included_index: u64,
        snapshot: &[u8],
    ) -> bool {
        // 你的代码写在这里。
        // 示例：
        // self.raft.cond_install_snapshot(last_included_term, last_included_index, snapshot)
        crate::your_code_here((last_included_term, last_included_index, snapshot));
    }

    /// 服务表示它已创建一个快照，其中包含截至并包括指定索引的所有信息。
    /// 这意味着服务不再需要该索引（含）之前的日志。
    /// Raft 现在应尽可能裁剪其日志。
    pub fn snapshot(&self, index: u64, snapshot: &[u8]) {
        // 你的代码写在这里。
        // 示例：
        // self.raft.snapshot(index, snapshot)
        crate::your_code_here((index, snapshot));
    }
}

#[async_trait::async_trait]
impl RaftService for Node {
    // RequestVote RPC 处理器的示例。
    //
    // 注意：请避免在此处加锁或休眠，否则可能会阻塞网络。
    async fn request_vote(&self, args: RequestVoteArgs) -> labrpc::Result<RequestVoteReply> {
        // 你的代码写在这里（2A, 2B）。
        crate::your_code_here(args)
    }
}
```