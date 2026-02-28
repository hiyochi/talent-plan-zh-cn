# PNA Rust — 构建模块 3

让我们学习一些构建模块！

把其他项目和杂务放在一边，深呼吸，放松一下。这里有一些有趣的资源供你探索。

请阅读所有材料并完成所有练习。

- **[阅读：`log` crate API][l]**。Rust 原生的日志库。只需阅读 crate 级别的文档（首页）。你可能需要点击小的 `[+]` 或 `[-]` 按钮来展开文档内容。这将帮助你了解 Rust 中日志的工作方式。

- **[阅读：`slog` crate API][sl]**。另一个流行的日志库，专为“结构化日志”设计。同样，只需阅读 crate 级别的文档，与 `log` 进行对比。你也可以查看 ["使用 slog 进行结构化日志的入门"][sli]。

- **[阅读：结构化日志 vs 基础日志的优势][lvsl]**。一篇关于传统文本行日志与结构化日志之间差异的 StackOverflow 讨论。

- **[阅读：Redis 协议规范][rp]**。[Redis]（一个内存键值存储）的协议规范。思考他们的设计优先级是什么。阅读时，最好同时参考 Redis 的 [命令列表]。

- **练习：使用 `std::io` 编写一个 Redis ping-pong 客户端和服务器**。编写一个简单的客户端和服务器，使用 Redis 协议通信：客户端发送 [PING] 命令，服务器做出正确响应。使用 [`std::io`] API 直接读写字节。你的客户端能否与真实的 Redis 服务器通信？

- **练习：使用序列化消息编写一个 Redis ping-pong 客户端和服务器**。与上一练习相同，但这次使用类型定义协议，并编写一个 [`serde` 数据格式][df]，通过序列化间接读写消息。

- **[阅读：统计严谨的 Java 性能评估][pe]**。虽然该文专门针对 Java，并讨论了与垃圾回收语言相关的话题，但它很好地展示了如何构建有效基准测试所需的思维方式。

<!-- TODO: 更好的基准测试阅读材料 -->
<!-- TODO: 关于 trait 的内容？ -->

[pe]: https://dri.es/files/oopsla07-georges.pdf
[df]: https://serde.rs/data-format.html
[`std::io`]: https://doc.rust-lang.org/std/io/
[PING]: https://redis.io/commands/ping
[commands]: https://redis.io/commands
[Redis]: https://redis.io/
[rp]: https://redis.io/topics/protocol
[l]: https://docs.rs/log/
[sl]: https://docs.rs/slog/
[sli]: https://github.com/slog-rs/slog/wiki/Introduction-to-structured-logging-with-slog
[lvsl]: https://softwareengineering.stackexchange.com/questions/312197/benefits-of-structured-logging-vs-basic-logging