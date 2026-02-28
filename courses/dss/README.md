# Rust 中的分布式系统

本课程是关于在 [Rust] 中实现分布式系统的培训课程。

涵盖的主题包括：

- [Raft 一致性算法]（包括基于 Raft 构建的容错键值存储服务）
- [Percolator 事务模型]

完成本课程后，您将具备在 Rust 中实现具有事务支持和容错能力的简易键值存储服务所需的知识。

**重要提示：Rust 中的分布式系统目前处于 Alpha 阶段**  
可能存在一些 Bug。我们非常欢迎您提供反馈！如遇任何问题，请[提交 Issue]。同时，我们也鼓励您自行修复问题并提交 Pull Request。

## 本课程的目标

本课程旨在帮助对分布式系统感兴趣的 Rust 程序员，了解如何构建可靠的分布式系统，以及如何实现分布式事务。

## 适合谁？

本课程面向具备丰富经验的 _Rust_ 程序员，要求您已熟悉 Rust 语言。如果您尚未掌握 Rust，建议先学习我们的 [rust] 课程。

## PingCAP 特别说明

本课程与 [Deep Dive TiKV] 结合使用，足以使程序员能够对 [TiKV] 项目做出有意义的贡献。本课程特别为中文 Rust 社区设计，旨在帮助他们掌握足够多的 Rust 知识以参与 TiKV 开发。课程语言力求简洁，以便英语阅读能力有限的学习者也能顺利理解。如您发现任何语言表达难以理解，请[提交 Issue]。

## 许可证

[CC-BY 4.0](https://opendefinition.org/licenses/cc-by/)

<!-- 链接 -->
[rust]: ../rust/README.md
[file issues]: https://github.com/pingcap/talent-plan/issues/
[Deep Dive TiKV]: https://tikv.github.io/deep-dive-tikv/overview/introduction.html
[TiKV]: https://github.com/tikv/tikv/
[Rust]: https://www.rust-lang.org/
[Raft 一致性算法]: raft/README.md
[Percolator 事务模型]: percolator/README.md