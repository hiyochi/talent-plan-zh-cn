```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use futures::channel::mpsc::unbounded;
use futures::future;
use futures::stream::StreamExt;
use rand::Rng;

use crate::proto::raftpb::*;
use crate::raft;
use crate::raft::persister::*;

pub const SNAPSHOT_INTERVAL: u64 = 10;

fn uniqstring() -> String {
    static ID: AtomicUsize = AtomicUsize::new(0);
    format!("{}", ID.fetch_add(1, Ordering::Relaxed))
}

/// 日志条目。
#[derive(Clone, PartialEq, Message)]
pub struct Entry {
    #[prost(uint64, tag = "100")]
    pub x: u64,
}

pub struct Storage {
    // 每个服务器已提交条目的副本
    logs: Vec<HashMap<u64, Entry>>,
    max_index: u64,
    max_index0: u64,
}

impl Storage {
    /// 有多少服务器认为某个日志条目已提交？
    pub fn n_committed(&self, index: u64) -> (usize, Option<Entry>) {
        let mut count = 0;
        let mut cmd = None;
        for log in &self.logs {
            let cmd1 = log.get(&index).cloned();
            if cmd1.is_some() {
                if count > 0 && cmd != cmd1 {
                    panic!(
                        "提交的值不匹配: 索引 {:?}, {:?}, {:?}",
                        index, cmd, cmd1
                    );
                }
                count += 1;
                cmd = cmd1;
            }
        }
        (count, cmd)
    }
}

fn init_logger() {
    use std::sync::Once;
    static LOGGER_INIT: Once = Once::new();
    LOGGER_INIT.call_once(env_logger::init);
}

pub struct Config {
    pub net: labrpc::Network,
    n: usize,
    // 使用 boxed slice 防止容量增长。
    pub rafts: Arc<Mutex<Box<[Option<raft::Node>]>>>,
    // 每个服务器是否连接到网络
    pub connected: Box<[bool]>,
    saved: Box<[Arc<SimplePersister>]>,
    // 每个服务器发送到的端口文件名
    endnames: Box<[Box<[String]>]>,

    pub storage: Arc<Mutex<Storage>>,

    // make_config() 被调用的时间
    start: Instant,

    // begin()/end() 统计信息

    // test_test.go 调用 cfg.begin() 的时间
    t0: Instant,
    // 测试开始时的 rpc_total()
    rpcs0: usize,
    // 协议数量
    cmds0: usize,
}

impl Config {
    pub fn new(n: usize) -> Config {
        Config::new_with(n, false, false)
    }

    pub fn new_with(n: usize, unreliable: bool, snapshot: bool) -> Config {
        init_logger();

        let net = labrpc::Network::new();
        net.set_reliable(!unreliable);
        net.set_long_delays(true);
        let storage = Storage {
            logs: vec![HashMap::new(); n],
            max_index: 0,
            max_index0: 0,
        };
        let mut saved = vec![];
        let mut endnames = vec![];
        for _ in 0..n {
            endnames.push(vec![String::new(); n].into_boxed_slice());
            saved.push(Arc::new(SimplePersister::new()));
        }
        let mut cfg = Config {
            net,
            n,
            rafts: Arc::new(Mutex::new(vec![None; n].into_boxed_slice())),
            connected: vec![true; n].into_boxed_slice(),
            saved: saved.into_boxed_slice(),
            endnames: endnames.into_boxed_slice(),
            storage: Arc::new(Mutex::new(storage)),

            start: Instant::now(),
            t0: Instant::now(),
            rpcs0: 0,
            cmds0: 0,
        };

        for i in 0..n {
            cfg.start1_ext(i, snapshot);
        }

        for i in 0..n {
            cfg.connect(i);
        }

        cfg
    }

    pub fn rpc_count(&self, server: usize) -> usize {
        self.net.count(&format!("{}", server))
    }

    fn rpc_total(&self) -> usize {
        self.net.total_count()
    }

    /// 所有服务器中最大的日志大小
    pub fn log_size(&self) -> usize {
        self.saved
            .iter()
            .map(|s| s.raft_state().len())
            .max()
            .unwrap()
    }

    // 检查是否恰好有一个领导者。
    // 尝试几次，以防需要重新选举。
    pub fn check_one_leader(&self) -> usize {
        let mut random = rand::thread_rng();
        let mut leaders = HashMap::new();
        for _iters in 0..10 {
            let ms = 450 + (random.gen::<u64>() % 100);
            thread::sleep(Duration::from_millis(ms));

            for (i, connected) in self.connected.iter().enumerate() {
                if *connected {
                    let state = self.rafts.lock().unwrap()[i]
                        .as_ref()
                        .unwrap()
                        .get_state()
                        .clone();
                    let term = state.term();
                    let is_leader = state.is_leader();
                    if is_leader {
                        leaders.entry(term).or_insert_with(Vec::new).push(i);
                    }
                }
            }

            let mut last_term_with_leader = 0;
            for (term, leaders) in &leaders {
                if leaders.len() > 1 {
                    panic!("任期 {} 有 {:?} (>1) 个领导者", term, leaders);
                }
                if *term > last_term_with_leader {
                    last_term_with_leader = *term;
                }
            }

            if !leaders.is_empty() {
                return leaders[&last_term_with_leader][0];
            }
        }

        panic!("期望有一个领导者，但没有找到")
    }

    /// 检查所有人是否同意任期。
    pub fn check_terms(&self) -> u64 {
        let mut term = 0;
        for (i, connected) in self.connected.iter().enumerate() {
            if *connected {
                let xterm = self.rafts.lock().unwrap()[i].as_ref().unwrap().term();
                if term == 0 {
                    term = xterm;
                } else if term != xterm {
                    panic!("服务器对任期不一致");
                }
            }
        }
        term
    }

    /// 检查是否没有领导者
    pub fn check_no_leader(&self) {
        for (i, connected) in self.connected.iter().enumerate() {
            if *connected {
                let is_leader = self.rafts.lock().unwrap()[i].as_ref().unwrap().is_leader();
                if is_leader {
                    panic!("期望没有领导者，但 {} 声称是领导者", i);
                }
            }
        }
    }

    pub fn check_timeout(&self) {
        // 对每个测试强制执行两分钟实时限制
        if self.start.elapsed() > Duration::from_secs(120) {
            panic!("测试耗时超过 120 秒");
        }
    }

    /// 有多少服务器认为某个日志条目已提交？
    pub fn n_committed(&self, index: u64) -> (usize, Option<Entry>) {
        let s = self.storage.lock().unwrap();
        s.n_committed(index)
    }

    // 等待至少 n 个服务器提交。
    // 但不要永远等待。
    pub fn wait(&self, index: u64, n: usize, start_term: Option<u64>) -> Option<Entry> {
        let mut to = Duration::from_millis(10);
        for _ in 0..30 {
            let (nd, _) = self.n_committed(index);
            if nd >= n {
                break;
            }
            thread::sleep(to);
            if to < Duration::from_secs(1) {
                to *= 2;
            }
            if let Some(start_term) = start_term {
                let rafts = self.rafts.lock().unwrap();
                for r in rafts.iter().flatten() {
                    let term = r.term();
                    if term > start_term {
                        // 有人已经前进
                        // 无法再保证我们会“获胜”
                        return None;
                    }
                }
            }
        }
        let (nd, cmd) = self.n_committed(index);
        if nd < n {
            panic!("只有 {} 个服务器决定索引 {}；期望 {}", nd, index, n);
        }
        cmd
    }

    /// 执行一个完整的协议。
    /// 它可能最初选择了错误的领导者，
    /// 在放弃后必须重新提交。
    /// 在大约 10 秒后完全放弃。
    /// 间接检查服务器是否同意相同的值，
    /// 因为 n_committed() 会检查这一点，
    /// 读取 applyCh 的线程也会检查。
    /// 返回索引。
    /// 如果 retry==true，可能会多次提交命令，
    /// 以防领导者在 Start() 后立即失败。
    /// 如果 retry==false，只调用一次 start()，
    /// 以简化早期的 Lab 2B 测试。
    pub fn one(&self, cmd: Entry, expected_servers: usize, retry: bool) -> u64 {
        let t0 = Instant::now();
        let mut starts = 0;
        while t0.elapsed() < Duration::from_secs(10) {
            // 尝试所有服务器，也许其中一个是领导者。
            let mut index = None;
            for _ in 0..self.n {
                starts = (starts + 1) % self.n;
                if self.connected[starts] {
                    let rafts = self.rafts.lock().unwrap();
                    if let Some(ref rf) = &rafts[starts] {
                        match rf.start(&cmd) {
                            Ok((index1, _)) => {
                                index = Some(index1);
                                break;
                            }
                            Err(e) => debug!("start cmd {:?} 失败: {:?}", cmd, e),
                        }
                    }
                }
            }

            if let Some(index) = index {
                // 有人声称是领导者并已提交我们的命令；
                // 等待一段时间以达成一致。
                let t1 = Instant::now();
                while t1.elapsed() < Duration::from_secs(2) {
                    let (nd, cmd1) = self.n_committed(index);
                    if nd > 0 && nd >= expected_servers {
                        // 已提交
                        if let Some(cmd2) = cmd1 {
                            if cmd2 == cmd {
                                // 并且是我们提交的命令。
                                return index;
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(20));
                }
                if !retry {
                    panic!("one({:?}) 未能达成一致", cmd);
                }
            } else {
                thread::sleep(Duration::from_millis(50));
            }
        }
        panic!("one({:?}) 未能达成一致", cmd);
    }

    /// 开始一个测试。
    /// 打印测试消息。
    /// 例如：cfg.begin("Test (2B): RPC 计数不太高")
    pub fn begin(&mut self, description: &str) {
        println!(); // 强制日志从新行开始。
        info!("{} ...", description);
        self.t0 = Instant::now();
        self.rpcs0 = self.rpc_total();
        self.cmds0 = 0;

        let mut s = self.storage.lock().unwrap();
        s.max_index0 = s.max_index;
    }

    /// 结束一个测试——我们到达这里的事实意味着没有失败。
    /// 打印通过消息和一些性能数字。
    pub fn end(&self) {
        self.check_timeout();

        // 实时
        let t = self.t0.elapsed();
        // Raft 对等体数量
        let npeers = self.n;
        // RPC 发送数量
        let nrpc = self.rpc_total() - self.rpcs0;

        // 报告的 Raft 协议数量
        let s = self.storage.lock().unwrap();
        let ncmds = s.max_index - s.max_index0;

        info!("  ... 通过 --");
        info!("  {:?}  {} {} {}", t, npeers, nrpc, ncmds);
    }

    /// 启动或重新启动一个 Raft。
    /// 如果已存在，则先“杀死”它。
    /// 分配新的传出端口文件名和新的状态持久化器，
    /// 以隔离此服务器的先前实例。因为我们无法真正杀死它。
    pub fn start1(&mut self, i: usize) {
        self.start1_ext(i, false);
    }

    pub fn start1_snapshot(&mut self, i: usize) {
        self.start1_ext(i, true);
    }

    fn start1_ext(&mut self, i: usize, snapshot: bool) {
        self.crash1(i);

        // 一组新的传出 ClientEnd 名称。
        // 以便旧的崩溃实例的 ClientEnd 无法发送。
        self.endnames[i] = vec![String::new(); self.n].into_boxed_slice();
        for j in 0..self.n {
            self.endnames[i][j] = uniqstring();
        }

        // 一组新的 ClientEnd。
        let mut clients = Vec::with_capacity(self.n);
        for (j, name) in self.endnames[i].iter().enumerate() {
            let cli = self.net.create_client(name.to_string());
            let client = RaftClient::new(cli);
            clients.push(client);
            self.net.connect(name, &format!("{}", j));
        }

        let (tx, apply_ch) = unbounded();
        let rf = raft::Raft::new(clients, i, Box::new(self.saved[i].clone()), tx);
        let node = raft::Node::new(rf);
        self.rafts.lock().unwrap()[i] = Some(node.clone());

        // 监听来自 Raft 的消息，指示新提交的条目。
        let storage = self.storage.clone();
        let rafts = self.rafts.clone();
        let apply = apply_ch.for_each(move |cmd: raft::ApplyMsg| match cmd {
            raft::ApplyMsg::Command { data, index } => {
                // debug!("apply {}", index);
                let entry = labcodec::decode(&data).expect("提交的命令不是条目");
                let mut s = storage.lock().unwrap();
                for (j, log) in s.logs.iter().enumerate() {
                    if let Some(old) = log.get(&index) {
                        if *old != entry {
                            // 某个服务器已经为此条目提交了不同的值！
                            panic!(
                                "提交 索引={:?} 服务器={:?} {:?} != 服务器={:?} {:?}",
                                index, i, entry, j, old
                            );
                        }
                    }
                }
                let log = &mut s.logs[i];
                if index > 1 && log.get(&(index - 1)).is_none() {
                    panic!("服务器 {} 应用顺序错误 {}", i, index);
                }
                log.insert(index, entry);
                if index > s.max_index {
                    s.max_index = index;
                }
                if snapshot && (index + 1) % SNAPSHOT_INTERVAL == 0 {
                    rafts.lock().unwrap()[i]
                        .as_ref()
                        .unwrap()
                        .snapshot(index, &data);
                }
                future::ready(())
            }
            raft::ApplyMsg::Snapshot { data, index, term } if snapshot => {
                // debug!("安装快照 {}", index);
                if rafts.lock().unwrap()[i]
                    .as_ref()
                    .unwrap()
                    .cond_install_snapshot(term, index, &data)
                {
                    let mut s = storage.lock().unwrap();
                    let log = &mut s.logs[i];
                    log.clear();
                    let entry = labcodec::decode(&data).unwrap();
                    log.insert(index, entry);
                }
                future::ready(())
            }
            // 忽略其他类型的 ApplyMsg
            _ => future::ready(()),
        });
        self.net.spawn_poller(apply);

        let mut builder = labrpc::ServerBuilder::new(format!("{}", i));
        raft::add_raft_service(node, &mut builder).unwrap();
        let srv = builder.build();
        self.net.add_server(srv);
    }

    /// 关闭一个 Raft 服务器，但保存其持久状态。
    pub fn crash1(&mut self, i: usize) {
        self.disconnect(i);
        // 禁用与服务器的客户端连接。
        self.net.delete_server(&format!("{}", i));

        // 一个新的持久化器，以防旧实例继续更新持久化器。
        // 但复制旧持久化器的内容，以便我们总是将上次持久化的状态传递给 Make()。
        let raft_state = self.saved[i].raft_state();
        let snapshot = self.saved[i].snapshot();
        let p = SimplePersister::new();
        p.save_state_and_snapshot(raft_state, snapshot);
        self.saved[i] = Arc::new(p);

        if let Some(rf) = self.rafts.lock().unwrap()[i].take() {
            rf.kill();
        }
    }

    /// 将服务器 i 从网络分离。
    pub fn disconnect(&mut self, i: usize) {
        debug!("disconnect({})", i);

        self.connected[i] = false;

        // 传出的 ClientEnd
        for endname in &*self.endnames[i] {
            self.net.enable(endname, false);
        }

        // 传入的 ClientEnd
        for names in &*self.endnames {
            let endname = &names[i];
            self.net.enable(endname, false);
        }
    }

    /// 将服务器 i 连接到网络。
    pub fn connect(&mut self, i: usize) {
        debug!("connect({})", i);

        self.connected[i] = true;

        // 传出的 ClientEnd
        for (j, connected) in self.connected.iter().enumerate() {
            if *connected {
                let endname = &*self.endnames[i][j];
                self.net.enable(endname, true);
            }
        }

        // 传入的 ClientEnd
        for (j, connected) in self.connected.iter().enumerate() {
            if *connected {
                let endname = &*self.endnames[j][i];
                self.net.enable(endname, true);
            }
        }
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        if let Ok(rafts) = self.rafts.try_lock() {
            for r in rafts.iter().flatten() {
                r.kill();
            }
        }

        // FIXME: 我们不应该在 drop 方法中 panic。
        self.check_timeout();
    }
}
```