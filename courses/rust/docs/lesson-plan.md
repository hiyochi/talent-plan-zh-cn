# Rust 实践网络应用课程计划

本课程旨在通过实践项目，学习如何使用 Rust 构建实用的系统级软件。

通过一系列项目，你将构建一个支持多线程和异步 I/O 的网络化 [键值数据库][kv]。在每个项目之间，你将学习并练习完成下一个项目所需的独立知识点。在此过程中，你将探索多种设计方案及其权衡取舍。

<!-- 注意：请保持上方内容与 README.md 同步 -->

有关课程概览、目标、受众和先决条件，请参阅 [README.md]。

- [先决条件](#user-content-prerequisites)
- [获取课程材料](#user-content-getting-the-materials)
- [课程结构](#user-content-course-structure)
- [获取帮助](#user-content-getting-help)
- [改进 PNA Rust 课程](#user-content-making-pna-rust-better)
- [Rust 中的实用网络应用](#user-content-practical-networked-applications-in-rust)
  - [构建模块 1](#user-content-building-blocks-1)
  - [项目 1：Rust 工具箱](#user-content-project-1-the-rust-toolbox)
  - [构建模块 2](#user-content-building-blocks-2)
  - [项目 2：日志结构文件 I/O](#user-content-project-2-log-structured-file-io)
  - [构建模块 3](#user-content-building-blocks-3)
  - [项目 3：同步客户端-服务器网络](#user-content-project-3-synchronous-client-server-networking)
  - [构建模块 4](#user-content-building-blocks-4)
  - [项目 4：并发与并行](#user-content-project-4-concurrency-and-parallelism)
  - [构建模块 5](#user-content-building-blocks-5)
  - [项目 5：Rust 中的异步编程](#user-content-project-5-asynchronous-programming-in-rust)
- [下一步该做什么？](#user-content-what-next)


## 先决条件

如 [README.md 中所述][pre]，本课程并非为编程新手设计，有显著的先决条件。请确保你满足所有要求后再继续学习。


## 获取课程材料

本课程的所有材料均位于 GitHub 上的以下 Git 仓库中：

> https://github.com/pingcap/talent-plan

具体位于 [`rust` 子目录][rs]。你应当将该仓库克隆到本地计算机，以便轻松访问每个项目的合规性测试。

## 课程结构

本课程的整体结构由一系列编码项目组成，这些项目逐步引入 Rust 系统编程中重要的新主题。每个项目都建立在前一个项目的基础上，因此你完全可以从上一个项目结束的地方继续在同一个 Git 仓库中开展工作（尽管你可能希望添加一个 [git tag] 标记上一个项目结束的位置）。

[git tag]: https://git-scm.com/book/en/v2/Git-Basics-Tagging

由于在学习所有相关概念的同时从零开始构建一个完整数据库是一项艰巨的任务，每个项目之前都设有“构建模块”部分，用于单独探索各个概念。这些构建模块将包含外部阅读材料、外部编程练习以及其他单主题内容。

构建模块部分为你提供了一个清空思绪、暂时脱离大型项目、专注于孤立学习和练习单一主题的机会。**请不要跳过这些部分**。

每个项目都建立在前一个项目的基础上——API、命令行接口和部分实现通常在项目之间保持不变，你只需集成与当前项目主题相关的新功能和改进。因此，你可以直接从上一个项目的源代码开始每个新项目。

项目位于 [`projects` 子目录][psd] 中，每个项目都有独立的目录，包含一个 `README.md` 项目说明，以及一个包含完整示例解决方案和测试套件的 Cargo 项目。每个项目都附带测试套件，只有当项目通过 `cargo test` 测试时，才视为完成。

**在你自己完成项目之前，请不要阅读任何项目的示例代码** —— 良好的学习需要你自己尝试、失败，再尝试，直到成功为止。不过，你被鼓励在完成项目后，回顾并应用示例代码中包含的技术。但请记住，示例项目并非唯一或最佳的解决方案。相信你自己和你的创造力。

<!-- TODO 这段话过于严厉

> 关于抄袭（主要适用于课程作业评分的学生）：通过阅读代码学习技术与直接复制代码之间的界限有时难以界定。但作为专业人士，你有道德责任，只有你自己才能判断是否遵守了这些责任。对于不参与课程评分的人，直接复制示例代码并无大碍；但对于正在接受评估的学生，你的导师和评估者期望你运用自己的技能。

-->

随着你逐步推进各个项目，你将获得有关如何设置源代码和测试套件以及项目规范的进一步指导。

目前尚无法准确估算完成每个部分所需的时间，但“构建模块”和“项目”阶段预计均需数小时而非数天，其中项目耗时更长。如果你花费的时间远少于或远多于这个范围，请不要担心：这些只是粗略估计，每个人的体验都不同。

## 获取帮助

你可能会遇到无法解决的问题。请不要独自忍受。每个人有时都需要帮助。

幸运的是，Rust 社区非常出色且乐于助人，他们愿意帮助你。在学习本课程的过程中，我们强烈建议你加入 Rust 社区。

以下是你可以优先考虑的资源：

- [TiKV Slack] 上的 #rust-training 频道。该频道专为本课程设立。请考虑加入，以支持你的同学和其他学习者。总有人在那里回答问题，但由于时区和其他因素，可能会有延迟。这里欢迎使用英语和中文交流。

- 官方 [Rust Discord] 的 `#beginners` 频道。你几乎一定能在这里得到答案，如果没得到，不要犹豫，再问一次。这里的人专门为了帮助他人而来。由于时区差异，可能需要一些时间才能得到回复。此处仅能保证英语交流。

- QQ Rust 群组 #1（[二维码][qq]）。对于中国学生，这是中国主要的 Rust 社区之一。这里非常适合日常交流，对于英语能力不足的同学，也是寻求帮助的好地方。此外还有微信群组，但因人数上限低且需邀请，使用起来较为困难。

- QQ Rust 群组 #2（[二维码][qq2]）。与上面类似。如果群组 1 已满，你可以加入这个群组。

以下资源也可能有帮助：

- 官方 [users forum]。在你的帖子中添加 "help" 标签。问题通常能得到回复，但回复可能有限。

- [StackOverflow]。添加 "rust" 标签。你可能得到满意的答案，也可能不会。

你也可以通过电子邮件联系本课程的主要作者 [Brian Anderson][brson]，邮箱为 brian@pingcap.com。我乐意回答你的问题，并热切期待听到你对本课程的体验。

最后，如果你附近有 [Rust meetup]，请去参加一下。作为 Rust 程序员，这些团体是你建立最牢固联系的地方。（注意：该链接指向旧版 Rust 网站，可能未更新。）

## 改进 PNA Rust 课程

<!-- TODO 每个项目结束后都有一个关于你体验的调查链接。请花几分钟完成它，坦诚表达你遇到的挑战和对课程的批评。调查结果仅课程[主要作者][author]可见，但汇总统计可能公开。 -->

在学习课程内容的过程中，请留意任何你认为可以改进的地方，并通过 [提交问题][si] 说明，或 [提交拉取请求][spr] 提出改进方案。（如果你正在接受评分，被接受的对本课程或其他课程所用仓库的拉取请求 _可能_ 在最终评估中计入额外加分 —— 请告知你的导师或评估者！）这是一个参与开源 Rust 项目贡献的绝佳机会。让本课程变得比你学习时更好，为下一位学习者提供更优质的体验。

更多信息请参见 [CONTRIBUTING.md]。

## Rust 中的实用网络应用

这是课程大纲。点击每个标题即可跳转到相应部分的说明。

### [构建模块 1][b1]

**主题**：CLI 编程、cargo 清单与环境变量、Rust 项目文档编写。

### [项目 1：Rust 工具箱][p1]

**任务**：创建一个内存中的键值存储，通过简单测试并响应命令行参数。

**目标**：

- 安装 Rust 编译器及工具
- 学习本课程中使用的项目结构
- 使用 `cargo init` / `run` / `test` / `clippy` / `fmt`
- 学习如何从 crates.io 查找并导入 crate
- 为键值存储定义合适的数据类型

**主题**：测试、`clap` crate、`CARGO_VERSION` 等、`clippy` 和 `rustfmt` 工具。

**扩展**：`structopt` crate。

### [构建模块 2][b2]

**主题**：日志结构文件 I/O、bitcask 算法、Rust 错误处理、集合类型比较。

### [项目 2：日志结构文件 I/O][p2]

**任务**：创建一个可通过命令行访问的持久化键值存储。

**目标**：

- 健壮地处理和报告错误
- 使用 serde 进行序列化
- 使用标准文件 API 将数据以日志形式写入磁盘
- 从磁盘读取键值存储的状态
- 将内存中的键索引映射到磁盘上的值
- 定期压缩日志以移除过期数据

**主题**：日志结构文件 I/O、bitcask、`failure` crate、`Read` / `Write` trait、`serde` crate。

### [构建模块 3][b3]

**主题**：非结构化日志与结构化日志、Redis 协议、基准测试。

### [项目 3：同步客户端-服务器网络][p3]

**任务**：创建一个单线程、持久化的键值存储服务器和客户端，使用自定义协议通过同步网络通信。

**目标**：

- 创建客户端-服务器应用程序
- 使用 `std` 网络 API 编写自定义协议
- 为服务器引入日志记录
- 使用 trait 实现可插拔后端
- 将手写后端与 `sled` 进行基准测试对比

**主题**：`std::net`、日志记录、trait、基准测试。

### [构建模块 4][b4]

**主题**：多线程、线程池、别名与可变性、并发数据类型。

### [项目 4：并发与并行][p4]

**任务**：创建一个多线程、持久化的键值存储服务器和客户端，使用自定义协议通过同步网络通信。

**目标**：

- 编写一个简单的线程池
- 使用通道进行跨线程通信
- 使用锁共享数据结构
- 在不加锁的情况下执行读操作
- 对比单线程与多线程的基准测试

**主题**：线程池、通道、锁、无锁数据结构、原子操作、参数化基准测试。

### 构建模块 5

即将推出！([预览][b5])

### 项目 5：Rust 中的异步编程

即将推出！([预览][p5])

## 下一步该做什么？

你已经完成了《Rust 中的实用网络应用》课程。这是一项了不起的 Rust 成就！现在你已踏上成为一名优秀 Rust 程序员的道路。想知道接下来该往哪里走吗？我们为你准备了 [一些建议][n]。

<!-- 构建模块链接 -->

[b1]: ../building-blocks/bb-1.md
[b2]: ../building-blocks/bb-2.md
[b3]: ../building-blocks/bb-3.md
[b4]: ../building-blocks/bb-4.md
[b5]: ../building-blocks/bb-5.md

<!-- 项目链接 -->

[p1]: ../projects/project-1/README.md
[p2]: ../projects/project-2/README.md
[p3]: ../projects/project-3/README.md
[p4]: ../projects/project-4/README.md
[p5]: ../projects/project-5/README.md

<!-- 其他链接 -->

[CONTRIBUTING.md]: ../CONTRIBUTING.md
[README.md]: ../README.md
[Rust Discord]: https://discord.gg/rust-lang
[Rust meetup]: https://www.meetup.com/topics/rust
[StackOverflow]: https://stackoverflow.com/questions/tagged/rust
[TiKV Slack]: https://join.slack.com/t/tikv-wg/shared_invite/enQtNTUyODE4ODU2MzI0LWVlMWMzMDkyNWE5ZjY1ODAzMWUwZGVhNGNhYTc3MzJhYWE0Y2FjYjliYzY1OWJlYTc4OWVjZWM1NDkwN2QxNDE
[author]: https://github.com/brson/
[brson]: https://github.com/brson/
[kv]: https://en.wikipedia.org/wiki/Key-value_database
[pre]: ../README.md#user-content-prerequisites
[psd]: https://github.com/pingcap/talent-plan/tree/master/courses/rust/projects
[qq]: ./qq-qr.jpg
[qq2]: ./qq2-qr.jpg
[rs]: https://github.com/pingcap/talent-plan/tree/master/courses/rust
[si]: https://github.com/pingcap/talent-plan/issues
[spr]: https://github.com/pingcap/talent-plan/pulls
[users forum]: https://users.rust-lang.org/
[n]: ./what-next.md