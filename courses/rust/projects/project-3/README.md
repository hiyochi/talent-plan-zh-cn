# PNA Rust 项目 3：同步客户端-服务器网络

**任务**：创建一个**单线程**、持久化的键值存储**服务器和客户端**，使用**自定义协议**进行同步网络通信。

**目标**：

- 创建客户端-服务器应用程序
- 使用 `std` 网络 API 编写自定义协议
- 为服务器引入日志记录
- 使用特质实现可插拔的后端
- 将手写后端与 `sled` 进行基准测试

**主题**：`std::net`、日志记录、特质、基准测试。

<!-- TODO **扩展**：在信号下关闭。 -->

- [简介](#user-content-introduction)
- [项目规范](#user-content-project-spec)
- [项目设置](#user-content-project-setup)
- [第一部分：命令行解析](#user-content-part-1-command-line-parsing)
- [第二部分：日志记录](#user-content-part-2-logging)
- [第三部分：客户端-服务器网络设置](#user-content-part-3-client-server-networking-setup)
- [第四部分：跨网络实现命令](#user-content-part-4-implementing-commands-across-the-network)
- [第五部分：可插拔存储引擎](#user-content-part-5-pluggable-storage-engines)
- [第六部分：基准测试](#user-content-part-6-benchmarking)

## 简介

在本项目中，您将创建一个简单的键值服务器和客户端。它们将通过您设计的自定义网络协议进行通信。您将使用标准日志库输出日志，并正确处理跨网络边界的错误。一旦您拥有一个正常工作的客户端-服务器架构，您将通过特质抽象存储引擎，并将您的实现与 [`sled`] 引擎的性能进行比较。

## 项目规范

cargo 项目 `kvs` 构建两个命令行键值存储工具：`kvs-client`（客户端）和 `kvs-server`（服务器），二者均调用一个名为 `kvs` 的库。客户端通过自定义协议与服务器通信。

`kvs-server` 可执行文件支持以下命令行参数：

- `kvs-server [--addr IP-PORT] [--engine ENGINE-NAME]`

  启动服务器并开始监听传入连接。`--addr` 接受一个 IP 地址（IPv4 或 IPv6）和端口号，格式为 `IP:PORT`。如果未指定 `--addr`，则监听 `127.0.0.1:4000`。

  如果指定了 `--engine`，则 `ENGINE-NAME` 必须是 "kvs"（使用内置引擎）或 "sled"（使用 sled）。如果是首次运行（无先前持久化数据），默认值为 "kvs"；如果存在先前持久化的数据，则默认使用已使用的引擎。如果先前持久化的数据使用的是与当前选择不同的引擎，请打印错误并以非零退出码退出。

  如果绑定套接字失败、`ENGINE-NAME` 无效或 `IP-PORT` 无法解析为地址，请打印错误并返回非零退出码。

- `kvs-server -V`

  打印版本号。

`kvs-client` 可执行文件支持以下命令行参数：

- `kvs-client set <KEY> <VALUE> [--addr IP-PORT]`

  将字符串键的值设置为字符串。

  `--addr` 接受一个 IP 地址（IPv4 或 IPv6）和端口号，格式为 `IP:PORT`。如果未指定 `--addr`，则连接到 `127.0.0.1:4000`。

  如果服务器出错或 `IP-PORT` 无法解析为地址，请打印错误并返回非零退出码。

- `kvs-client get <KEY> [--addr IP-PORT]`

  获取指定字符串键的字符串值。

  `--addr` 接受一个 IP 地址（IPv4 或 IPv6）和端口号，格式为 `IP:PORT`。如果未指定 `--addr`，则连接到 `127.0.0.1:4000`。

  如果服务器出错或 `IP-PORT` 无法解析为地址，请打印错误并返回非零退出码。

- `kvs-client rm <KEY> [--addr IP-PORT]`

  删除指定的字符串键。

  `--addr` 接受一个 IP 地址（IPv4 或 IPv6）和端口号，格式为 `IP:PORT`。如果未指定 `--addr`，则连接到 `127.0.0.1:4000`。

  如果服务器出错、`IP-PORT` 无法解析为地址，或键不存在（"key not found"），请打印错误并返回非零退出码。

- `kvs-client -V`

  打印版本号。

所有错误消息应打印到 stderr。

`kvs` 库包含四种类型：

- `KvsClient` — 实现 `kvs-client` 与 `kvs-server` 通信所需的功能
- `KvsServer` — 实现 `kvs-server` 向 `kvs-client` 提供响应的功能
- `KvsEngine` 特质 — 定义 `KvsServer` 调用的存储接口
- `KvStore` — 手动实现 `KvsEngine` 特质
- `SledKvsEngine` — 为 [`sled`] 存储引擎实现 `KvsEngine`

[`sled`]: https://github.com/spacejam/sled

`KvsClient` 和 `KvsServer` 的设计由您决定，将受您网络协议设计的影响。测试套件不直接使用这两种类型，仅通过 CLI 进行测试。

`KvsEngine` 特质支持以下方法：

- `KvsEngine::set(&mut self, key: String, value: String) -> Result<()>`

  将字符串键的值设置为字符串。

  如果值未成功写入，则返回错误。

- `KvsEngine::get(&mut self, key: String) -> Result<Option<String>>`

  获取字符串键的字符串值。如果键不存在，则返回 `None`。

  如果值未成功读取，则返回错误。

- `KvsEngine::remove(&mut self, key: String) -> Result<()>`

  删除指定的字符串键。

  如果键不存在或值未成功读取，则返回错误。

当设置键值时，`KvStore` 将 `set` 命令以顺序日志形式写入磁盘。删除键时，`KvStore` 将 `rm` 命令写入日志。启动时，重新评估日志中的命令，并在内存索引中记录每个键最后一次设置命令的日志指针（文件偏移量）。

当使用 `get` 命令检索键值时，它在索引中搜索，如果找到，则从对应日志指针处加载日志并执行该命令。

当未压缩的日志条目大小达到给定阈值时，`KvStore` 将其压缩为新日志，移除冗余条目以回收磁盘空间。

## 项目设置

继续您之前的项目，删除之前的 `tests` 目录，并将本项目的 `tests` 目录复制到其位置。本项目应包含一个名为 `kvs` 的库，以及两个可执行文件：`kvs-server` 和 `kvs-client`。<!-- TODO 解释如何协调两个二进制文件与现有代码 -->

您需要在 `Cargo.toml` 中添加以下开发依赖项：

```toml
[dev-dependencies]
assert_cmd = "0.11"
criterion = "0.3"
predicates = "1.0.0"
rand = "0.6.5"
tempfile = "3.0.7"
walkdir = "2.2.7"
```

与之前的项目一样，添加足够的定义，使测试套件能够构建。

## 第一部分：命令行解析

与之前的项目相比，本项目的命令行解析几乎没有新内容。`kvs-client` 二进制文件接受与之前项目相同的命令行参数。现在 `kvs-server` 也有自己的一组命令行参数需要处理，如规范中所述。

_为 `kvs-server` 的命令行处理添加占位符。_

## 第二部分：日志记录

生产级服务器应用程序通常具有强大且可配置的日志系统。因此，我们现在将为 `kvs-server` 添加日志记录，并在后续过程中寻找有用的日志信息。在开发过程中，通常使用 `debug!` 和 `trace!` 级别的日志进行“打印调试”。

Rust 中有两个主要的日志系统：[`log`] 和 [`slog`]。两者都导出类似的宏，用于在不同级别记录日志，如 `error!`、`info!` 等。两者都可扩展，支持不同的后端，如记录到控制台、文件、系统日志等。

[`log`]: https://docs.rs/log/
[`slog`]: https://docs.rs/slog/

主要区别在于：`log` 相对简单，仅记录格式化字符串；而 `slog` 功能丰富，支持“结构化日志”，其中日志条目是类型化的，并以易于解析的格式序列化。

`log` 源于 Rust 最早期阶段，曾是编译器的一部分，后成为标准库的一部分，最终作为独立 crate 发布。它由 Rust 项目维护。`slog` 较新，由独立团队维护。两者均被广泛使用。

对于这两种系统，都需要选择一个“接收器”（sink）crate，即日志发送到用于显示或存储的组件。

_阅读两者，选择您喜欢的，将其作为依赖项添加，然后修改 `kvs-server` 在启动时初始化日志（在命令行解析之前）。设置为输出到 stderr（额外发送到其他位置也可以，但必须输出到 stderr 才能通过本项目的测试）。_

启动时记录服务器的版本号。同时记录配置信息。目前这意味着 IP 地址和端口，以及存储引擎的名称。

## 第三部分：客户端-服务器网络设置

接下来，我们将设置网络。在本项目中，您将使用 `std::net` 中的基本 TCP/IP 网络 API：[`TcpListener`] 和 [`TcpStream`]。

[`TcpListener`]: https://doc.rust-lang.org/std/net/struct.TcpListener.html
[`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html

本项目中，服务器是同步且单线程的。这意味着您将监听一个套接字，然后接受连接，并逐个执行和响应命令。未来我们将多次重新审视这一决策，以逐步实现异步、多线程和高性能数据库。

思考您的手动测试工作流程。现在有两个可执行文件需要处理，您需要一种同时运行它们的方法。如果您像许多人一样，您将使用两个终端，在一个中运行 `cargo run --bin kvs-server`（它将运行直到您按下 CTRL-D），在另一个中运行 `cargo run --bin kvs-client`。

这是一个使用日志宏进行调试的好机会。请继续记录每个已接受连接的信息。

_在考虑协议之前，修改 `kvs-server` 以监听并接受连接，修改 `kvs-client` 以发起连接。_

## 第四部分：跨网络实现命令

在上一个项目中，您定义了数据库接受的命令，并学习了如何使用 `serde` 将它们序列化和反序列化到日志中。

<!-- 上述内容提示您已具备所需的两个工具 -->

现在是时候在网络上传输键值存储了，远程执行之前在单个进程中实现的命令。与上一个项目中为创建日志而进行的文件 I/O 类似，您将使用 `Read` 和 `Write` 特质序列化和流式传输命令。

您将设计一个网络协议。有多种方式将数据输入/输出 TCP 流，需要做出许多决策：是基于文本的协议还是二进制协议？数据如何从内存格式转换为字节流格式？是每个连接一个请求，还是多个请求？

请记住，它必须支持成功结果和错误，现在有两种错误：由您的存储引擎生成的错误，以及网络错误。

协议的所有细节均由您决定。测试套件完全不关心数据如何在两端之间传输，只关心结果是否正确。

_编写您的网络协议。_

<!-- ## 第五部分：更多错误处理

TODO 编写本节

- 通过将错误转换为可序列化格式来处理错误响应
- 为错误添加上下文
- 将 `fn main() -> Result` 替换为自定义错误报告
-->

## 第五部分：可插拔存储引擎

您的数据库有一个由您实现的存储引擎 `KvStore`。现在您将添加第二个存储引擎。

有多个原因这样做：

- 不同的工作负载需要不同的性能特征。某些存储引擎可能在特定工作负载下表现更好。

- 它创建了一个熟悉的框架，用于比较不同的后端。

- 它为我们提供了一个创建和使用特质的理由。

- 它为我们提供了一个编写一些比较基准测试的理由！

因此，您将从 `KvStore` 接口中**提取**一个新的特质 `KvsEngine`。这是一个经典的**重构**，其中现有代码逐步转变为新形式。重构时，您通常希望将工作分解为最小的变更，以确保持续构建和运行。

您最终需要的 API 如下：

- `trait KvsEngine` 具有与 `KvStore` 相同签名的 `get`、`set` 和 `remove` 方法。

- `KvStore` 实现 `KvsEngine`，不再拥有自己的 `get`、`set` 和 `remove` 方法。

- 新增一个 `KvsEngine` 的实现 `SledKvsEngine`。您稍后需使用 `sled` 库填充其 `get` 和 `set` 方法。

您可能已经为这些定义添加了占位符（如果您的测试正在构建）。_现在是填充它们的时候了。将您的重构分解为一系列有意的变更，并确保项目在继续之前持续构建并通过之前通过的测试。_

作为最后一步，您需要考虑当 `kvs-server` 以一个引擎启动、被终止，然后以不同引擎重新启动时会发生什么。这种情况只能导致错误，您需要弄清楚如何检测这种情况并报告错误。测试 `cli_wrong_engine` 反映了此场景。

## 第六部分：基准测试

随着课程的推进，我们将越来越关注数据库的性能，探索不同架构的影响。我们鼓励您超越此处描述的模型，尝试自己的优化。

性能工作需要基准测试，因此我们现在开始。有多种方式对数据库进行基准测试，如标准测试套件 [ycsb] 和 [sysbench]。在 Rust 中，基准测试从内置工具开始，我们将从这里入手。

[ycsb]: https://github.com/brianfrankcooper/YCSB
[sysbench]: https://github.com/akopytov/sysbench

Cargo 支持使用 `cargo bench` 进行基准测试。基准测试可以使用 Rust 内置基准测试框架编写，也可以使用外部框架。

内置框架通过带有 `#[bench]` 属性的函数创建基准测试。但它不能在 Rust 稳定版通道上使用，仅在[不稳定手册][tb]和[`test` crate 文档][tc]中简要记录。尽管如此，它在整个 Rust 生态系统中被广泛使用 —— 即使使用稳定版编译的 crate，也使用夜间版进行基准测试。

[tb]: https://doc.rust-lang.org/stable/unstable-book/library-features/test.html
[tc]: https://doc.rust-lang.org/stable/test/index.html

然而，该系统实际上已被弃用 —— 它不再更新，似乎永远不会被提升到稳定版通道。

Rust 中有更好的基准测试框架。您将使用的是 [criterion]。您将使用它来满足您对 `kvs` 引擎与 `sled` 引擎性能的疑问。

这些基准测试工具通过定义一个基准测试函数，并在该函数中循环执行要基准测试的操作来工作。基准测试工具将循环尽可能多次，以统计显著性地确定操作的持续时间。

请参阅 criterion 指南中的这个基本示例：

```rust
fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
	    b.iter(|| {
		    fibonacci(20)
		});
	});
}
```

`bench_function` 的调用定义了基准测试，`iter` 的调用定义了基准测试中运行的代码。`iter` 之前和之后的代码不会被计时。

[criterion]: https://docs.rs/criterion

通过创建一个名为 `benches/benches.rs` 的文件为编写基准测试做准备。与 `tests/tests.rs` 一样，cargo 将自动找到此文件并将其编译为基准测试。

首先编写以下基准测试：

- `kvs_write` — 使用 kvs 引擎，写入 100 个值，键的长度为 1-100000 字节，值的长度为 1-100000 字节（随机）。

- `sled_write` — 使用 sled 引擎，写入 100 个值，键的长度为 1-100000 字节，值的长度为 1-100000 字节（随机）。

- `kvs_read` — 使用 kvs 引擎，从先前写入的键中读取 1000 个值，键和值长度随机。

- `sled_read` — 使用 sled 引擎，从先前写入的键中读取 1000 个值，键和值长度随机。

（作为替代方案，您也可以选择编写 2 个参数化引擎的基准测试，如 [criterion 手册][pb] 所述）。

[pb]: https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html

这些基准测试未完全指定，实现起来有相当多的细微差别。我们需要至少考虑三个因素：

- 哪些代码应被计时（写在基准测试循环内），哪些不应被计时（写在循环外）？

- 如何确保每次迭代的循环行为一致，尽管使用了“随机”数字？

- 在“读取”基准测试中，如何读取与之前写入相同的“随机”键集？

这些因素相互关联：需要仔细选择未计时的设置代码，并适当重用随机数生成器的种子值。

在所有情况下，可能返回错误的操作应断言（使用 `assert!`）它们未返回错误；在读取情况下，“get” 操作应断言键被找到。

随机数可以使用 [`rand`] crate 生成。

[`rand`]: https://docs.rs/crate/rand/

一旦您编写了基准测试，使用 `cargo bench` 运行它们。

_编写上述基准测试，并比较 `kvs` 和 `sled` 之间的结果。_

_注意：请在其他进程未运行的机器上运行基准测试。基准测试结果对运行环境非常敏感，尽管 criterion 库尽力补偿“噪声”，但最好在干净的机器上进行基准测试，没有其他活动进程。如果您有一台专门用于开发的备用机器，请使用它。如果没有，AWS 或其他云实例可能比您的本地桌面产生更一致的结果。_

<!-- TODO: criterion 输出示例 -->

编码愉快，朋友。享受一段美好的休息吧。

<!-- TODO
## 扩展 1：信号处理

- 在 KILL 信号下关闭
- TODO 需要弄清楚如何中断 tcp 监听器
-->

<!--

## 背景阅读建议

- log 文档
- slog 文档
- TCP/IP 基础知识
- 重构概述
- 特质和 impl trait
- https://bheisler.github.io/post/benchmarking-with-criterion-rs/
- 基准测试概览
- 引擎的条件编译？

## TODOs

- 考虑 `Kvs_Engine_` 特质 vs `Kv_Store_` 实现

-->