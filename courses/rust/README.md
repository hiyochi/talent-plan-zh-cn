# Rust 中实用的网络应用程序

本课程旨在教授如何使用 [Rust] 构建实用的系统软件。

通过一系列项目，你将构建一个单一的、网络化的、多线程的、异步的 Rust 应用程序。创建这个应用程序——一个 [键值数据库][kv]——将为你提供实践最佳 crate 生态系统、多种并发数据类型、异步 Rust 世界、有趣的语言特性以及重要 Rust 工具的机会。在各个项目之间，还穿插着小课程和练习，帮助你掌握完成下一个项目所需的必要知识。

<!-- TODO 让上面的内容更出彩 -->
<!-- NOTE: 请保持上述内容与 lesson-plan.md 一致 -->

涵盖的主题包括：

- Rust 程序的结构设计与维护
- 使用 [clippy] 和 [rustfmt] 等常用工具
- Rust 错误处理的最佳实践
- 使用 [serde] 进行序列化
- 受 [bitcask] 启发的简单日志结构化存储
- 使用 std 和 [tokio] 进行网络编程
- 使用 [criterion] 进行基准测试
- 使用 [crossbeam] 等工具实现有趣且可靠的并行编程
- 使用 Rust [futures] 进行异步编程
- 如何学习你未知的 Rust 知识，并找到成功所需的文档和 crate

完成本课程后，你将具备编写高性能、可靠的 Rust 系统软件所需的知识和经验。你甚至可能会发现，这样做比你想象的要简单得多。

_**重要提示：《Rust 中实用的网络应用程序》目前处于 alpha 阶段**。它包含一些错误，且范围有限。如果你现在正在学习，你很勇敢，但同时也是早期测试者，你的反馈将被高度重视。在学习过程中，请[提交问题]<!-- TODO 并完成 [项目后调查] -->。我们也鼓励你自行修复问题并提交拉取请求。详情请参见 [CONTRIBUTING.md]。<!-- 有关未来课程内容的详细信息，请参见 [路线图]。-->_

**[查看课程计划][plan]**。

## 本课程的目标

本课程的目标是教会新的 Rust 程序员如何构建真实的 [系统程序][sp]，并具备所有理想的 Rust 特性，包括高性能、可靠性和易于并发；同时使用那些对新手而言可能并不明显的最佳实践。

本课程**不**包括以下内容：安装、语法及其他 Rust 基础知识；基础数据结构与算法；基础并行与异步编程概念；或作为 Rust 语言的全面资源。这些信息在 [《Rust 编程语言》][The Rust Book] 及其他地方很容易找到。

**[查看课程计划][plan]**。

## 本课程适合谁？

《Rust 中实用的网络应用程序》面向的是新手 **Rust** 程序员，但**不**适合新手程序员。

本课程的主要受众是即将或刚刚完成计算机科学本科教育、考虑从事 Rust 系统编程职业的毕业生。其他人群也可能受益，包括没有系统编程经验的资深开发者。

## 先修要求

参加本课程者应具备：

- [ ] 相当于本科计算机科学教育水平的知识，
- [ ] 在某种编程语言中具备中等水平的经验，
- [ ] 熟悉终端和命令行操作，
- [ ] 会使用 [git]，
- [ ] 在某种语言中具备初级并行编程经验，
- [ ] 在某种语言中具备初级异步编程经验，
- [ ] 具备初级编写数据库查询代码的经验，如 [SQL]、[NoSQL]、[NewSQL]、[键值][kv] 等。
- [ ] **已完整阅读过 [《Rust 编程语言》]**，
- [ ] 已编写过一些 Rust 代码，包括书中项目：
  - [编写猜数字游戏]，
  - [构建命令行程序] 和
  - [构建多线程 Web 服务器]。

再次强调：**在开始本课程前，请先完整阅读 [《Rust 编程语言》]**。你不需要对 Rust 有超过初级水平的知识或经验，但本课程**不教授 Rust 基础知识**。

如果你能勾选以上所有项目，那么你已准备好开始本课程。如果不能，我们提供了一些 [建议][pre] 来帮助你学习先修知识。

立即开始 —— **[查看课程计划][plan]**。

## 本系列的其他课程

本课程是 [PingCAP] 发起的 [一系列课程] 的一部分，旨在培训学生、贡献者、新员工和现有员工掌握 Rust 用于分布式系统开发。完成本课程后，你可能希望继续学习 [Rust 中的分布式系统]。

## PingCAP 特别说明

本课程结合 [深入 TiKV] 和 [Rust 中的分布式系统] 课程，旨在使程序员能够有意义地为 [TiKV] 做出贡献。它特别为教授中国 Rust 社区成员足够多的 Rust 知识以参与 TiKV 开发而设计。课程语言力求简单，以便英语阅读能力有限的人也能理解。如果你发现任何语言表达难以理解，请[提交问题]。

## 贡献指南

请参阅 [CONTRIBUTING.md]。

## 许可证

本课程的所有文本和代码均采用 [CC-BY 4.0] 和 [MIT] 双重许可。你可以自由地根据任一或两者条款重用此处的任何材料。

<!-- 链接 -->

[CONTRIBUTING.md]: CONTRIBUTING.md
[CC-BY 4.0]: https://opendefinition.org/licenses/cc-by/
[MIT]: https://opensource.org/licenses/MIT
[Deep Dive TiKV]: https://tikv.org/deep-dive/introduction/
[Distributed Systems in Rust]: https://github.com/pingcap/talent-plan/tree/master/courses/dss
[NewSQL]: https://en.wikipedia.org/wiki/NewSQL
[NoSQL]: https://www.thoughtworks.com/insights/blog/nosql-databases-overview
[PingCAP]: https://pingcap.com/
[SQL]: https://en.wikipedia.org/wiki/SQL
[The Rust Book]: https://doc.rust-lang.org/book/
[The Rust Book]: https://doc.rust-lang.org/stable/book/
[TiKV]: https://github.com/tikv/tikv/
[asynchronous programming]: todo
[bitcask]: https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf
[building a command-line program]: https://doc.rust-lang.org/stable/book/ch12-00-an-io-project.html
[building a multithreaded web server]: https://doc.rust-lang.org/stable/book/ch20-00-final-project-a-web-server.html
[clippy]: https://github.com/rust-lang/rust-clippy/
[criterion]: https://github.com/bheisler/criterion.rs
[crossbeam]: https://github.com/crossbeam-rs/crossbeam
[file issues]: https://github.com/pingcap/talent-plan/issues/
[futures]: https://docs.rs/futures/0.1.27/futures/
[git]: https://git-scm.com/
[kv]: https://en.wikipedia.org/wiki/Key-value_database
[parallel programming]: todo
[plan]: ./docs/lesson-plan.md
[post-project surveys]: ./docs/lesson-plan.md#user-content-making-pna-rust-better
[pre]: ./docs/prerequisites.md
[programming a guessing game]: https://doc.rust-lang.org/stable/book/ch02-00-guessing-game-tutorial.html
[rustfmt]: https://github.com/rust-lang/rustfmt/
[serde]: https://github.com/serde-rs/serde
[series of courses]: https://github.com/pingcap/talent-plan/
[sp]: https://en.wikipedia.org/wiki/System_programming
[the roadmap]: ./docs/roadmap.md
[tokio]: https://github.com/tokio-rs/tokio
[Rust]: https://www.rust-lang.org/