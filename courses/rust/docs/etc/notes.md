# 实现者笔记

## 希望涵盖的主题

这份主题列表比 README 中的内容更全面。

- 错误处理
  - 简单与复杂错误处理，Fail 与 StdError 等
  - `fn main() -> Result`
  - `panic!` 和栈展开（unwinding）
- 使用 log 和 slog 进行日志记录
  - env_logger 的工作原理？
- 树结构 vs 映射结构
- 异步 vs 同步网络编程
  - `std` 网络库
  - TCP vs UDP
  - `reqwest`
  - 使用 Iron 实现阻塞式 HTTP 服务
- 同步文件 I/O 及阻塞问题的解决方案
- 缓冲 I/O vs 非缓冲 I/O
- 基准测试：criterion 和 critcmp，black_box
- RUST_BACKTRACE
- 提问渠道
- futures
- tokio
- mio？
- async/await？——可能留待后续迭代
- Pin？
- 泛型占位符惯用法：`let foo: Vec<_> =`
- 使用迭代器构造数据的惯用法
- semver 技巧
- impl trait，以及 `Into<Option<_>>` 技巧
- Rust 历史、文化和设计原则
- rustfmt、clippy 及其配置
- Rust 2018？
  - 我们默认使用 Rust 2018，除非必要，否则不提及 Rust 2015
- 大多数 Rust 程序员应掌握的工具
- 构建脚本（build scripts）
  - Protocol Buffers 编译示例
  - 获取 rustc 版本
  - 深度解析依赖构建脚本的 crate 示例
- 使用 RUSTFLAGS
- 调试
- 性能分析（profiling）
- 如何避免用其他语言的不良习惯编写 Rust 代码
  - Ana 喜欢这个主题
- 变量遮蔽（shadowing）
- 动态大小类型（DSTs）
- 配置 clippy / rustfmt
- 为 CI 编写 clippy / rustfmt 脚本
- CI 环境搭建
- 何时使用何种结构体类型、impl 模式、构造函数模式、析构函数、repr 属性、
  内存对齐演示、紧凑结构体（packed structs）、深入讲解大小与对齐、
  枚举实现及其优化
- 导入 crate、特性（features）、调试和修复依赖项、
  std 与 crate 的哲学与历史、如何查找 crate
- 测试机制是如何工作的？
- `cargo run` 做了什么？
- 深入解析 cargo / rustc 包装模式（例如 rustup、`RUSTC_WRAPPER`）
- 格式化技巧、深入讲解 `derive Debug`、`format!` 的工作原理
- 可变别名 bug，所有权如何防止可变别名
- Sync / Send、唯一引用 / 共享引用 vs 不可变 / 可变
- `Rc` 和 `Arc`、内部可变性（interior mutability）深度解析
- 何时使用传值，移动操作的性能影响
- 持有引用的结构体
- 共享状态 vs 消息传递
- 线程池
- 工作区（workspaces）
- 重构
- https://github.com/altsysrq/proptest
- 模糊测试（fuzzing）
- dbg!
- 变量遮蔽（shadowing）

## 参考资料来源

- https://pdos.csail.mit.edu/6.824/schedule.html
  - 本课程受其启发，并旨在作为其前置课程
- https://github.com/ferrous-systems/rust-three-days-course
- https://github.com/nrc/talks
- RustBridge

## 教学方法

- https://launchschool.com/pedagogy
- https://launchschool.com/is_this_for_me

## 阅读材料来源

- 之前的阅读建议：https://github.com/pingcap/talent-plan/blob/32311e6999a2a5b7db25cd2b4dd96491d5181165/rust/plan.md
- http://highscalability.com/blog/2011/1/10/riaks-bitcask-a-log-structured-hash-table-for-fast-keyvalue.html
- https://github.com/brson/rust-anthology/blob/master/master-list.md
- https://github.com/ctjhoa/rust-learning
- https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf
- https://rust-lang-nursery.github.io/failure/
- https://serde.rs/
- https://doc.rust-lang.org/cargo/
- https://medium.com/rabiprasadpadhy/google-spanner-a-newsql-journey-or-beginning-of-the-end-of-the-nosql-era-3785be8e5c38
- https://github.com/Hexilee/async-io-demo
- https://stjepang.github.io/2019/01/29/lock-free-rust-crossbeam-in-2019.html
- https://limpet.net/mbrubeck/2019/02/07/rust-a-unique-perspective.html
- https://doc.rust-lang.org/nomicon/aliasing.html
- https://manishearth.github.io/blog/2015/05/17/the-problem-with-shared-mutability/
- https://www.youtube.com/watch?v=9_3krAQtD2k （futures async await）
- https://shipilev.net/blog/2014/java-scala-divided-we-fail/
- https://www.internalpointers.com/post/lock-free-multithreading-atomic-operations

## 练习来源

- https://github.com/rust-lang/rustlings
- https://exercism.io/tracks/rust
- https://doc.rust-lang.org/rust-by-example/index.html

## 可能需要删减的主题

- 并行性章节
- 格式化课程
- 构建时间课程
- 集合与迭代器

## 评分机制

- 基于文本的自动化作弊检测
- 测试用例的自动化评分
- 通过 Python 脚本对非单元测试要求进行自动化评分
- 如何对开放式答案进行评分？

## 待办事项（TODO）

- 调研其他资料的主题编排顺序
- 课程和实验应提出问题
- 请中文母语者识别并替换“难懂”的词汇和短语
- 在某个地方说明我们仅使用 Rust 2018，以及如何验证
- 将 slides.html 中的 URL 改为链接到托管文件
- 增加更多关于如何处理项目的说明
  - 注明鼓励超出项目范围的改进，并加以注释
- 为每个项目添加目录（TOC）
- 使用 fail-rs 进行一致性测试
- 注明每个项目各部分应重点关注的测试名称
- 尝试通过 KvsEngine trait 使用特化（specialization）
- 使用命令式“部分”标题？如“creating” vs “create”
- 在并行性 / 线程池项目中增加对“阻塞”的解释
- 根据 README 要求，让并行性项目“有趣且万无一失”
- 根据 README 要求，整合 critcmp
- 从 README 中删除的内容：
  - 构建脚本及其与运行时之间的交互
  - 深入理解语言和库的内部工作原理
  - “深入底层”以理解 Rust 为何如此工作
  - 课程可离线交付
  - _课程_：聚焦于编写对应项目类型实用软件所需的主题，
    包括高级技巧、最佳实践和深度解析。形式为幻灯片、
    演讲笔记和简短的_说明文档_。
  - 删除了“教会如何自行查找 Rust 相关信息”的目标
    - 现为隐含目标
- 更清晰地说明 crate 依赖关系
- 统一 crate 名称的格式（是否使用代码格式？）（是，当未明确提及时应突出显示为 crate）
- 更好地识别并解释所解决的各个问题
- 增加查找文档和 crate 的技巧
- 首次链接到书籍时提及 mdbook
- rustup doc 和 cargo doc 及其内容来源
- 使项目内容更一致
  - 项目包含与主题无关的部分
- 将 project.md 改为 description.md，避免路径重复
- 用博客文章、论文等更具体、更难找到的资源替代维基百科、docs.rs、官方 Rust 文档和阅读材料
- 创建提交解决方案的位置
- 比较 criterion 与 bench
- 将所有链接移至文档末尾，并在 contributing.md 中说明
- 对链接行进行排序
- 如何查找所有 lint 的名称和描述
- 考虑在某个地方添加指向项目源代码的链接
- 添加鼓励阅读 rustfmt / rustclippy 文档的说明
- 将“_Note: ...”替换为“_**Note**: ...”
- 更详细地描述最终成果
- 在每个项目顶部添加关于构建模块的提醒
- 以某种非侵入方式添加反向链接
- 在 p1 中仅添加一个有用的 Rust CLI 工具列表！
- 对清单文件中的依赖项进行排序
- 在 p1 中首次使用 crate 名称时链接到 docs.rs
- 说明我们进行每个练习或拓展的原因
- 在 crates.io 上发布
- critcmp

# 未来项目规划

- 5 - 使用 futures + HTTP，采用标准 HTTP 基准测试工具
- 6 - 使用 async/await
- 7 - 使用 gRPC
- 8 - 聚焦数据完整性
- 9 - 尝试匹配生产组件的性能
- ? - 带生命周期的流式扫描操作与流式网络 API

# 调研

- 完成所需时间