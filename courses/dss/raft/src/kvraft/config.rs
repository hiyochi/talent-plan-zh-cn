```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rand::seq::SliceRandom;

use crate::kvraft::errors::{Error, Result};
use crate::kvraft::{client, server};
use crate::proto::kvraftpb::*;
use crate::proto::raftpb::*;
use crate::raft;
use crate::raft::persister::*;

static ID: AtomicUsize = AtomicUsize::new(300_000);

fn uniqstring() -> String {
    format!("{}", ID.fetch_add(1, Ordering::Relaxed))
}

struct Servers {
    kvservers: Vec<Option<server::Node>>,
    saved: Vec<Arc<SimplePersister>>,
    endnames: Vec<Vec<String>>,
}

fn init_logger() {
    use std::sync::Once;
    static LOGGER_INIT: Once = Once::new();
    LOGGER_INIT.call_once(env_logger::init);
}

pub struct Config {
    pub net: labrpc::Network,
    pub n: usize,
    servers: Mutex<Servers>,
    clerks: Mutex<HashMap<String, Vec<String>>>,
    next_client_id: AtomicUsize,
    maxraftstate: Option<usize>,

    // 配置创建时的时间。
    start: Instant,

    // begin()/end() 统计信息
    // test_test.go 调用 cfg.begin() 时的时间
    t0: Mutex<Instant>,
    // 测试开始时的 rpc_total()
    rpcs0: AtomicUsize,
    // 协议数量
    ops: AtomicUsize,
}

impl Config {
    pub fn new(n: usize, unreliable: bool, maxraftstate: Option<usize>) -> Config {
        init_logger();

        let servers = Servers {
            kvservers: vec![None; n],
            saved: (0..n).map(|_| Arc::new(SimplePersister::new())).collect(),
            endnames: vec![vec![String::new(); n]; n],
        };
        let cfg = Config {
            n,
            net: labrpc::Network::new(),
            servers: Mutex::new(servers),
            clerks: Mutex::new(HashMap::new()),
            // 客户端 ID 从比最高服务器 ID 大 1000 开始，
            next_client_id: AtomicUsize::new(n + 1000),
            maxraftstate,
            start: Instant::now(),
            t0: Mutex::new(Instant::now()),
            rpcs0: AtomicUsize::new(0),
            ops: AtomicUsize::new(0),
        };

        // 创建一组完整的 KV 服务器。
        for i in 0..cfg.n {
            cfg.start_server(i);
        }

        cfg.connect_all();

        cfg.net.set_reliable(!unreliable);

        cfg
    }

    pub fn op(&self) {
        self.ops.fetch_add(1, Ordering::Relaxed);
    }

    fn rpc_total(&self) -> usize {
        self.net.total_count()
    }

    pub fn check_timeout(&self) {
        // 强制每个测试最多运行两分钟（实时时间）
        if self.start.elapsed() > Duration::from_secs(120) {
            panic!("test took longer than 120 seconds");
        }
    }

    /// 所有服务器中的最大日志大小
    pub fn log_size(&self) -> usize {
        let servers = self.servers.lock().unwrap();
        let mut logsize = 0;
        for save in &servers.saved {
            let n = save.raft_state().len();
            if n > logsize {
                logsize = n;
            }
        }
        logsize
    }

    /// 所有服务器中的最大快照大小
    pub fn snapshot_size(&self) -> usize {
        let mut snapshotsize = 0;
        let servers = self.servers.lock().unwrap();
        for save in &servers.saved {
            let n = save.snapshot().len();
            if n > snapshotsize {
                snapshotsize = n;
            }
        }
        snapshotsize
    }

    /// 将服务器 i 连接到 to 中列出的服务器
    fn connect(&self, i: usize, to: &[usize], servers: &Servers) {
        debug!("connect peer {} to {:?}", i, to);
        // 出站套接字文件
        for j in to {
            let endname = &servers.endnames[i][*j];
            self.net.enable(endname, true);
        }

        // 入站套接字文件
        for j in to {
            let endname = &servers.endnames[*j][i];
            self.net.enable(endname, true);
        }
    }

    /// 将服务器 i 与 from 中列出的服务器断开连接
    fn disconnect(&self, i: usize, from: &[usize], servers: &Servers) {
        debug!("disconnect peer {} from {:?}", i, from);
        // 出站套接字文件
        for j in from {
            if !servers.endnames[i].is_empty() {
                let endname = &servers.endnames[i][*j];
                self.net.enable(endname, false);
            }
        }

        // 入站套接字文件
        for j in from {
            if !servers.endnames[*j].is_empty() {
                let endname = &servers.endnames[*j][i];
                self.net.enable(endname, false);
            }
        }
    }

    pub fn all(&self) -> Vec<usize> {
        (0..self.n).collect()
    }

    pub fn connect_all(&self) {
        let servers = self.servers.lock().unwrap();
        for i in 0..self.n {
            self.connect(i, &self.all(), &*servers);
        }
    }

    /// 设置两个分区，每个分区内的服务器之间保持连接。
    pub fn partition(&self, p1: &[usize], p2: &[usize]) {
        debug!("partition servers into: {:?} {:?}", p1, p2);
        let servers = self.servers.lock().unwrap();
        for i in p1 {
            self.disconnect(*i, p2, &*servers);
            self.connect(*i, p1, &*servers);
        }
        for i in p2 {
            self.disconnect(*i, p1, &*servers);
            self.connect(*i, p2, &*servers);
        }
    }

    // 创建一个 clerk，并为其分配特定的服务器名称。
    // 让它与所有服务器建立连接，但目前只启用与 to[] 中服务器的连接。
    pub fn make_client(&self, to: &[usize]) -> client::Clerk {
        // 一组新的 ClientEnd。
        let mut ends = Vec::with_capacity(self.n);
        let mut endnames = Vec::with_capacity(self.n);
        for j in 0..self.n {
            let name = uniqstring();
            endnames.push(name.clone());
            let cli = self.net.create_client(name.clone());
            ends.push(KvClient::new(cli));
            self.net.connect(&name, &format!("{}", j));
        }

        ends.shuffle(&mut rand::thread_rng());
        let ck_name = uniqstring();
        let ck = client::Clerk::new(ck_name.clone(), ends);
        self.clerks.lock().unwrap().insert(ck_name, endnames);
        self.next_client_id.fetch_add(1, Ordering::Relaxed);
        self.connect_client(&ck, to);
        ck
    }

    pub fn delete_client(&self, ck: &client::Clerk) {
        self.clerks.lock().unwrap().remove(&ck.name);
    }

    pub fn connect_client(&self, ck: &client::Clerk, to: &[usize]) {
        self.connect_client_by_name(&ck.name, to);
    }

    pub fn connect_client_by_name(&self, ck_name: &str, to: &[usize]) {
        debug!("connect_client {:?} to {:?}", ck_name, to);
        let clerks = self.clerks.lock().unwrap();
        let endnames = &clerks[ck_name];
        for j in to {
            let s = &endnames[*j];
            self.net.enable(s, true);
        }
    }

    /// 通过隔离来关闭服务器
    pub fn shutdown_server(&self, i: usize) {
        let mut servers = self.servers.lock().unwrap();
        self.disconnect(i, &self.all(), &*servers);

        // 禁用客户端到该服务器的连接。
        // 在创建 saved[i] 中的新 Persister 之前执行此操作非常重要，
        // 以避免服务器对 Append 返回肯定回复，
        // 但将结果持久化到已废弃的 Persister 中。
        self.net.delete_server(&format!("{}", i));

        // 使用新的 Persister，以防旧实例
        // 继续更新 Persister。
        // 但复制旧 Persister 的内容，以便我们始终
        // 将 Make() 传递给最后一次持久化的状态。
        let p = raft::persister::SimplePersister::new();
        p.save_state_and_snapshot(servers.saved[i].raft_state(), servers.saved[i].snapshot());
        servers.saved[i] = Arc::new(p);

        if let Some(kv) = servers.kvservers[i].take() {
            kv.kill();
        }
    }

    /// 启动服务器 i。
    /// 如果要重启服务器，请先调用 shutdown_server
    pub fn start_server(&self, i: usize) {
        // 一组新的出站 ClientEnd 名称。
        let mut servers = self.servers.lock().unwrap();
        servers.endnames[i] = (0..self.n).map(|_| uniqstring()).collect();

        // 一组新的 ClientEnd。
        let mut ends = Vec::with_capacity(self.n);
        for (j, name) in servers.endnames[i].iter().enumerate() {
            let cli = self.net.create_client(name.clone());
            ends.push(RaftClient::new(cli));
            self.net.connect(name, &format!("{}", j));
        }

        // 使用新的 Persister，这样旧实例就不会覆盖
        // 新实例的持久化状态。
        // 给新的 Persister 一个旧 Persister 的副本，
        // 以便规范是我们将最后一次持久化的状态传递给 StartKVServer()。
        let sp = raft::persister::SimplePersister::new();
        sp.save_state_and_snapshot(servers.saved[i].raft_state(), servers.saved[i].snapshot());
        let p = Arc::new(sp);
        servers.saved[i] = p.clone();

        let kv = server::KvServer::new(ends, i, Box::new(p), self.maxraftstate);
        let rf_node = kv.rf.clone();
        let kv_node = server::Node::new(kv);
        servers.kvservers[i] = Some(kv_node.clone());

        let mut builder = labrpc::ServerBuilder::new(format!("{}", i));
        add_raft_service(rf_node, &mut builder).unwrap();
        add_kv_service(kv_node, &mut builder).unwrap();
        let srv = builder.build();
        self.net.add_server(srv);
    }

    pub fn leader(&self) -> Result<usize> {
        let servers = self.servers.lock().unwrap();
        for (i, kv) in servers.kvservers.iter().enumerate() {
            if let Some(kv) = kv {
                if kv.is_leader() {
                    return Ok(i);
                }
            }
        }
        Err(Error::NoLeader)
    }

    /// 将服务器划分为两组，并将当前 leader 放入少数派
    pub fn make_partition(&self) -> (Vec<usize>, Vec<usize>) {
        let l = self.leader().unwrap_or(0);
        let mut p1 = Vec::with_capacity(self.n / 2 + 1);
        let mut p2 = Vec::with_capacity(self.n / 2);
        for i in 0..self.n {
            if i != l {
                if p1.len() < self.n / 2 + 1 {
                    p1.push(i);
                } else {
                    p2.push(i);
                }
            }
        }
        p2.push(l);
        (p1, p2)
    }

    /// 开始一个测试。
    /// 打印测试消息。
    /// 例如：cfg.begin("Test (2B): RPC counts aren't too high")
    pub fn begin(&self, description: &str) {
        println!(); // 强制日志从新行开始。
        info!("{} ...", description);
        *self.t0.lock().unwrap() = Instant::now();
        self.rpcs0.store(self.rpc_total(), Ordering::Relaxed);
        self.ops.store(0, Ordering::Relaxed);
    }

    /// 结束一个测试——我们能到达这里意味着没有发生故障。
    /// 打印通过消息，
    /// 以及一些性能数据。
    pub fn end(&self) {
        self.check_timeout();

        // 实际时间
        let t = self.t0.lock().unwrap().elapsed();
        // Raft 对等体数量
        let npeers = self.n;
        // RPC 发送次数
        let nrpc = self.rpc_total() - self.rpcs0.load(Ordering::Relaxed);
        // clerk get/put/append 调用次数
        let nops = self.ops.load(Ordering::Relaxed);

        info!("  ... Passed --");
        info!("  {:?}  {} {} {}", t, npeers, nrpc, nops);
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        let servers = self.servers.lock().unwrap();
        for s in servers.kvservers.iter().flatten() {
            s.kill();
        }
    }
}
```