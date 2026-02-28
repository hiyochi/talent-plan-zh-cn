# 项目：异步编程

**任务**：创建一个支持多线程、持久化键值存储的服务器和客户端，通过自定义协议实现**异步**网络通信。

**目标**：

- 理解在 Rust 中编写 Future 时使用的模式
- 理解 Future 中的错误处理机制
- 学习调试 Rust 的类型系统
- 使用 tokio 运行时进行异步网络编程
- 使用装箱 Future（boxed futures）解决复杂的类型系统问题
- 使用 `impl Trait` 创建匿名的 `Future` 类型

**主题**：异步编程、Future、tokio、`impl Trait`

**扩展**：tokio-fs

- [引言](#user-content-introduction)
- [项目规范](#user-content-project-spec)
- [项目设置](#user-content-project-setup)
- [背景：在 Rust 中思考 Future](#user-content-background-thinking-in-futures,-in-rust)
- [第一部分：为客户端引入 tokio](#user-content-part-1-introducing-tokio-to-the-client)
- [第二部分：将 `KvsClient` 转换为装箱 Future](#user-content-part-2-converting-kvsclient-to-boxed-futures)
- [第三部分：使用显式 Future 类型的 `KvsClient`](#user-content-part-3-kvsclient-with-explicit-future-types)
- [第四部分：使用匿名 Future 类型的 `KvsClient`](#user-content-part-4-kvsclient-with-anonymous-future-types)
- [第五部分：使 `ThreadPool` 可共享](#user-content-part-5-making-threadpool-sharable)
- [第六部分：将 `KvsEngine` 转换为 Future](#user-content-part-6-converting-kvsengine-to-futures)
- [第七部分：使用 tokio 驱动 `KvsEngine`](#user-content-part-7-driving-kvsengine-with-tokio)
- [扩展一：转换为 tokio-fs](#user-content-extension-1-converting-to-tokio-fs)


## 引言

> **注意**：本项目目前仅提供大纲，尚未完整撰写。如果你正在课程的这一阶段，请发送邮件至 brian@pingcap.com 告知我，我会尽快完成撰写。

在本项目中，你将构建一个简单的键值存储服务器和客户端，它们通过自定义协议进行通信。服务器将基于 tokio 运行时实现异步网络通信。键值存储引擎（负责读写文件）仍保持同步实现，但通过底层线程池调度任务，对外提供异步接口。在此过程中，你将探索多种定义和使用 Future 类型的方式。

由于学习 Rust 的 Future 编程尤其具有挑战性，且相关文档有限，本项目的范围相对较小，但提供了比以往项目更直接的解释。

请务必阅读本项目的背景阅读材料。如果你感到沮丧，请原谅自己，休息一下，再以全新的心态重新尝试。异步 Rust 编程对所有人来说都是困难的。

## 项目规范

cargo 项目 `kvs` 将构建两个命令行工具：键值存储客户端 `kvs-client` 和键值存储服务器 `kvs-server`，二者均调用一个名为 `kvs` 的库。客户端通过自定义协议与服务器通信。

CLI 的接口与[上一个项目]相同。引擎实现也基本一致，通过线程池分发同步文件 I/O 操作。

不同之处在于，本次所有网络通信都将以异步方式执行。

作为异步转换的一部分，`KvsClient` 将提供基于 Future 的 API，`KvsEngine` 特征也将提供基于 Future 的接口，即使其内部实现仍依赖通过线程池执行的阻塞（同步）I/O。

你的 `KvsServer` 将基于 tokio 运行时构建，该运行时会自动将异步任务分发到多个线程（tokio 本身包含一个线程池）。这意味着你的架构实际上将包含两层线程池：第一层用于异步网络通信（每个 CPU 核心一个线程）；第二层用于同步文件 I/O（线程数量足够多，以确保网络线程始终处于忙碌状态）。

由于这种架构变化，你的任务将从多个线程中被派发到线程池中，因此你的 `ThreadPool` 特征及其具体实现将变为可共享类型，实现 `Clone + Send + Sync`，就像你的 `KvsEngine` 一样。

由于你将实验多种 Future 返回类型的定义方式，因此本规范中并未完全指定它们，而是根据需要逐步明确。

更具体地说，你将处理如下形式的函数签名：

- `Client::get(&mut self, key: String) -> Box<Future<Item = Option<String>, Error = Error>>`
- `Client::get(&mut self, key: String) -> future::SomeExplicitCombinator<...>`
- `Client::get(&mut self, key: String) -> impl Future<Item = Option<String>, Error = Error>`
- `Client::get(&mut self, key: String) -> ClientGetFuture`


## 项目设置

继续使用你之前的项目，删除旧的 `tests` 目录，并将本项目的 `tests` 目录复制到其位置。本项目应包含一个名为 `kvs` 的库，以及两个可执行程序：`kvs-server` 和 `kvs-client`。

你需要在 `Cargo.toml` 中添加以下开发依赖项：

```toml
[dev-dependencies]
assert_cmd = "0.11"
criterion = "0.2.11"
crossbeam-utils = "0.6.5"
predicates = "1.0.0"
rand = "0.6.5"
tempfile = "3.0.7"
walkdir = "2.2.7"
panic-control = "0.1.4"
```

与之前的项目不同，无需急于填充足够的类型定义使测试套件编译通过。这样做会一次性跳过多个步骤。文本将明确指示何时开始处理测试套件。

## 背景：在 Rust 中思考 Future

- 为什么使用 Future？网络 vs 文件 I/O，阻塞 vs 非阻塞，同步 vs 异步
- 从用户视角理解 Future（而非从 poll 为中心的实现视角）
- 不要过度思考执行器和运行时
- 方法链式调用如何转换 Future 类型
- 调试 Rust 类型
- Result vs Future vs FutureResult
- Future 中的错误处理
- 具体 Future vs 装箱 Future vs 匿名 Future
- 关于 futures 0.1 和 futures 0.3 的说明（我们将使用 futures 0.1）
- 关于 async/await 的说明


## 第一部分：为客户端引入 tokio

最终我们将把客户端和服务器都转换为基于 Future 的实现，但由于客户端结构简单，我们先从它开始。我们将首先引入 tokio 运行时，同时继续使用你现有的同步 `KvsClient`。

对于客户端，我们将在保留同步 `KvsClient` 的前提下引入异步运行时，然后再将其转换。注意，作为库，`KvsClient` 可以基于 Future 提供最高效率，但我们的 `kvs-client` 可执行程序并未利用这一点，因此它将显得有些“愚蠢”——仅运行一个 Future 后就退出。

TODO @sticnarf —— 请尝试编写与具体 Future 类型无关的测试用例，使其能兼容以下所有策略。

## 第二部分：将 `KvsClient` 转换为装箱 Future

处理 Future 类型的最简单路径

## 第三部分：使用显式 Future 类型的 `KvsClient`

只是为了体验其不可持续性

## 第四部分：使用匿名 Future 类型的 `KvsClient`

最终解决方案

## 第五部分：使 `ThreadPool` 可共享

## 第六部分：将 `KvsEngine` 转换为 Future

对于服务器，我们将采取与客户端相反的做法：为 `KvsEngine` 提供异步接口。这将表明 Future 和底层运行时是独立的，它们只是提供了一种体验的连续谱。

## 第七部分：使用 tokio 驱动 `KvsEngine`

请注意，尽管我们自己编写的异步代码非常少，但 tokio 本身已在 CPU 核心数对应的线程上分发异步任务。思考将 CPU 密集型工作直接放在网络线程或文件线程上的权衡：例如，序列化操作应放在哪里？

TODO

编码愉快，朋友。好好休息一下吧。

---

## 扩展一：转换为 tokio-fs

不确定这是否应作为必做项或仅作为扩展

<!--

TODO:
- 我们能否找一个理由手动编写一个 Future？

- 背景阅读材料
  - 关于关联类型的资料

via @sticnarf:

> 由于项目5仅提供了大纲，我主要根据自己的想法编写了代码。希望这能为你撰写文本提供参考。@brson

> 我已将 concurrent_get/set 测试改为使用异步方式。学生需要修改他们的 SledKvsEngine 和 KvStore，以适配具有新异步 API 的 KvsEngine 特征。引擎具有 ThreadPool 类型参数，构造函数包含并发参数（也许我们应该移除它）。学生需要遵循此设计，以确保测试能正常运行。

> 我没有测试客户端。实现者可以自行设计客户端 API（除非我们能达成一个完美的设计，从而可以直接给出明确指令给学生）。

-->