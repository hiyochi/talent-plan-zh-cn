# PNA Rust — 构建模块 2

让我们学习一些构建模块！

把其他项目和杂务都放到一边，深呼吸，放松一下。这里有一些有趣的资源供你探索。

请仔细阅读所有材料，并完成所有练习。

- **[阅读：Damn Cool Algorithms：日志结构化存储][lss]**。对日志结构化存储基本概念的简单概述。日志结构化存储算法有很多，但本文所描述的并非你将要使用的那种。

- **[阅读：日志结构化文件系统的设计与实现][lsfs]**。一篇具有影响力的论文。

- **[阅读：Bitcask：用于快速键值数据的日志结构化哈希表][bc]**。一种简单而有效的键值数据库设计，它使用了日志结构化存储。

- **[阅读：Rust 中的错误处理][e]**。Rust 的错误处理功能强大，许多 Rust 程序员一旦掌握后就爱不释手。但它复杂，且有着复杂的发展历程。这是一篇关于 Rust 错误处理最佳实践的经典深度文章。该文发布于 2015 年，此后错误处理机制虽有小幅调整，但其中蕴含大量智慧。作者 [BurntSushi] 对 Rust 错误处理进行了大量实验，被公认为该领域及[其他方面]的权威。

- **[阅读：`std::collections`][c]**。作为系统程序员，必须充分了解各种数据结构的行为（即使不一定要了解其具体实现）。Rust 标准库的 `collections` 模块对计算机科学中最常见的几种集合类型的权衡取舍提供了极为出色的概述。本部分只需阅读模块文档即可。

- **[阅读：`std::io`][io]**。同样，你必须熟悉你的 I/O 工具。虽然 Rust 的 `io` 模块文档不如 `collections` 那样精彩，但仍能为你提供工具集的全面概览。你只需阅读模块文档即可。

- **练习：使用 `serde`（JSON）序列化和反序列化一个数据结构**。

  本练习及接下来两个练习将介绍使用 [`serde`] 进行基础序列化与反序列化。`serde` 序列化速度快、使用简便，同时具备可扩展性和表达力。

  为你的可序列化数据结构设想一个平坦的游戏平面，上面布满网格状方格，类似国际象棋棋盘。假设你有一个游戏角色，每回合可沿单一方向移动任意数量的方格。定义一个类型 `Move`，表示该角色的一次移动。

  派生 [`Debug`] 特性，以便使用 `{:?}` 格式说明符轻松打印 `Move`。

  编写一个 `main` 函数，定义一个类型为 `Move` 的变量 `a`，使用 [`serde`] 将其序列化到一个 [`File`] 中，然后再反序列化回另一个类型为 `Move` 的变量 `b`。

  使用 [JSON] 作为序列化格式。

  使用 `println!` 和 `{:?}` 格式说明符打印 `a` 和 `b`，以验证反序列化成功。

  注意：`serde` 官方文档提供了许多[示例]可供参考。

- **练习：使用 `serde`（RON）将数据结构序列化和反序列化到缓冲区**。

  与上一练习相同，但这次不是序列化到 `File`，而是序列化到一个 `Vec<u8>` 缓冲区，并尝试使用 [RON] 替代 JSON 作为格式。序列化到 `Vec` 而非 `File` 是否有差异？使用 RON 库与 JSON 库相比又有什么不同？

  使用 [`str::from_utf8`] 将 `Vec<u8>` 转换为 `String`，并解包结果，然后打印出该序列化后的字符串表示形式，观察 `Move` 被序列化为 RON 格式时的样子。

- **练习：使用 `serde`（BSON）序列化和反序列化 1000 个数据结构**。

  这个练习略有不同。前两个练习是将单个值序列化和反序列化到缓冲区，而本练习需将 1000 个不同的 `Move` 值连续序列化到一个文件中，然后再反序列化回来。这次使用 [BSON] 格式。

  你需要探索的问题包括：`serde` 是否会自动维护正确的文件偏移量（“游标”）以连续反序列化多个值？还是你需要为每个值自行定义“帧”以标明其大小？以及如何检测文件末尾是否已无更多可解析的值。

  成功将多个值序列化和反序列化到文件后，再尝试在 `Vec<u8>` 上进行相同操作。序列化和反序列化通常要求目标实现 [`Write`] 和 [`Read`] 特性。`Vec<u8>` 是否实现了其中任一或两者？这些实现的行为是怎样的？你可能需要将缓冲区包装在实现了这些特性的封装类型中，才能获得正确的行为 —— 这些特性的 API 文档列出了标准库中所有实现者，你需要的类型一定就在其中。

[`File`]: https://doc.rust-lang.org/std/fs/struct.File.html
[`Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
[`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
[BSON]: https://github.com/zonyitoo/bson-rs
[RON]: https://github.com/ron-rs/ron
[`str::from_utf8`]: https://doc.rust-lang.org/std/str/fn.from_utf8.html
[JSON]: https://github.com/serde-rs/json
[`Debug`]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
[examples]: https://serde.rs/examples.html
[`serde`]: https://serde.rs/
[lss]: http://blog.notdot.net/2009/12/Damn-Cool-Algorithms-Log-structured-storage
[lsfs]: https://people.eecs.berkeley.edu/~brewer/cs262/LFS.pdf
[io]: https://doc.rust-lang.org/std/io/
[c]: https://doc.rust-lang.org/std/collections/
[e]: https://blog.burntsushi.net/rust-error-handling/
[bc]: https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf
[BurntSushi]: https://github.com/BurntSushi
[other things]: https://github.com/BurntSushi/ripgrep

<!-- TODO: 更好的 LSS 论文 -->
<!-- TODO: 希望有一篇非维基百科的、关于数据库和/或键值数据库工作原理的综述性文章 -->