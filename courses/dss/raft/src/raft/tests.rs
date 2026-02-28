#![allow(clippy::identity_op)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use futures::channel::oneshot;
use futures::executor::block_on;
use futures::future;
use rand::{rngs::ThreadRng, Rng};

use crate::raft::config::{Config, Entry, Storage, SNAPSHOT_INTERVAL};
use crate::raft::Node;

/// 测试者允许解决方案在一秒钟内完成选举
/// （远远超过论文中规定的超时时间范围）。
const RAFT_ELECTION_TIMEOUT: Duration = Duration::from_millis(1000);

fn random_entry(rnd: &mut ThreadRng) -> Entry {
    Entry {
        x: rnd.gen::<u64>(),
    }
}

#[test]
fn test_initial_election_2a() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2A): initial election");

    // 是否选出了领导者？
    cfg.check_one_leader();

    // 稍微休眠一下，避免与跟随者学习选举结果产生竞争，
    // 然后检查所有节点是否对任期达成一致。
    thread::sleep(Duration::from_millis(50));
    let term1 = cfg.check_terms();

    // 如果没有网络故障，领导者和任期是否保持不变？
    thread::sleep(2 * RAFT_ELECTION_TIMEOUT);
    let term2 = cfg.check_terms();
    if term1 != term2 {
        warn!("warning: term changed even though there were no failures")
    }

    // 应该仍然存在一个领导者。
    cfg.check_one_leader();

    cfg.end();
}

#[test]
fn test_reelection_2a() {
    let servers = 3;
    let mut cfg = Config::new(servers);
    cfg.begin("Test (2A): election after network failure");

    let leader1 = cfg.check_one_leader();
    // 如果领导者断开连接，应该选出新的领导者。
    cfg.disconnect(leader1);
    cfg.check_one_leader();

    // 如果旧领导者重新加入，不应
    // 干扰新领导者。
    cfg.connect(leader1);
    let leader2 = cfg.check_one_leader();

    // 如果没有法定人数，不应
    // 选出领导者。
    cfg.disconnect(leader2);
    cfg.disconnect((leader2 + 1) % servers);
    thread::sleep(2 * RAFT_ELECTION_TIMEOUT);
    cfg.check_no_leader();

    // 如果形成法定人数，应该选出领导者。
    cfg.connect((leader2 + 1) % servers);
    cfg.check_one_leader();

    // 最后一个节点重新加入不应阻止领导者存在。
    cfg.connect(leader2);
    cfg.check_one_leader();

    cfg.end();
}

#[test]
fn test_many_election_2a() {
    let servers = 7;
    let iters = 10;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2A): multiple elections");

    cfg.check_one_leader();

    let mut random = rand::thread_rng();
    for _ in 0..iters {
        // 断开三个节点
        let i1 = random.gen::<usize>() % servers;
        let i2 = random.gen::<usize>() % servers;
        let i3 = random.gen::<usize>() % servers;
        cfg.disconnect(i1);
        cfg.disconnect(i2);
        cfg.disconnect(i3);

        // 要么当前领导者仍然存活，
        // 要么剩下的四个节点应选出新的领导者。
        cfg.check_one_leader();

        cfg.connect(i1);
        cfg.connect(i2);
        cfg.connect(i3);
    }

    cfg.check_one_leader();

    cfg.end();
}

#[test]
fn test_basic_agree_2b() {
    let servers = 5;
    let mut cfg = Config::new(servers);
    cfg.begin("Test (2B): basic agreement");

    let iters = 3;
    for index in 1..=iters {
        let (nd, _) = cfg.n_committed(index);
        if nd > 0 {
            panic!("some have committed before start()");
        }

        let xindex = cfg.one(Entry { x: index * 100 }, servers, false);
        if xindex != index {
            panic!("got index {} but expected {}", xindex, index);
        }
    }

    cfg.end()
}

#[test]
fn test_fail_agree_2b() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2B): agreement despite follower disconnection");

    cfg.one(Entry { x: 101 }, servers, false);

    // 跟随者网络断开
    let leader = cfg.check_one_leader();
    cfg.disconnect((leader + 1) % servers);

    // 尽管有一个服务器断开连接，仍能达成一致？
    cfg.one(Entry { x: 102 }, servers - 1, false);
    cfg.one(Entry { x: 103 }, servers - 1, false);
    thread::sleep(RAFT_ELECTION_TIMEOUT);
    cfg.one(Entry { x: 104 }, servers - 1, false);
    cfg.one(Entry { x: 105 }, servers - 1, false);

    // 重新连接
    cfg.connect((leader + 1) % servers);

    // 与全部服务器集达成一致？
    cfg.one(Entry { x: 106 }, servers, true);
    thread::sleep(RAFT_ELECTION_TIMEOUT);
    cfg.one(Entry { x: 107 }, servers, true);

    cfg.end();
}

#[test]
fn test_fail_no_agree_2b() {
    let servers = 5;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2B): no agreement if too many followers disconnect");

    cfg.one(Entry { x: 10 }, servers, false);

    // 5个跟随者中有3个断开连接
    let leader = cfg.check_one_leader();
    cfg.disconnect((leader + 1) % servers);
    cfg.disconnect((leader + 2) % servers);
    cfg.disconnect((leader + 3) % servers);
    let (index, _) = cfg.rafts.lock().unwrap()[leader]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 20 })
        .expect("leader rejected start");
    if index != 2 {
        panic!("expected index 2, got {}", index);
    }

    thread::sleep(2 * RAFT_ELECTION_TIMEOUT);

    let (n, _) = cfg.n_committed(index);
    if n > 0 {
        panic!("{} committed but no majority", n);
    }

    // 修复
    cfg.connect((leader + 1) % servers);
    cfg.connect((leader + 2) % servers);
    cfg.connect((leader + 3) % servers);

    // 断开连接的多数派可能已从他们自己中选出领导者，
    // 忘记了索引2。
    let leader2 = cfg.check_one_leader();
    let (index2, _) = cfg.rafts.lock().unwrap()[leader2]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 30 })
        .expect("leader2 rejected start");
    if !(2..=3).contains(&index2) {
        panic!("unexpected index {}", index2);
    }

    cfg.one(Entry { x: 1000 }, servers, true);

    cfg.end();
}

#[test]
fn test_concurrent_starts_2b() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2B): concurrent start()s");
    let mut success = false;
    'outer: for tried in 0..5 {
        if tried > 0 {
            // 给解决方案一些时间稳定下来
            thread::sleep(Duration::from_secs(3));
        }

        let leader = cfg.check_one_leader();
        let term = match cfg.rafts.lock().unwrap()[leader]
            .as_ref()
            .unwrap()
            .start(&Entry { x: 1 })
        {
            Err(err) => {
                warn!("start leader {} meet error {:?}", leader, err);
                continue;
            }
            Ok((_, term)) => term,
        };

        let mut idx_rxs = vec![];
        for ii in 0..5 {
            let (tx, rx) = oneshot::channel();
            idx_rxs.push(rx);
            let node = cfg.rafts.lock().unwrap()[leader].clone().unwrap();
            cfg.net.spawn(future::lazy(move |_| {
                let idx = match node.start(&Entry { x: 100 + ii }) {
                    Err(err) => {
                        warn!("start leader {} meet error {:?}", leader, err);
                        None
                    }
                    Ok((idx, term1)) => {
                        if term1 != term {
                            None
                        } else {
                            Some(idx)
                        }
                    }
                };
                tx.send(idx)
                    .map_err(|e| panic!("send failed: {:?}", e))
                    .unwrap();
            }));
        }
        let idxes = block_on(async {
            future::join_all(idx_rxs)
                .await
                .into_iter()
                .map(|idx_rx| idx_rx.unwrap())
                .collect::<Vec<_>>()
        });

        for j in 0..servers {
            let t = cfg.rafts.lock().unwrap()[j].as_ref().unwrap().term();
            if t != term {
                // 任期已改变——不能期望RPC数量较低
                continue 'outer;
            }
        }

        let mut cmds = vec![];
        for index in idxes.into_iter().flatten() {
            if let Some(cmd) = cfg.wait(index, servers, Some(term)) {
                cmds.push(cmd.x);
            } else {
                // 节点已进入后续任期
                // 因此不能期望所有Start()都成功
                continue;
            }
        }

        for ii in 0..5 {
            let x = 100 + ii;
            let mut ok = false;
            for cmd in &cmds {
                if *cmd == x {
                    ok = true;
                }
            }
            assert!(ok, "cmd {} missing in {:?}", x, cmds)
        }

        success = true;
        break;
    }

    assert!(success, "term changed too often");

    cfg.end();
}

#[test]
fn test_rejoin_2b() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2B): rejoin of partitioned leader");

    cfg.one(Entry { x: 101 }, servers, true);

    // 领导者网络故障
    let leader1 = cfg.check_one_leader();
    cfg.disconnect(leader1);

    // 让旧领导者尝试就某些条目达成一致
    let _ = cfg.rafts.lock().unwrap()[leader1]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 102 });
    let _ = cfg.rafts.lock().unwrap()[leader1]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 103 });
    let _ = cfg.rafts.lock().unwrap()[leader1]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 104 });

    // 新领导者提交，同样适用于索引=2
    cfg.one(Entry { x: 103 }, 2, true);

    // 新领导者网络故障
    let leader2 = cfg.check_one_leader();
    cfg.disconnect(leader2);

    // 旧领导者重新连接
    cfg.connect(leader1);

    cfg.one(Entry { x: 104 }, 2, true);

    // 现在全部在一起
    cfg.connect(leader2);

    cfg.one(Entry { x: 105 }, servers, true);

    cfg.end();
}

#[test]
fn test_backup_2b() {
    let servers = 5;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2B): leader backs up quickly over incorrect follower logs");

    let mut random = rand::thread_rng();
    cfg.one(random_entry(&mut random), servers, true);

    // 将领导者和一个跟随者放入分区
    let leader1 = cfg.check_one_leader();
    cfg.disconnect((leader1 + 2) % servers);
    cfg.disconnect((leader1 + 3) % servers);
    cfg.disconnect((leader1 + 4) % servers);

    // 提交大量不会提交的命令
    for _i in 0..50 {
        let _ = cfg.rafts.lock().unwrap()[leader1]
            .as_ref()
            .unwrap()
            .start(&random_entry(&mut random));
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT / 2);

    cfg.disconnect((leader1 + 0) % servers);
    cfg.disconnect((leader1 + 1) % servers);

    // 允许其他分区恢复
    cfg.connect((leader1 + 2) % servers);
    cfg.connect((leader1 + 3) % servers);
    cfg.connect((leader1 + 4) % servers);

    // 向新组提交大量成功命令。
    for _i in 0..50 {
        cfg.one(random_entry(&mut random), 3, true);
    }

    // 现在另一个分区领导者及其一个跟随者
    let leader2 = cfg.check_one_leader();
    let mut other = (leader1 + 2) % servers;
    if leader2 == other {
        other = (leader2 + 1) % servers;
    }
    cfg.disconnect(other);

    // 更多不会提交的命令
    for _i in 0..50 {
        let _ = cfg.rafts.lock().unwrap()[leader2]
            .as_ref()
            .unwrap()
            .start(&random_entry(&mut random));
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT / 2);

    // 让原始领导者恢复活动，
    for i in 0..servers {
        cfg.disconnect(i);
    }
    cfg.connect((leader1 + 0) % servers);
    cfg.connect((leader1 + 1) % servers);
    cfg.connect(other);

    // 向新组提交大量成功命令。
    for _i in 0..50 {
        cfg.one(random_entry(&mut random), 3, true);
    }

    // 现在所有人
    for i in 0..servers {
        cfg.connect(i);
    }
    cfg.one(random_entry(&mut random), servers, true);

    cfg.end();
}

#[test]
fn test_count_2b() {
    const SERVERS: usize = 3;
    fn rpcs(cfg: &Config) -> usize {
        let mut n: usize = 0;
        for j in 0..SERVERS {
            n += cfg.rpc_count(j);
        }
        n
    }

    let mut cfg = Config::new(SERVERS);

    cfg.begin("Test (2B): RPC counts aren't too high");

    cfg.check_one_leader();
    let mut total1 = rpcs(&cfg);

    if !(1..=30).contains(&total1) {
        panic!("too many or few RPCs ({}) to elect initial leader", total1);
    }

    let mut total2 = 0;
    let mut success = false;
    'outer: for tried in 0..5 {
        if tried > 0 {
            // 给解决方案一些时间稳定下来
            thread::sleep(Duration::from_secs(3));
        }

        let leader = cfg.check_one_leader();
        total1 = rpcs(&cfg);

        let iters = 10;
        let (starti, term) = match cfg.rafts.lock().unwrap()[leader]
            .as_ref()
            .unwrap()
            .start(&Entry { x: 1 })
        {
            Ok((starti, term)) => (starti, term),
            Err(err) => {
                warn!("start leader {} meet error {:?}", leader, err);
                continue;
            }
        };

        let mut cmds = vec![];
        let mut random = rand::thread_rng();
        for i in 1..iters + 2 {
            let x = random.gen::<u64>();
            cmds.push(x);
            match cfg.rafts.lock().unwrap()[leader]
                .as_ref()
                .unwrap()
                .start(&Entry { x })
            {
                Ok((index1, term1)) => {
                    if term1 != term {
                        // 启动时任期已改变
                        continue 'outer;
                    }
                    if starti + i != index1 {
                        panic!("start failed");
                    }
                }
                Err(err) => {
                    warn!("start leader {} meet error {:?}", leader, err);
                    continue 'outer;
                }
            }
        }

        for i in 1..=iters {
            if let Some(ix) = cfg.wait(starti + i, SERVERS, Some(term)) {
                if ix.x != cmds[(i - 1) as usize] {
                    panic!(
                        "wrong value {:?} committed for index {}; expected {:?}",
                        ix,
                        starti + i,
                        cmds
                    );
                }
            }
        }

        let mut failed = false;
        total2 = 0;
        for j in 0..SERVERS {
            let t = cfg.rafts.lock().unwrap()[j].as_ref().unwrap().term();
            if t != term {
                // 任期已改变——不能期望RPC数量较低
                // 需要继续更新total2
                failed = true;
            }
            total2 += cfg.rpc_count(j);
        }

        if failed {
            continue 'outer;
        }

        if total2 - total1 > (iters as usize + 1 + 3) * 3 {
            panic!("too many RPCs ({}) for {} entries", total2 - total1, iters);
        }

        success = true;
        break;
    }

    if !success {
        panic!("term changed too often");
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT);

    let mut total3 = 0;
    for j in 0..SERVERS {
        total3 += cfg.rpc_count(j);
    }

    if total3 - total2 > 3 * 20 {
        panic!(
            "too many RPCs ({}) for 1 second of idleness",
            total3 - total2
        );
    }
    cfg.end();
}

#[test]
fn test_persist1_2c() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2C): basic persistence");

    cfg.one(Entry { x: 11 }, servers, true);

    // 崩溃并重启所有节点
    for i in 0..servers {
        cfg.start1(i);
    }
    for i in 0..servers {
        cfg.disconnect(i);
        cfg.connect(i);
    }

    cfg.one(Entry { x: 12 }, servers, true);

    let leader1 = cfg.check_one_leader();
    cfg.disconnect(leader1);
    cfg.start1(leader1);
    cfg.connect(leader1);

    cfg.one(Entry { x: 13 }, servers, true);

    let leader2 = cfg.check_one_leader();
    cfg.disconnect(leader2);
    cfg.one(Entry { x: 14 }, servers - 1, true);
    cfg.start1(leader2);
    cfg.connect(leader2);

    cfg.wait(4, servers, None); // 在杀死i3之前等待leader2加入

    let i3 = (cfg.check_one_leader() + 1) % servers;
    cfg.disconnect(i3);
    cfg.one(Entry { x: 15 }, servers - 1, true);
    cfg.start1(i3);
    cfg.connect(i3);

    cfg.one(Entry { x: 16 }, servers, true);

    cfg.end();
}

#[test]
fn test_persist2_2c() {
    let servers = 5;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2C): more persistence");

    let mut index = 1;
    for _ in 0..5 {
        cfg.one(Entry { x: 10 + index }, servers, true);
        index += 1;

        let leader1 = cfg.check_one_leader();

        cfg.disconnect((leader1 + 1) % servers);
        cfg.disconnect((leader1 + 2) % servers);

        cfg.one(Entry { x: 10 + index }, servers - 2, true);
        index += 1;

        cfg.disconnect((leader1 + 0) % servers);
        cfg.disconnect((leader1 + 3) % servers);
        cfg.disconnect((leader1 + 4) % servers);

        cfg.start1((leader1 + 1) % servers);
        cfg.start1((leader1 + 2) % servers);
        cfg.connect((leader1 + 1) % servers);
        cfg.connect((leader1 + 2) % servers);

        thread::sleep(RAFT_ELECTION_TIMEOUT);

        cfg.start1((leader1 + 3) % servers);
        cfg.connect((leader1 + 3) % servers);

        cfg.one(Entry { x: 10 + index }, servers - 2, true);
        index += 1;

        cfg.connect((leader1 + 4) % servers);
        cfg.connect((leader1 + 0) % servers);
    }

    cfg.one(Entry { x: 1000 }, servers, true);

    cfg.end();
}

#[test]
fn test_persist3_2c() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2C): partitioned leader and one follower crash, leader restarts");

    cfg.one(Entry { x: 101 }, 3, true);

    let leader = cfg.check_one_leader();
    cfg.disconnect((leader + 2) % servers);

    cfg.one(Entry { x: 102 }, 2, true);

    cfg.crash1((leader + 0) % servers);
    cfg.crash1((leader + 1) % servers);
    cfg.connect((leader + 2) % servers);
    cfg.start1((leader + 0) % servers);
    cfg.connect((leader + 0) % servers);

    cfg.one(Entry { x: 103 }, 2, true);

    cfg.start1((leader + 1) % servers);
    cfg.connect((leader + 1) % servers);

    cfg.one(Entry { x: 104 }, servers, true);

    cfg.end();
}

// 测试扩展Raft论文图8中描述的场景。每次迭代都会询问领导者（如果存在）
// 是否要在Raft日志中插入一条命令。如果存在领导者，该领导者很可能很快失败
// （可能未提交命令），或者稍后崩溃（很可能已提交命令）。
// 如果存活的服务器数量不足以形成多数派，可能需要启动新服务器。
// 新任期中的领导者可能会尝试完成复制尚未提交的日志条目。
#[test]
fn test_figure_8_2c() {
    let servers = 5;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2C): Figure 8");

    let mut random = rand::thread_rng();
    cfg.one(random_entry(&mut random), 1, true);

    let mut nup = servers;
    for _iters in 0..1000 {
        let mut leader = None;
        for i in 0..servers {
            let mut rafts = cfg.rafts.lock().unwrap();
            if let Some(Some(raft)) = rafts.get_mut(i) {
                if raft.start(&random_entry(&mut random)).is_ok() {
                    leader = Some(i);
                }
            }
        }

        if (random.gen::<usize>() % 1000) < 100 {
            let ms = random.gen::<u64>() % ((RAFT_ELECTION_TIMEOUT.as_millis() / 2) as u64);
            thread::sleep(Duration::from_millis(ms));
        } else {
            let ms = random.gen::<u64>() % 13;
            thread::sleep(Duration::from_millis(ms));
        }

        if let Some(leader) = leader {
            cfg.crash1(leader);
            nup -= 1;
        }

        if nup < 3 {
            let s = random.gen::<usize>() % servers;
            if cfg.rafts.lock().unwrap().get(s).unwrap().is_none() {
                cfg.start1(s);
                cfg.connect(s);
                nup += 1;
            }
        }
    }

    for i in 0..servers {
        if cfg.rafts.lock().unwrap().get(i).unwrap().is_none() {
            cfg.start1(i);
            cfg.connect(i);
        }
    }

    cfg.one(random_entry(&mut random), servers, true);

    cfg.end();
}

#[test]
fn test_unreliable_agree_2c() {
    let servers = 5;

    let cfg = {
        let mut cfg = Config::new_with(servers, true, false);
        cfg.begin("Test (2C): unreliable agreement");
        Arc::new(cfg)
    };

    let mut dones = vec![];
    for iters in 1..50 {
        for j in 0..4 {
            let c = cfg.clone();
            let (tx, rx) = oneshot::channel();
            thread::spawn(move || {
                c.one(
                    Entry {
                        x: (100 * iters) + j,
                    },
                    1,
                    true,
                );
                tx.send(()).map_err(|e| panic!("send failed: {:?}", e))
            });
            dones.push(rx);
        }
        cfg.one(Entry { x: iters }, 1, true);
    }

    cfg.net.set_reliable(true);

    block_on(async {
        future::join_all(dones)
            .await
            .into_iter()
            .for_each(|done| done.unwrap());
    });

    cfg.one(Entry { x: 100 }, servers, true);

    cfg.end();
}

#[test]
fn test_figure_8_unreliable_2c() {
    let servers = 5;
    let mut cfg = Config::new_with(servers, true, false);

    cfg.begin("Test (2C): Figure 8 (unreliable)");
    let mut random = rand::thread_rng();
    cfg.one(
        Entry {
            x: random.gen::<u64>() % 10000,
        },
        1,
        true,
    );

    let mut nup = servers;
    for iters in 0..1000 {
        if iters == 200 {
            cfg.net.set_long_reordering(true);
        }
        let mut leader = None;
        for i in 0..servers {
            if cfg.rafts.lock().unwrap()[i]
                .as_ref()
                .unwrap()
                .start(&Entry {
                    x: random.gen::<u64>() % 10000,
                })
                .is_ok()
                && cfg.connected[i]
            {
                leader = Some(i);
            }
        }

        if (random.gen::<usize>() % 1000) < 100 {
            let ms = random.gen::<u64>() % (RAFT_ELECTION_TIMEOUT.as_millis() as u64 / 2);
            thread::sleep(Duration::from_millis(ms as u64));
        } else {
            let ms = random.gen::<u64>() % 13;
            thread::sleep(Duration::from_millis(ms));
        }

        if let Some(leader) = leader {
            if (random.gen::<usize>() % 1000) < (RAFT_ELECTION_TIMEOUT.as_millis() as usize) / 2 {
                cfg.disconnect(leader);
                nup -= 1;
            }
        }

        if nup < 3 {
            let s = random.gen::<usize>() % servers;
            if !cfg.connected[s] {
                cfg.connect(s);
                nup += 1;
            }
        }
    }

    for i in 0..servers {
        if !cfg.connected[i] {
            cfg.connect(i);
        }
    }

    cfg.one(
        Entry {
            x: random.gen::<u64>() % 10000,
        },
        servers,
        true,
    );

    cfg.end();
}

fn internal_churn(unreliable: bool) {
    let servers = 5;
    let mut cfg = Config::new_with(servers, unreliable, false);
    if unreliable {
        cfg.begin("Test (2C): unreliable churn")
    } else {
        cfg.begin("Test (2C): churn")
    }

    let stop = Arc::new(AtomicUsize::new(0));

    // 创建并发客户端
    // TODO: 将其改为future
    fn cfn(
        me: usize,
        stop_clone: Arc<AtomicUsize>,
        tx: Sender<Option<Vec<u64>>>,
        rafts: Arc<Mutex<Box<[Option<Node>]>>>,
        storage: Arc<Mutex<Storage>>,
    ) {
        let mut values = vec![];
        while stop_clone.load(Ordering::SeqCst) == 0 {
            let mut random = rand::thread_rng();
            let x = random.gen::<u64>();
            let mut index: i64 = -1;
            let mut ok = false;
            // 尝试所有节点，可能其中一个是领导者
            let rafts: Vec<_> = rafts.lock().unwrap().iter().cloned().collect();
            for raft in &rafts {
                match raft {
                    Some(rf) => {
                        match rf.start(&Entry { x }) {
                            Ok((index1, _)) => {
                                index = index1 as i64;
                                ok = true;
                            }
                            Err(_) => continue,
                        };
                    }
                    None => continue,
                }
            }
            if ok {
                // 领导者可能提交我们的值，也可能不提交。
                // 但不要永远等待。
                for to in &[10, 20, 50, 100, 200] {
                    let (nd, cmd) = storage.lock().unwrap().n_committed(index as u64);
                    if nd > 0 {
                        match cmd {
                            Some(xx) => {
                                if xx.x == x {
                                    values.push(xx.x);
                                }
                            }
                            None => panic!("wrong command type"),
                        }
                        break;
                    }
                    thread::sleep(Duration::from_millis(*to));
                }
            } else {
                thread::sleep(Duration::from_millis((79 + me * 17) as u64));
            }
        }
        if !values.is_empty() {
            tx.send(Some(values)).unwrap();
        } else {
            tx.send(None).unwrap();
        }
    }

    let ncli = 3;
    let mut nrec = vec![];
    for i in 0..ncli {
        let stop_clone = stop.clone();
        let (tx, rx) = channel();
        let storage = cfg.storage.clone();
        let rafts = cfg.rafts.clone();
        thread::spawn(move || {
            cfn(i, stop_clone, tx, rafts, storage);
        });
        nrec.push(rx);
    }
    let mut random = rand::thread_rng();
    for _iters in 0..20 {
        if (random.gen::<usize>() % 1000) < 200 {
            let i = random.gen::<usize>() % servers;
            cfg.disconnect(i);
        }

        if (random.gen::<usize>() % 1000) < 500 {
            let i = random.gen::<usize>() % servers;
            if cfg.rafts.lock().unwrap().get(i).unwrap().is_none() {
                cfg.start1(i);
            }
            cfg.connect(i);
        }

        if (random.gen::<usize>() % 1000) < 200 {
            let i = random.gen::<usize>() % servers;
            if cfg.rafts.lock().unwrap().get(i).unwrap().is_some() {
                cfg.crash1(i);
            }
        }

        // 使崩溃/重启足够频繁，以便节点通常能跟上，
        // 但又不能太频繁，以免每次变化之间都已稳定下来。
        // 选择一个小于选举超时但又不小很多的值。
        thread::sleep((RAFT_ELECTION_TIMEOUT * 7) / 10)
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT);
    cfg.net.set_reliable(true);
    for i in 0..servers {
        if cfg.rafts.lock().unwrap().get(i).unwrap().is_none() {
            cfg.start1(i);
        }
        cfg.connect(i);
    }

    stop.store(1, Ordering::SeqCst);

    let mut values = vec![];
    for rx in &nrec {
        let mut vv = rx.recv().unwrap().unwrap();
        values.append(&mut vv);
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT);

    let last_index = cfg.one(random_entry(&mut random), servers, true);

    let mut really = vec![];
    for index in 1..=last_index {
        let v = cfg.wait(index, servers, None).unwrap();
        really.push(v.x);
    }

    for v1 in &values {
        let mut ok = false;
        for v2 in &really {
            if v1 == v2 {
                ok = true;
            }
        }
        assert!(ok, "didn't find a value");
    }

    cfg.end()
}

#[test]
fn test_reliable_churn_2c() {
    internal_churn(false);
}

#[test]
fn test_unreliable_churn_2c() {
    internal_churn(true);
}

fn snap_common(name: &str, disconnect: bool, reliable: bool, crash: bool) {
    const MAX_LOG_SIZE: usize = 2000;

    let iters = 30;
    let servers = 3;
    let mut cfg = Config::new_with(servers, !reliable, true);

    cfg.begin(name);

    let mut random = rand::thread_rng();
    cfg.one(random_entry(&mut random), servers, true);
    let mut leader1 = cfg.check_one_leader();

    for i in 0..iters {
        let mut victim = (leader1 + 1) % servers;
        let mut sender = leader1;
        if i % 3 == 1 {
            sender = (leader1 + 1) % servers;
            victim = leader1;
        }

        if disconnect {
            cfg.disconnect(victim);
            cfg.one(random_entry(&mut random), servers - 1, true);
        }
        if crash {
            cfg.crash1(victim);
            cfg.one(random_entry(&mut random), servers - 1, true);
        }
        // 发送足够多的条目以获得快照
        for _ in 0..=SNAPSHOT_INTERVAL {
            let _ = cfg.rafts.lock().unwrap()[sender]
                .as_ref()
                .unwrap()
                .start(&random_entry(&mut random));
        }
        // 让应用线程赶上Start()的进度
        cfg.one(random_entry(&mut random), servers - 1, true);

        assert!(cfg.log_size() < MAX_LOG_SIZE, "log size too large");

        if disconnect {
            // 重新连接跟随者，该跟随者可能落后，
            // 需要接收快照以赶上进度。
            cfg.connect(victim);
            cfg.one(random_entry(&mut random), servers, true);
            leader1 = cfg.check_one_leader();
        }
        if crash {
            cfg.start1_snapshot(victim);
            cfg.connect(victim);
            cfg.one(random_entry(&mut random), servers, true);
            leader1 = cfg.check_one_leader();
        }
    }
    cfg.end();
}

#[test]
fn test_snapshot_basic_2d() {
    snap_common("Test (2D): snapshots basic", false, true, false);
}

#[test]
fn test_snapshot_install_2d() {
    snap_common(
        "Test (2D): install snapshots (disconnect)",
        true,
        true,
        false,
    );
}

#[test]
fn test_snapshot_install_unreliable_2d() {
    snap_common(
        "Test (2D): install snapshots (disconnect+unreliable)",
        true,
        false,
        false,
    );
}

#[test]
fn test_snapshot_install_crash_2d() {
    snap_common("Test (2D): install snapshots (crash)", false, true, true);
}

#[test]
fn test_snapshot_install_unreliable_crash_2d() {
    snap_common(
        "Test (2D): install snapshots (unreliable+crash)",
        false,
        false,
        true,
    );
}