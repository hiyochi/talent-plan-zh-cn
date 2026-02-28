# PNA Rust — 构建模块 1

让我们学习一些基础构建模块！

把其他项目和杂务暂时放下，深呼吸，放松一下。这里有一些有趣的资源供你探索。

请仔细阅读所有材料，并完成所有练习。

- **[练习：编写一个优秀的 CLI 程序]**。用 Rust 编写一个命令行程序。这将为你在本课程中即将编写的 CLI 程序提供良好的热身，该作者所使用的技术可能与我们推荐的方法形成有趣的对比。请跟随教程，亲手编写相同的代码。你能复现他们的结果吗？

- **[阅读：Cargo 清单格式]**。来自《Cargo 书籍》的单页内容，它将帮助你了解如何根据需要自定义你的项目。作为 Rust 开发者，你将反复回看这一页。

- **[阅读：Cargo 环境变量]**。同样来自《Cargo 书籍》，这也是你未来会多次遇到的页面。环境变量是 Cargo 与 `rustc` 通信的一种方式，允许在构建时为你的程序源码和构建脚本设置各种 [`env!`] 宏。同时，它也是脚本和其他系统与 Cargo 通信的一种途径。

- **[阅读：Rust API 指南：文档]**。Rust 项目对 Rust 源码的编写方式有明确的偏好。本页内容聚焦于如何为 Rust 项目编写文档，但整本书都值得一读。这些指南由经验丰富的 Rust 开发者撰写，目前仍处于不完整状态。请注意它所属的 GitHub 组织 &mdash; [`rust-lang-nursery`]，其中包含许多有趣的项目。

[Reading: Rust API Guidelines: Documentation]: https://rust-lang-nursery.github.io/api-guidelines/documentation.html
[Reading: The Cargo manifest format]: https://doc.rust-lang.org/cargo/reference/manifest.html
[Reading: Cargo environment variables]: https://doc.rust-lang.org/cargo/reference/environment-variables.html
[The Cargo Book]: https://doc.rust-lang.org/cargo/reference/manifest.html
[`env!`]: https://doc.rust-lang.org/std/macro.env.html
[`rust-lang-nursery`]: https://github.com/rust-lang-nursery
[Reading: The rustup documentation]: https://github.com/rust-lang/rustup.rs/blob/master/README.md
[Exercise: Write a Good CLI Program]: https://qiita.com/tigercosmos/items/678f39b1209e60843cc3