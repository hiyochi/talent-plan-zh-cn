# Raft 实验

这是一系列基于 Raft 共识算法构建的键值存储系统的实验。这些实验源自著名的 [MIT 6.824][6824] 课程的 [lab2:raft][6824lab2] 和 [lab3:kvraft][6824lab3]，但使用 Rust 语言重写。以下文本材料也深受该课程内容的影响。

[6824lab2]:http://nil.csail.mit.edu/6.824/2018/labs/lab-raft.html  
[6824lab3]:http://nil.csail.mit.edu/6.824/2018/labs/lab-kvraft.html  
[6824]:http://nil.csail.mit.edu/6.824/2018/index.html  

在这些实验中，你将首先在 lab2:raft 中实现 Raft 共识算法，然后在 lab3:kvraft 中构建一个键值服务。

Raft 是一种旨在易于理解的共识算法。你可以在 [Raft 官方网站][raftsite] 上阅读有关 Raft 的资料，包括 [Raft 扩展论文][raftpaper]、Raft 的交互式可视化演示以及其他资源。这些材料对你完成本实验非常有帮助。

[raftsite]:https://raft.github.io/  

## 开始实验

首先，请使用 `git` 克隆此仓库以获取实验的源代码。

然后，确保你已安装 `rustup`。此外，为简化操作，建议你也安装 `make`。

现在，你可以运行 `make test_others` 来检查环境是否配置正确。你应该看到所有测试都通过。

（如果你是 Windows 用户，可能需要研究如何在 Windows 上使用 `make`，或者手动在控制台中输入 Makefile 中的命令，或者直接使用 Windows 子系统 for Linux）

## 实验 2：Raft

在本实验中，你将实现 Raft 共识算法。本实验分为三个部分，分别标记为 2A、2B 和 2C。

要运行本实验的所有测试，请执行 `make test_2`。请多次运行测试，以确保你的实现并非仅靠运气通过。

要运行单个测试，请执行 `make cargo_test_<在此处插入测试名称>`。

### 代码结构

本实验的所有代码应位于 `src/proto/mod.rs`、`src/proto/raft.proto` 和 `src/raft/mod.rs` 文件中。

`src/raft/mod.rs` 文件应包含你对 Raft 的主要实现。测试程序（以及你在实验 3 中的键值服务器）将调用该文件中的方法来使用你的 Raft 模块。

一个服务通过调用 `Raft::new` 创建一个 Raft 节点，然后调用 `Node::new` 启动该 Raft 节点。`Node::get_state`、`Node::is_leader` 和 `Node::term` 方法将被调用，以获取节点的当前任期以及它是否认为自己是领导者。

当服务器需要将命令追加到日志中时，会调用 `Node::start`。`Node::start` 应立即返回，无需等待日志追加完成。一个通道（`apply_ch`）会被传入 `Raft::new`，你应为每个新提交的日志条目向该通道发送一个 `ApplyMsg`。

你的实现应使用提供的 `labrpc` crate 来交换 RPC。`labrpc` 内部使用通道模拟套接字，这使得在具有挑战性的网络条件下测试代码变得容易。你应在 `src/proto/mod.rs` 中定义 RPC，并在 `impl RaftService for Node` 中实现 RPC 服务器。一组 RPC 客户端（`peers`）会被传入 `Raft::new`，供你向其他节点发送 RPC。

### 第 2A 部分

在本部分中，你应实现领导者选举和心跳机制（不带日志条目的 `AppendEntries` RPC）。你需要确保选出一个领导者，使领导者在无故障时保持领导地位，并在旧领导者失败或与旧领导者之间的数据包丢失时选出新的领导者。

要运行本部分的所有测试，请执行 `make test_2a`。

以下是一些提示：

- 根据需要向 `Raft` 结构体添加任何状态。
- `request_vote` RPC 已定义，你只需填充 `RequestVoteArgs` 和 `RequestVoteReply` 结构体。本实验使用 `labcodec` crate 对 RPC 中的消息进行编码和解码，其内部使用外部 crate `prost`。请参阅 [prost 文档][prost]，了解如何使用 `#[Derive(Message)]` 和 `#[prost(...)]` 定义用作消息的结构体。
- 你需要自行定义 `append_entries` RPC。`labrpc` 使用 `labrpc::service!` 宏定义 RPC 服务，并根据你的定义生成服务器和客户端 trait。`labrpc/examples/echo.rs` 中有一个示例，可能对你定义新的 RPC 有所帮助。
- 本实验大量使用 `futures` 外部 crate 的功能，如通道和 `Future` trait。请阅读 [此处][futures] 关于 futures 的资料。
- 你需要让你的代码定期或在延迟后执行操作。你可以使用多个线程并调用 `std::thread::sleep`（[文档][sleep]），也可以使用 `futures-timer` 外部 crate 提供的 `futures_timer::Delay` 等工具（[文档][futures-timer]）。
- 不要忘记确保不同节点的选举超时不会总是同时触发，否则所有节点只会为自己投票，无人能成为领导者。你可以使用 `rand` 外部 crate 生成随机数（[文档][rand]）。
- 测试程序限制每对发送方和接收方之间的 RPC 调用频率为每秒 10 次。请不要在没有等待超时的情况下重复发送 RPC。
- 测试程序要求你的 Raft 在旧领导者失败后的五秒内选出新的领导者（前提是大多数节点仍能通信）。然而，如果出现平票（可能因数据包丢失或候选人不幸选择相同的随机退避时间），领导者选举可能需要多轮。你必须选择足够短的选举超时（从而心跳间隔），以确保即使需要多轮，选举也很可能在五秒内完成。
- 但同样，由于测试程序限制了 RPC 调用频率，你的选举超时也不应过小，应大于论文第 5.2 节中提到的 150~300 毫秒。请明智选择数值。
- 在 Rust 中，我们锁定数据而非代码。请仔细考虑哪些数据应放在同一个锁中。
- 当你遇到困难无法通过测试时，[Raft 论文][raftpaper] 中的图 2 非常有用。
- 打印日志消息有助于调试。我们使用外部 `log` crate（[文档][log]）以不同级别打印日志。你可以通过设置 `LOG_LEVEL` 环境变量（如 `LOG_LEVEL=labs6824=debug make test_2a`）来配置日志级别和作用域。此功能由外部 crate `env_logger` 提供（[文档][env_logger]），请阅读其文档以了解日志级别的语法。此外，你也可以通过将输出重定向到文件（如 `make test_2a 2>test.log`）来收集输出。

[prost]:https://github.com/danburkert/prost  
[futures]:https://docs.rs/futures/0.3/futures/index.html  
[sleep]:https://doc.rust-lang.org/std/thread/fn.sleep.html  
[futures-timer]:https://docs.rs/futures-timer/3.0/futures_timer/index.html  
[rand]:https://docs.rs/rand/0.7/rand/index.html  
[log]:https://docs.rs/log/0.4/log/  
[env_logger]:https://docs.rs/env_logger/0.7/env_logger/  

### 第 2B 部分

在本部分中，你应实现日志复制。你需要实现 `Node::Start` 方法，完成 `append_entries` RPC 中的其余字段并发送它们，并在领导者端推进 `commit_index`。

要运行本部分的所有测试，请执行 `make test_2b`。你可以先尝试通过 `test_basic_agree_2b` 测试。

以下是一些提示：

- 不要忘记选举限制，请参阅 [Raft 论文][raftpaper] 第 5.4.1 节。
- 每个服务器通过按正确顺序写入 `apply_ch` 来独立提交新条目。`apply_ch` 是一个 `UnboundedSender`，它会缓冲消息直到内存耗尽，因此不像原始的 Go 版本那样容易产生死锁。
- 给自己足够的时间重写实现，因为只有在编写第一个实现后，你才会意识到如何清晰地组织代码。
- `test_count_2b` 要求你在无故障时 RPC 数量不能过多。因此，你应该将 RPC 数量优化到最小。
- 你可能需要编写等待特定事件发生的代码。在这种情况下，你可以直接使用通道并等待它。

### 第 2C 部分

在本部分中，你应首先通过添加代码来实现持久化，将持久状态保存和恢复到 `Persister`，例如在 `Raft::persist` 和 `Raft::restore` 中使用 `labcodec`。你还需要确定何时以及保存什么内容，并在 `Raft::new` 中调用 `Raft::restore`。

要运行本部分的所有测试，请执行 `make test_2c`。你可以先尝试通过 `test_persist1_2c` 测试。

以下是一些提示：

- `labcodec` 的用法已在第 2A 部分的提示中涵盖。
- 本部分还引入了各种具有挑战性的测试，涉及服务器故障以及网络丢失 RPC 请求或回复的情况。请仔细检查你的实现，以发现在这种情况下才会出现的错误。
- 为了通过最后的一些具有挑战性的测试（如标记为“不可靠”的测试），你需要实现优化，允许跟随者一次性将领导者的 `nextIndex` 回退多个条目。请参阅 [Raft 论文][raftpaper] 第 7 页底部和第 8 页顶部（灰色线标记）的描述。然而，论文对此细节不多，你需要填补空白，或许可以参考 [此 6.824 Raft 讲座][optimize-hint]。

[optimize-hint]:http://nil.csail.mit.edu/6.824/2018/notes/l-raft2.txt  

## 实验 3：KvRaft

在本实验中，你将使用实验 2 中的 Raft 模块构建一个容错的键值存储服务。本实验分为两个部分，分别标记为 3A 和 3B。

要运行本实验的所有测试，请执行 `make test_3`。请多次运行测试，以确保你的实现并非仅靠运气通过。

要运行单个测试，请执行 `make cargo_test_<在此处插入测试名称>`。

### 代码结构

本实验的所有代码应位于 `src/proto/mod.rs`、`src/proto/kvraft.proto`、`src/kvraft/server.rs` 和 `src/kvraft/client.rs` 文件中。文件名说明了它们的用途。此外，你还需要修改在实验 2:raft 中接触过的文件。

### 第 3A 部分

在本部分中，你应首先实现一个在无消息丢失和无服务器故障情况下工作的解决方案。你的服务必须确保 `get(...)` 和 `put_append(...)` 是 [线性一致][linearizable] 的。

[linearizable]:https://en.wikipedia.org/wiki/Linearizability  

这意味着，对 `src/kvraft/client.rs` 中 `Clerk` 结构体方法的已完成应用程序调用，必须对所有客户端表现为以相同的线性顺序影响服务，即使在发生故障和领导者变更的情况下。在已完成的 `Clerk::Put` 或 `Clerk::Append` 之后开始的 `Clerk::Get` 应看到线性顺序中最近一次 `Clerk::Put` 或 `Clerk::Append` 写入的值。已完成的调用应具有恰好一次的语义。

一个合理的实现计划应为：

- 客户端在 `src/kvraft/client.rs` 中发送 RPC 请求
- 在 `KvServer` 的 RPC 处理程序中，通过 `raft::Node::start` 将客户端的操作输入到 Raft 日志中
- 当 Raft 日志提交时执行操作，然后回复 RPC

实现后，你应该通过基本的单客户端测试，运行 `make cargo_test_basic_3a` 进行检查。

以下是一些提示：

- 你应在接收 RPC 的同时，通过 `apply_ch` 接收来自 Raft 的提交消息。
- 如果领导者已为 Clerk 的 RPC 调用 `Raft::start`，但在请求提交到日志之前失去领导地位，你应让客户端重新向其他服务器发送 RPC 请求，直到找到新的领导者。你可以通过检查从 `apply_ch` 接收的内容来检测这一点。
- `Clerk` 客户端应记住谁是上一个领导者，并首先尝试上一个领导者。这将避免每次 RPC 都浪费时间寻找领导者。
- 如果服务器不属于大多数派且没有最新数据，则不应完成 `get` RPC。你可以简单地将 get 操作放入日志中，或实现 [Raft 论文][raftpaper] 第 8 节中描述的只读操作的优化。
- 一开始就要考虑如何使用锁。

然后，你应处理重复的客户端请求，包括客户端在一个任期内向服务器领导者发送请求，等待回复超时，并在另一个任期内向新领导者重新发送请求的情况。请求应始终只执行一次。

完成此步骤后，你应该通过本部分的所有测试。要运行本实验的所有测试，请执行 `make test_3a`。

以下是一些提示：

- 你需要唯一标识客户端操作，以确保键值服务对每个操作只执行一次。
- 你的重复检测方案应快速释放服务器内存，例如使用哈希表仅存储未提交的日志。

### 第 3B 部分

在你当前的实现中，重启的服务器会重放完整的 Raft 日志以恢复其状态。然而，对于长期运行的服务器来说，永远记住完整的 Raft 日志是不切实际的。

相反，你将修改 Raft 和 kvserver 以协同节省空间：kvserver 会不时持久存储其当前状态的“快照”，而 Raft 将丢弃快照之前的日志条目。当服务器重启（或落后领导者太多而必须追赶）时，服务器首先安装快照，然后重放快照创建点之后的日志条目。[Raft 论文][raftpaper] 第 7 节概述了该方案；你需要设计细节。

测试程序向 `KvServer::new` 传递一个 `maxraftstate`，表示你的持久 Raft 状态的最大允许大小（以字节为单位，包括日志，但不包括快照）。你应检查 Raft 状态的大小，当 Raft 状态大小接近此阈值时，应保存快照，并告知 Raft 库已拍摄快照，以便 Raft 可以丢弃旧的日志条目。`maxraftstate` 是一个 `Option<usize>`，当它为 `None` 时，你无需拍摄快照。

首先，你应修改 Raft 实现以接受压缩请求，并丢弃给定索引之前的条目，同时继续运行，仅存储该索引之后的日志条目。实验 2:raft 的测试仍应通过。

然后，你应修改 kvserver，使其能够将快照交给 Raft，并在 Raft 状态过大时请求压缩。快照应保存在 `raft::Persister` 中。

以下是一些提示：

- 你可以向 Raft 添加方法，以便 kvserver 可以管理 Raft 日志的修剪过程和 kvserver 快照的管理。
- 你可以通过运行实验 3A 的测试并将 `maxraftstate` 覆盖为 `Some(1)` 来测试你的 Raft 和 kvserver 在修剪日志下运行的能力，以及从 kvserver 快照和持久 Raft 状态的组合中重启的能力。
- 考虑快照中应包含什么内容。你应使用 `raft::Persister` 保存新快照并恢复最新快照。
- 未提交的日志也可能在快照中，因此你的 kvserver 在此情况下仍必须能够检测重复操作。

之后，你应定义 `install_snapshot` RPC，当领导者已丢弃跟随者所需的日志条目时，领导者应发送此 RPC。当跟随者收到 `install_snapshot` RPC 时，它应将快照发送给 kvserver（可能通过 `apply_ch`）。

完成此步骤后，你应该通过本部分的所有测试。要运行本实验的所有测试，请执行 `make test_3b`。

以下是一些提示：

- 你无需通过多个 RPC 发送快照。只需在单个 `install_snapshot` RPC 中发送整个快照，这对本实验来说已足够。
- 你可以先尝试 `test_snapshot_rpc_3b`。

[raftpaper]:https://raft.github.io/raft.pdf