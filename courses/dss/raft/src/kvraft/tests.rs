```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Poll;
use std::thread;
use std::time::{Duration, Instant};

use futures::channel::oneshot;
use futures::executor::block_on;
use futures::future;
use futures::{Future, FutureExt};
use futures_timer::Delay;
use rand::{seq::SliceRandom, Rng};

use linearizability::check_operations_timeout;
use linearizability::model::Operation;
use linearizability::models::{KvInput, KvModel, KvOutput, Op};

use crate::kvraft::client::Clerk;
use crate::kvraft::config::Config;

/// 测试者允许解决方案在一秒内完成选举
/// （远超过论文中定义的超时范围）。
const RAFT_ELECTION_TIMEOUT: Duration = Duration::from_millis(1000);

const LINEARIZABILITY_CHECK_TIMEOUT: Duration = Duration::from_millis(1000);

// 执行 get/put/append 操作并统计次数
fn get(cfg: &Config, ck: &Clerk, key: &str) -> String {
    let v = ck.get(key.to_owned());
    cfg.op();
    v
}

fn put(cfg: &Config, ck: &Clerk, key: &str, value: &str) {
    ck.put(key.to_owned(), value.to_owned());
    cfg.op();
}

fn append(cfg: &Config, ck: &Clerk, key: &str, value: &str) {
    ck.append(key.to_owned(), value.to_owned());
    cfg.op();
}

fn check(cfg: &Config, ck: &Clerk, key: &str, value: &str) {
    let v = get(cfg, ck, key);
    if v != value {
        panic!("get({:?}): expected:\n{:?}\nreceived:\n{:?}", key, value, v);
    }
}

// 启动 ncli 个客户端并等待它们全部完成
fn spawn_clients_and_wait<Func, Fact>(
    cfg: Arc<Config>,
    ncli: usize,
    fact: Fact,
) -> impl Future<Output = ()> + Send + 'static
where
    Fact: Fn() -> Func + Send + 'static,
    Func: Fn(usize, &Clerk) + Send + 'static,
{
    let mut cas = Vec::with_capacity(ncli);
    for cli in 0..ncli {
        let (tx, rx) = oneshot::channel();
        cas.push(rx.map(move |_| {
            debug!("spawn_clients_and_wait: client {} is done", cli);
        }));

        let cfg_ = cfg.clone();
        // 客户端运行函数 func，然后通知已完成
        let func = fact();
        thread::spawn(move || {
            let ck = cfg_.make_client(&cfg_.all());
            func(cli, &ck);
            cfg_.delete_client(&ck);
            tx.send(())
        });
    }
    debug!("spawn_clients_and_wait: waiting for clients");
    future::join_all(cas).map(|_| ())
}

// 预测 append(k, val) 操作对旧值 prev 的影响
fn next_value(prev: String, val: &str) -> String {
    prev + val
}

// 检查特定客户端的所有已知 append 操作是否按顺序出现在值中
fn check_clnt_appends(clnt: usize, v: String, count: usize) {
    let mut lastoff = None;
    for j in 0..count {
        let wanted = format!("x {} {} y", clnt, j);
        if let Some(off) = v.find(&wanted) {
            let off1 = v.rfind(&wanted).unwrap();
            assert_eq!(off1, off, "duplicate element {:?} in Append result", wanted);

            if let Some(lastoff) = lastoff {
                assert!(
                    off > lastoff,
                    "wrong order for element {:?} in Append result",
                    wanted
                );
            }
            lastoff = Some(off);
        } else {
            panic!(
                "{:?} missing element {:?} in Append result {:?}",
                clnt, wanted, v
            )
        }
    }
}

// 检查所有已知 append 操作是否按每个并发客户端的顺序出现在值中
#[allow(clippy::needless_range_loop)]
fn check_concurrent_appends(v: String, counts: &[usize]) {
    let nclients = counts.len();
    for i in 0..nclients {
        let mut lastoff = None;
        for j in 0..counts[i] {
            let wanted = format!("x {} {} y", i, j);
            if let Some(off) = v.find(&wanted) {
                let off1 = v.rfind(&wanted).unwrap();
                assert_eq!(off1, off, "duplicate element {:?} in Append result", wanted);

                if let Some(lastoff) = lastoff {
                    assert!(
                        off > lastoff,
                        "wrong order for element {:?} in Append result",
                        wanted
                    );
                }
                lastoff = Some(off);
            } else {
                panic!(
                    "{:?} missing element {:?} in Append result {:?}",
                    i, wanted, v
                )
            }
        }
    }
}

// 定期重新划分服务器分区
fn partitioner(
    cfg: Arc<Config>,
    ch: mpsc::Sender<bool>,
    done: Arc<AtomicUsize>,
) -> impl Future<Output = ()> + Send + 'static {
    fn delay(r: u64) -> Delay {
        Delay::new(RAFT_ELECTION_TIMEOUT + Duration::from_millis(r % 200))
    }

    // poll_fn 的上下文
    let mut all = cfg.all();
    let mut sleep = None;
    let mut is_parked = false;
    future::poll_fn(move |cx| {
        let mut rng = rand::thread_rng();
        while done.load(Ordering::Relaxed) == 0 {
            if !is_parked {
                all.shuffle(&mut rng);
                let offset = rng.gen_range(0, cfg.n);
                cfg.partition(&all[..offset], &all[offset..]);
                sleep = Some(delay(rng.gen::<u64>()));
            }
            is_parked = true;
            let sleep = sleep.as_mut().unwrap();
            futures::pin_mut!(sleep);
            futures::ready!(sleep.poll(cx));
            is_parked = false;
        }
        ch.send(true).unwrap();
        Poll::Ready(())
    })
}

// 基本测试如下：一个或多个客户端在一段时间内向一组服务器提交 Append/Get 操作。
// 在时间段结束后，测试检查特定键的所有追加值是否按顺序存在。
// 如果 unreliable 为 true，则 RPC 可能失败。
// 如果 crash 为 true，则服务器在时间段结束后崩溃并重启。
// 如果 partitions 为 true，则测试会与客户端和服务器并发地重新划分网络。
// 如果 maxraftstate 为正数，则 Raft 状态的大小（即日志大小）不应超过 2*maxraftstate。
fn generic_test(
    part: &str,
    nclients: usize,
    unreliable: bool,
    crash: bool,
    partitions: bool,
    maxraftstate: Option<usize>,
) {
    let mut title = "Test: ".to_owned();
    if unreliable {
        // 网络会丢弃 RPC 请求和响应
        title += "unreliable net, ";
    }
    if crash {
        // 节点重启，因此持久化必须正常工作
        title += "restarts, ";
    }
    if partitions {
        // 网络可能会分区
        title += "partitions, ";
    }
    if maxraftstate.is_some() {
        title += "snapshots, ";
    }
    if nclients > 1 {
        title += "many clients";
    } else {
        title += "one client";
    }
    title = format!("{} ({})", title, part); // 3A 或 3B

    const NSERVERS: usize = 5;
    let cfg = Arc::new(Config::new(NSERVERS, unreliable, maxraftstate));

    cfg.begin(&title);

    let ck = cfg.make_client(&cfg.all());

    let done_partitioner = Arc::new(AtomicUsize::new(0));
    let done_clients = Arc::new(AtomicUsize::new(0));
    let mut clnt_txs = vec![];
    let mut clnt_rxs = vec![];
    for _ in 0..nclients {
        let (tx, rx) = mpsc::channel();
        clnt_txs.push(tx);
        clnt_rxs.push(rx);
    }
    for i in 0..3 {
        let (partitioner_tx, partitioner_rx) = mpsc::channel();
        debug!("Iteration {}", i);
        done_clients.store(0, Ordering::Relaxed);
        done_partitioner.store(0, Ordering::Relaxed);
        let clnt_txs_ = clnt_txs.clone();
        let cfg_ = cfg.clone();
        let done_clients_ = done_clients.clone();
        thread::spawn(move || {
            block_on(async {
                spawn_clients_and_wait(cfg_.clone(), nclients, move || {
                    let cfg1 = cfg_.clone();
                    let clnt_txs1 = clnt_txs_.clone();
                    let done_clients1 = done_clients_.clone();
                    move |cli, myck| {
                        // TODO: 将闭包改为 future
                        let mut j = 0;
                        let mut rng = rand::thread_rng();
                        let mut last = String::new();
                        let key = format!("{}", cli);
                        put(&cfg1, myck, &key, &last);
                        while done_clients1.load(Ordering::Relaxed) == 0 {
                            if (rng.gen::<u32>() % 1000) < 500 {
                                let nv = format!("x {} {} y", cli, j);
                                debug!("{}: client new append {}", cli, nv);
                                last = next_value(last, &nv);
                                append(&cfg1, myck, &key, &nv);
                                j += 1;
                            } else {
                                debug!("{}: client new get {:?}", cli, key);
                                let v = get(&cfg1, myck, &key);
                                if v != last {
                                    panic!(
                                        "get wrong value, key {:?}, wanted:\n{:?}\n, got\n{:?}",
                                        key, last, v
                                    );
                                }
                            }
                        }
                        clnt_txs1[cli].send(j).unwrap();
                    }
                })
                .await
            })
        });

        if partitions {
            // 允许客户端在无干扰的情况下执行一些操作
            thread::sleep(Duration::from_secs(1));
            cfg.net.spawn_poller(partitioner(
                cfg.clone(),
                partitioner_tx,
                done_partitioner.clone(),
            ));
        }
        thread::sleep(Duration::from_secs(5));

        // 通知客户端退出
        done_clients.store(1, Ordering::Relaxed);
        // 通知分区器退出
        done_partitioner.store(1, Ordering::Relaxed);

        if partitions {
            debug!("wait for partitioner");
            partitioner_rx.recv().unwrap();
            // 重新连接网络并提交请求。客户端可能在少数派中提交了请求，
            // 该请求直到服务器发现新任期开始才会返回
            cfg.connect_all();
            // 等待一段时间以确保新任期开始
            thread::sleep(RAFT_ELECTION_TIMEOUT);
        }

        if crash {
            debug!("shutdown servers");
            for i in 0..NSERVERS {
                cfg.shutdown_server(i)
            }
            // 等待一段时间让服务器关闭，因为 shutdown 不是真正的崩溃，也不是瞬时的
            thread::sleep(RAFT_ELECTION_TIMEOUT);
            debug!("restart servers");
            // 崩溃并重启所有服务器
            for i in 0..NSERVERS {
                cfg.start_server(i);
            }
            cfg.connect_all();
        }

        debug!("wait for clients");
        for (i, clnt_rx) in clnt_rxs.iter().enumerate() {
            debug!("read from clients {}", i);
            let j = clnt_rx.recv().unwrap();
            if j < 10 {
                debug!(
                    "Warning: client {} managed to perform only {} put operations in 1 sec?",
                    i, j
                );
            }
            let key = format!("{}", i);
            debug!("Check {:?} for client {}", j, i);
            let v = get(&cfg, &ck, &key);
            check_clnt_appends(i, v, j);
        }

        if let Some(maxraftstate) = maxraftstate {
            // 在所有服务器处理完客户端请求并有时间进行快照后检查最大值
            if cfg.log_size() > 2 * maxraftstate {
                panic!(
                    "logs were not trimmed ({} > 2*{})",
                    cfg.log_size(),
                    maxraftstate
                )
            }
        }
    }

    cfg.check_timeout();
    cfg.end();
}

fn generic_test_linearizability(
    part: &str,
    nclients: usize,
    nservers: usize,
    unreliable: bool,
    crash: bool,
    partitions: bool,
    maxraftstate: Option<usize>,
) {
    let mut title = "Test: ".to_owned();
    if unreliable {
        // 网络会丢弃 RPC 请求和响应
        title += "unreliable net, ";
    }
    if crash {
        // 节点重启，因此持久化必须正常工作
        title += "restarts, ";
    }
    if partitions {
        // 网络可能会分区
        title += "partitions, ";
    }
    if maxraftstate.is_some() {
        title += "snapshots, ";
    }
    if nclients > 1 {
        title += "many clients";
    } else {
        title += "one client";
    }
    title = format!("{}, linearizability checks ({})", title, part); // 3A 或 3B

    let cfg = Arc::new(Config::new(nservers, unreliable, maxraftstate));

    cfg.begin(&title);

    let begin = Instant::now();
    let operations = Arc::new(Mutex::new(vec![]));

    let done_partitioner = Arc::new(AtomicUsize::new(0));
    let done_clients = Arc::new(AtomicUsize::new(0));
    let mut clnt_txs = vec![];
    let mut clnt_rxs = vec![];
    for _ in 0..nclients {
        let (tx, rx) = mpsc::channel();
        clnt_txs.push(tx);
        clnt_rxs.push(rx);
    }
    for i in 0..3 {
        let (partitioner_tx, partitioner_rx) = mpsc::channel();
        debug!("Iteration {}", i);
        done_clients.store(0, Ordering::Relaxed);
        done_partitioner.store(0, Ordering::Relaxed);
        let clnt_txs_ = clnt_txs.clone();
        let cfg_ = cfg.clone();
        let done_clients_ = done_clients.clone();
        let operations_ = operations.clone();
        cfg.net
            .spawn_poller(spawn_clients_and_wait(cfg.clone(), nclients, move || {
                let cfg1 = cfg_.clone();
                let clnt_txs1 = clnt_txs_.clone();
                let done_clients1 = done_clients_.clone();
                let operations1 = operations_.clone();
                move |cli, myck| {
                    // TODO: 将闭包改为 future
                    let mut j = 0;
                    let mut rng = rand::thread_rng();
                    while done_clients1.load(Ordering::Relaxed) == 0 {
                        let key = format!("{}", rng.gen::<usize>() % nclients);
                        let nv = format!("x {} {} y", cli, j);

                        let start = begin.elapsed().as_nanos() as i64;
                        let (inp, out) = if rng.gen::<usize>() % 1000 < 500 {
                            append(&cfg1, myck, &key, &nv);
                            j += 1;
                            (
                                KvInput {
                                    op: Op::Append,
                                    key,
                                    value: nv,
                                },
                                KvOutput {
                                    value: "".to_string(),
                                },
                            )
                        } else if rng.gen::<usize>() % 1000 < 100 {
                            put(&cfg1, myck, &key, &nv);
                            j += 1;
                            (
                                KvInput {
                                    op: Op::Put,
                                    key,
                                    value: nv,
                                },
                                KvOutput {
                                    value: "".to_string(),
                                },
                            )
                        } else {
                            let v = get(&cfg1, myck, &key);
                            (
                                KvInput {
                                    op: Op::Get,
                                    key,
                                    value: "".to_string(),
                                },
                                KvOutput { value: v },
                            )
                        };

                        let end = begin.elapsed().as_nanos() as i64;
                        let op = Operation {
                            input: inp,
                            call: start,
                            output: out,
                            finish: end,
                        };
                        let mut data = operations1.lock().unwrap();
                        data.push(op);
                    }
                    clnt_txs1[cli].send(j).unwrap();
                }
            }));

        if partitions {
            // 允许客户端在无干扰的情况下执行一些操作
            thread::sleep(Duration::from_secs(1));
            cfg.net.spawn_poller(partitioner(
                cfg.clone(),
                partitioner_tx,
                done_partitioner.clone(),
            ));
        }
        thread::sleep(Duration::from_secs(5));

        // 通知客户端退出
        done_clients.store(1, Ordering::Relaxed);
        // 通知分区器退出
        done_partitioner.store(1, Ordering::Relaxed);

        if partitions {
            debug!("wait for partitioner");
            partitioner_rx.recv().unwrap();
            // 重新连接网络并提交请求。客户端可能在少数派中提交了请求，
            // 该请求直到服务器发现新任期开始才会返回
            cfg.connect_all();
            // 等待一段时间以确保新任期开始
            thread::sleep(RAFT_ELECTION_TIMEOUT);
        }

        if crash {
            debug!("shutdown servers");
            for i in 0..nservers {
                cfg.shutdown_server(i)
            }
            // 等待一段时间让服务器关闭，因为 shutdown 不是真正的崩溃，也不是瞬时的
            thread::sleep(RAFT_ELECTION_TIMEOUT);
            debug!("restart servers");
            // 崩溃并重启所有服务器
            for i in 0..nservers {
                cfg.start_server(i);
            }
            cfg.connect_all();
        }

        // 等待客户端
        for clnt_rx in &clnt_rxs {
            clnt_rx.recv().unwrap();
        }

        if let Some(maxraftstate) = maxraftstate {
            // 在所有服务器处理完客户端请求并有时间进行快照后检查最大值
            if cfg.log_size() > 2 * maxraftstate {
                panic!(
                    "logs were not trimmed ({} > 2*{})",
                    cfg.log_size(),
                    maxraftstate
                )
            }
        }
    }

    cfg.check_timeout();
    cfg.end();

    if !check_operations_timeout(
        KvModel {},
        Arc::try_unwrap(operations).unwrap().into_inner().unwrap(),
        LINEARIZABILITY_CHECK_TIMEOUT,
    ) {
        panic!("history is not linearizable");
    }
}

#[test]
fn test_basic_3a() {
    // 测试：一个客户端 (3A) ...
    generic_test("3A", 1, false, false, false, None)
}

#[test]
fn test_concurrent_3a() {
    // 测试：多个客户端 (3A) ...
    generic_test("3A", 5, false, false, false, None)
}

#[test]
fn test_unreliable_3a() {
    // 测试：不可靠网络，多个客户端 (3A) ...
    generic_test("3A", 5, true, false, false, None)
}

#[test]
fn test_unreliable_one_key_3a() {
    let nservers = 3;
    let cfg = {
        let cfg = Config::new(nservers, true, None);
        cfg.begin("Test: concurrent append to same key, unreliable (3A)");
        Arc::new(cfg)
    };

    let all = cfg.all();
    let ck = cfg.make_client(&all);

    put(&cfg, &ck, "k", "");

    let cfg_ = cfg.clone();
    let nclient = 5;
    let upto = 10;
    block_on(async {
        spawn_clients_and_wait(cfg.clone(), nclient, move || {
            let cfg1 = cfg_.clone();
            move |me, myck| {
                for n in 0..upto {
                    append(&cfg1, myck, "k", &format!("x {} {} y", me, n));
                }
            }
        })
        .await
    });

    let counts = vec![upto; nclient];

    let vx = get(&cfg, &ck, "k");
    check_concurrent_appends(vx, &counts);

    cfg.check_timeout();
    cfg.end();
}

// 在少数派分区中提交请求，并检查请求是否在分区恢复前未通过。
// 原始网络中的 leader 最终处于少数派分区。
#[test]
fn test_one_partition_3a() {
    let nservers = 5;
    let cfg = Config::new(nservers, false, None);

    let all = cfg.all();
    let ck = cfg.make_client(&all);

    put(&cfg, &ck, "1", "13");

    cfg.begin("Test: progress in majority (3A)");

    let (p1, p2) = cfg.make_partition();
    cfg.partition(&p1, &p2);

    // 将 ckp1 连接到 p1
    let ckp1 = cfg.make_client(&p1);
    // 将 ckp2a 连接到 p2
    let ckp2a = cfg.make_client(&p2);
    let ckp2a_name = ckp2a.name.clone();
    // 将 ckp2b 连接到 p2
    let ckp2b = cfg.make_client(&p2);
    let ckp2b_name = ckp2b.name.clone();

    put(&cfg, &ckp1, "1", "14");
    check(&cfg, &ckp1, "1", "14");

    cfg.end();

    let (done0_tx, done0_rx) = oneshot::channel::<&'static str>();
    let (done1_tx, done1_rx) = oneshot::channel::<&'static str>();

    cfg.begin("Test: no progress in minority (3A)");
    cfg.net.spawn(future::lazy(move |_| {
        ckp2a.put("1".to_owned(), "15".to_owned());
        done0_tx
            .send("put")
            .map_err(|e| {
                warn!("done0 send failed: {:?}", e);
            })
            .unwrap();
    }));
    let done0_rx = done0_rx.map(|op| {
        cfg.op();
        op
    });

    cfg.net.spawn(future::lazy(move |_| {
        // p2 中的不同客户端
        ckp2b.get("1".to_owned());
        done1_tx
            .send("get")
            .map_err(|e| {
                warn!("done0 send failed: {:?}", e);
            })
            .unwrap();
    }));
    let done1_rx = done1_rx.map(|op| {
        cfg.op();
        op
    });

    let timeout = Delay::new(Duration::from_secs(1));

    let dones = block_on(
        future::select(timeout, future::select(done0_rx, done1_rx)).map(|res| match res {
            future::Either::Left((_, dones)) => dones,
            future::Either::Right((future::Either::Left((op, _)), _)) => {
                panic!("{} in minority completed", op.unwrap())
            }
            future::Either::Right((future::Either::Right((op, _)), _)) => {
                panic!("{} in minority completed", op.unwrap())
            }
        }),
    );

    check(&cfg, &ckp1, "1", "14");
    put(&cfg, &ckp1, "1", "16");
    check(&cfg, &ckp1, "1", "16");

    cfg.end();

    cfg.begin("Test: completion after heal (3A)");

    cfg.connect_all();
    cfg.connect_client_by_name(&ckp2a_name, &all);
    cfg.connect_client_by_name(&ckp2b_name, &all);

    thread::sleep(RAFT_ELECTION_TIMEOUT);

    let timeout = Delay::new(Duration::from_secs(3));
    let (timeout, next) = block_on(async {
        future::select(timeout, dones)
            .map(|res| match res {
                future::Either::Left(_) => panic!("put/get did not complete"),
                future::Either::Right((future::Either::Left((op, next)), timeout)) => {
                    info!("{} completes", op.unwrap());
                    (timeout, future::Either::Left(next))
                }
                future::Either::Right((future::Either::Right((op, next)), timeout)) => {
                    info!("{} completes", op.unwrap());
                    (timeout, future::Either::Right(next))
                }
            })
            .await
    });

    block_on(async {
        future::select(timeout, next)
            .map(|res| match res {
                future::Either::Left(_) => panic!("put/get did not complete"),
                future::Either::Right((op, _)) => info!("{} completes", op.unwrap()),
            })
            .await
    });

    check(&cfg, &ck, "1", "15");

    cfg.end();
}

#[test]
fn test_many_partitions_one_client_3a() {
    // 测试：分区，一个客户端 (3A) ...
    generic_test("3A", 1, false, false, true, None)
}

#[test]
fn test_many_partitions_many_clients_3a() {
    // 测试：分区，多个客户端 (3A) ...
    generic_test("3A", 5, false, false, true, None)
}

#[test]
fn test_persist_one_client_3a() {
    // 测试：重启，一个客户端 (3A) ...
    generic_test("3A", 1, false, true, false, None)
}

#[test]
fn test_persist_concurrent_3a() {
    // 测试：重启，多个客户端 (3A) ...
    generic_test("3A", 5, false, true, false, None)
}

#[test]
fn test_persist_concurrent_unreliable_3a() {
    // 测试：不可靠网络，重启，多个客户端 (3A) ...
    generic_test("3A", 5, true, true, false, None)
}

#[test]
fn test_persist_partition_3a() {
    // 测试：重启，分区，多个客户端 (3A) ...
    generic_test("3A", 5, false, true, true, None)
}

#[test]
fn test_persist_partition_unreliable_3a() {
    // 测试：不可靠网络，重启，分区，多个客户端 (3A) ...
    generic_test("3A", 5, true, true, true, None)
}

#[test]
fn test_persist_partition_unreliable_linearizable_3a() {
    // 测试：不可靠网络，重启，分区，线性一致性检查 (3A) ...
    generic_test_linearizability("3A", 15, 7, true, true, true, None)
}

// 如果一个服务器落后，然后重新加入，它是否通过 InstallSnapshot RPC 恢复？
// 同时检查多数派是否丢弃已提交的日志条目，即使少数派没有响应。
#[test]
fn test_snapshot_rpc_3b() {
    let nservers = 3;
    let maxraftstate = 1000;
    let cfg = Config::new(nservers, false, Some(maxraftstate));

    let all = cfg.all();
    let ck = cfg.make_client(&all);

    cfg.begin("Test: InstallSnapshot RPC (3B)");

    put(&cfg, &ck, "a", "A");
    check(&cfg, &ck, "a", "A");

    // 向多数派分区发送大量 put 操作
    cfg.partition(&[0, 1], &[2]);
    {
        let ck1 = cfg.make_client(&[0, 1]);
        for i in 0..50 {
            put(&cfg, &ck1, &format!("{}", i), &format!("{}", i));
        }
        thread::sleep(RAFT_ELECTION_TIMEOUT);
        put(&cfg, &ck1, "b", "B");
    }

    // 检查多数派分区是否丢弃了大部分日志条目
    if cfg.log_size() > 2 * maxraftstate {
        panic!(
            "logs were not trimmed ({} > 2*{})",
            cfg.log_size(),
            maxraftstate
        );
    }

    // 现在构造一个需要落后服务器参与的组，使其必须赶上
    cfg.partition(&[0, 2], &[1]);
    {
        let ck1 = cfg.make_client(&[0, 2]);
        put(&cfg, &ck1, "c", "C");
        put(&cfg, &ck1, "d", "D");
        check(&cfg, &ck1, "a", "A");
        check(&cfg, &ck1, "b", "B");
        check(&cfg, &ck1, "1", "1");
        check(&cfg, &ck1, "49", "49");
    }

    // 现在所有人
    cfg.partition(&[0, 1, 2], &[]);

    put(&cfg, &ck, "e", "E");
    check(&cfg, &ck, "c", "C");
    check(&cfg, &ck, "e", "E");
    check(&cfg, &ck, "1", "1");

    cfg.check_timeout();
    cfg.end();
}

// 快照是否不会太大？对于我们这里的操作，500 字节是一个宽松的上限
#[test]
fn test_snapshot_size_3b() {
    let nservers = 3;
    let maxraftstate = 1000;
    let maxsnapshotstate = 500;
    let cfg = Config::new(nservers, false, Some(maxraftstate));

    let all = cfg.all();
    let ck = cfg.make_client(&all);

    cfg.begin("Test: snapshot size is reasonable (3B)");

    for _ in 0..200 {
        put(&cfg, &ck, "x", "0");
        check(&cfg, &ck, "x", "0");
        put(&cfg, &ck, "x", "1");
        check(&cfg, &ck, "x", "1");
    }

    // 检查服务器是否丢弃了大部分日志条目
    if cfg.log_size() > 2 * maxraftstate {
        panic!(
            "logs were not trimmed ({} > 2*{})",
            cfg.log_size(),
            maxraftstate,
        )
    }

    // 检查快照是否不会过大
    if cfg.snapshot_size() > maxsnapshotstate {
        panic!(
            "snapshot too large ({} > {})",
            cfg.snapshot_size(),
            maxsnapshotstate,
        )
    }

    cfg.check_timeout();
    cfg.end();
}

#[test]
fn test_snapshot_recover_3b() {
    // 测试：重启，快照，一个客户端 (3B) ...
    generic_test("3B", 1, false, true, false, Some(1000))
}

#[test]
fn test_snapshot_recover_many_clients_3b() {
    // 测试：重启，快照，多个客户端 (3B) ...
    generic_test("3B", 20, false, true, false, Some(1000))
}

#[test]
fn test_snapshot_unreliable_3b() {
    // 测试：不可靠网络，快照，多个客户端 (3B) ...
    generic_test("3B", 5, true, false, false, Some(1000))
}

#[test]
fn test_snapshot_unreliable_recover_3b() {
    // 测试：不可靠网络，重启，快照，多个客户端 (3B) ...
    generic_test("3B", 5, true, true, false, Some(1000))
}

#[test]
fn test_snapshot_unreliable_recover_concurrent_partition_3b() {
    // 测试：不可靠网络，重启，分区，快照，多个客户端 (3B) ...
    generic_test("3B", 5, true, true, true, Some(1000))
}

#[test]
fn test_snapshot_unreliable_recover_concurrent_partition_linearizable_3b() {
    // 测试：不可靠网络，重启，分区，快照，线性一致性检查 (3B) ...
    generic_test_linearizability("3B", 15, 7, true, true, true, Some(1000))
}
```