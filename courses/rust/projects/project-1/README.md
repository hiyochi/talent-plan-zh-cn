# PNA Rust 项目 1：Rust 工具箱

**任务**：创建一个内存中的键/值存储，能够通过简单测试并响应命令行参数。

**目标**：

- 安装 Rust 编译器及相关工具
- 学习本课程使用的项目结构
- 使用 `cargo init` / `run` / `test` / `clippy` / `fmt`
- 学习如何查找并导入 crates.io 上的 crate
- 为键值存储定义合适的数据类型

**主题**：测试、`clap` crate、`CARGO_VERSION` 等环境变量、`clippy` 和 `rustfmt` 工具。

**扩展**：`structopt` crate。

- [简介](#user-content-introduction)
- [项目规范](#user-content-project-spec)
- [安装](#user-content-installation)
- [项目设置](#user-content-project-setup)
- [第 1 部分：让测试能够编译](#user-content-part-1-make-the-tests-compile)
  - [附注：测试技巧](#user-content-aside-testing-tips)
- [第 2 部分：接受命令行参数](#user-content-part-2-accept-command-line-arguments)
- [第 3 部分：Cargo 环境变量](#user-content-part-3-cargo-environment-variables)
- [第 4 部分：在内存中存储值](#user-content-part-4-store-values-in-memory)
- [第 5 部分：文档](#user-content-part-5-documentation)
- [第 6 部分：使用 `clippy` 和 `rustfmt` 确保良好风格](#user-content-part-6-ensure-good-style-with-clippy-and-rustfmt)
- [扩展 1：`structopt`](#user-content-extension-1-structopt)

## 简介

在本项目中，你将创建一个简单的内存键/值存储，它将字符串映射到字符串，并通过一些测试，同时响应命令行参数。本项目的重点是典型的 Rust 项目所需的工具和设置。

如果你觉得这听起来很基础，请仍然完成该项目，因为它讨论了一些将在整个课程中使用的通用模式。

## 项目规范

Cargo 项目 `kvs` 构建一个名为 `kvs` 的命令行键值存储客户端，该客户端反过来调用一个名为 `kvs` 的库。

`kvs` 可执行文件支持以下命令行参数：

- `kvs set <KEY> <VALUE>`

  将字符串键的值设置为字符串

- `kvs get <KEY>`

  获取给定字符串键的字符串值

- `kvs rm <KEY>`

  删除给定的键

- `kvs -V`

  打印版本信息

`kvs` 库包含一个类型 `KvStore`，它支持以下方法：

- `KvStore::set(&mut self, key: String, value: String)`

  将字符串键的值设置为字符串

- `KvStore::get(&self, key: String) -> Option<String>`

  获取字符串键的字符串值。如果键不存在，则返回 `None`。

- `KvStore::remove(&mut self, key: String)`

  删除给定的键。

`KvStore` 类型在内存中存储值，因此命令行客户端除了打印版本外几乎不能做更多事情。当从命令行运行时，`get` / `set` / `rm` 命令将返回“未实现”错误。未来的项目将把值存储在磁盘上，并具有可用的命令行界面。

## 安装

在目前的 Rust 编程经验中，你应该知道如何通过 [rustup] 安装 Rust。

[rustup]: https://www.rustup.rs

如果你还没有安装，请现在运行以下命令进行安装：

```
curl https://sh.rustup.rs -sSf | sh
```

（如果你在 Windows 上运行，请按照 rustup.rs 上的说明操作。但请注意，你在本课程中可能会面临比其他同学更多的挑战，因为本课程是在 Unix 系统上开发的。总的来说，Windows 上的 Rust 开发体验不如 Unix 系统完善）。

通过输入 `rustc -V` 验证工具链是否正常工作。如果不起作用，请注销并重新登录，以便安装过程中对登录配置文件所做的更改生效。

## 项目设置

你将在自己的 git 仓库中完成此项目的工作，使用自己的 Cargo 项目。你将从[本课程的源代码仓库][course]导入项目的测试用例。

[course]: https://github.com/pingcap/talent-plan

请注意，在该仓库中，与本课程相关的内容都在 `rust` 子目录中。你可以忽略任何其他目录。

本课程中的项目既包含库也包含可执行文件。它们是可执行文件，因为我们要开发一个可以运行的应用程序。它们是库，因为提供的测试用例必须链接到它们。

我们将在本课程的每个项目中使用相同的设置。

我们将使用的目录结构如下：

```
├── Cargo.toml
├── src
│   ├── bin
│   │   └── kvs.rs
│   └── lib.rs
└── tests
    └── tests.rs
```

`Cargo.toml`、`lib.rs` 和 `kvs.rs` 文件的内容如下：

`Cargo.toml`：

```toml
[package]
name = "kvs"
version = "0.1.0"
authors = ["Brian Anderson <andersrb@gmail.com>"]
description = "A key-value store"
edition = "2018"
```

`lib.rs`：

```rust
// 现在先让它保持为空
```

`kvs.rs`：

```rust
fn main() {
    println!("Hello, world!");
}
```

作者应该是你自己，但名称必须是 `kvs`，以便测试用例能够正常工作。这是因为项目名称也是它包含的库的名称。同样，二进制文件（命令行应用程序）的名称也必须是 `kvs`。在上面的设置中，它将隐式地基于文件名成为 `kvs`，但你可以通过在清单（`Cargo.toml`）中放置适当的信息来命名文件为你想要的任何名称。

你可以使用 `cargo new --lib`、`cargo init --lib`（在空目录中）或手动设置此项目。你可能还希望在同一个目录中初始化一个 git 仓库。

最后，`tests` 目录是从课程材料中复制的。在这种情况下，从课程仓库复制文件 `rust/projects/project-1/tests` 到你自己的仓库，作为 `tests`。

此时，你应该能够使用 `cargo run` 运行程序。

_现在就试试。_

你已经为此项目设置好了环境，可以开始动手了。

## 第 1 部分：让测试能够编译

你已经获得了位于 `tests/tests.rs` 的一组单元测试。打开它看看。

_尝试使用 `cargo test` 运行测试。_ 发生了什么？为什么？

你在此项目中的第一个任务是让测试_能够编译_。有趣吧！

如果你的项目像我的项目一样，你可能看到了大量的构建错误。看看前几个错误。一般来说，当你看到一堆错误时，前几个是最重要的——`rustc` 即使在遇到错误后仍会继续尝试编译，因此错误可能会级联，后面的错误往往没有太大意义。你的前几个错误可能看起来像这样：

```
error[E0433]: failed to resolve: use of undeclared type or module `assert_cmd`
 --> tests/tests.rs:1:5
  |
1 | use assert_cmd::prelude::*;
  |     ^^^^^^^^^^ use of undeclared type or module `assert_cmd`

error[E0432]: unresolved import
 --> tests/tests.rs:3:5
  |
3 | use predicates::str::contains;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^
```

（如果你看到的是其他内容，请提交一个问题）。

这两个错误对于 Rust 新手来说很难诊断，所以我就直接告诉你这里发生了什么：你的清单中缺少[开发依赖] crate。

[dev-dependency]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#development-dependencies

对于此项目，你的 `Cargo.toml` 文件需要包含这些行：

```toml
[dev-dependencies]
assert_cmd = "0.11.0"
predicates = "1.0.0"
```

这些依赖的细节对你完成项目并不重要，但你可能想自己研究一下它们。我们没有提前告诉你需要开发依赖，就是为了让你自己体验这些错误。在未来的项目中，设置文本会告诉你需要的开发依赖。

一个快速提示：你如何判断这些错误是由于清单中缺少依赖而不是源代码中的错误？这里有一个很大的线索，来自前面显示的错误：

```
1 | use assert_cmd::prelude::*;
  |     ^^^^^^^^^^ use of undeclared type or module `assert_cmd`
```

在 `use` 语句中，第一个路径元素始终是 crate 的名称。例外情况是当第一个路径元素引用了之前通过_另一个_ `use` 语句带入作用域的名称时。换句话说，如果这个文件中还有另一个 `use` 语句，如 `use foo::assert_cmd`，那么 `use assert_cmd::prelude::*` 将引用_那个_ `assert_cmd`。关于这一点还可以说更多，但我们不应该在这里深入探讨路径解析的细微差别。只需知道，一般来说，在 `use` 语句中，如果路径中的第一个元素找不到（即无法解析），问题很可能是该 crate 没有在清单中命名。

唉。这是在第一个项目中的一个不幸的偏离。但希望是有启发性的。

_继续并在你的清单中添加适当的开发依赖。_

再次尝试使用 `cargo test` 运行测试。发生了什么？为什么？

希望那些_之前的_错误已经消失了。现在所有的错误都是关于测试用例无法在你的代码中找到它期望的所有代码。

_所以现在你的任务是概述所有必要的类型、方法等，以使测试能够构建。_

在本课程中，你将经常阅读测试用例。测试用例准确地告诉了你代码应该做什么。如果文本和测试不一致，测试是正确的（提交一个 bug！）。在现实世界中也是如此。测试用例展示了软件_实际_做什么。它们是现实。习惯于阅读测试用例。

而且，额外的好处——测试用例通常是任何项目中最糟糕的代码，草率且没有文档。

再次尝试使用 `cargo test` 运行测试。发生了什么？为什么？

在 `src/lib.rs` 中编写必要的类型和方法定义，以使 `cargo test --no-run` 成功完成。现在不要编写任何方法体——而是编写 `panic!()`。这是在不知道或不关心实现的情况下勾勒出你的 API 的方法（还有 [`unimplemented!`] 宏，但由于输入它更长，通常只是使用 `panic!`，一个可能的例外是如果你正在发布包含未实现方法的软件）。

[`unimplemented!`]: https://doc.rust-lang.org/std/macro.unimplemented.html

_在继续之前现在就做那件事。_

一旦完成，如果你运行 `cargo test`（不带 `--no-run`），你应该看到一些测试失败了，像这样：

```
    Finished dev [unoptimized + debuginfo] target(s) in 2.32s
     Running target/debug/deps/kvs-b03a01e7008067f6

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/debug/deps/kvs-a3b5a004932c6715

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/debug/deps/tests-5e1c2e20bd1fa377

running 13 tests
test cli_get ... FAILED
test cli_invalid_get ... FAILED
test cli_invalid_rm ... FAILED
test cli_invalid_set ... FAILED
test cli_no_args ... FAILED
test cli_invalid_subcommand ... FAILED
... more lines of spew ...
```

...后面跟着更多行。这很好！这正是我们现在所需要的。你将在本项目的其余部分中让这些测试通过。

### 附注：测试技巧

如果你再次查看 `cargo test` 的输出，你会看到一些有趣的东西：

```
     Running target/debug/deps/kvs-b03a01e7008067f6

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/debug/deps/kvs-a3b5a004932c6715

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/debug/deps/tests-5e1c2e20bd1fa377


running 13 tests
test cli_get ... FAILED
```

Cargo 说了三次“Running ...”。前两次实际上没有运行任何测试。而且，如果所有这些测试都没有失败，cargo 还会运行_另一组_测试。

为什么会这样？

这是因为在 Rust 中你可以在很多地方编写测试：

- 在你的库的源代码内部
- 在你的每个二进制文件的源代码内部
- 在每个测试文件中
- 在你的库的文档注释中

而 cargo 不知道这些地方哪些实际上包含测试，所以它只是构建并运行它们全部。

所以那两组空测试：

```
     Running target/debug/deps/kvs-b03a01e7008067f6
running 0 tests
     Running target/debug/deps/kvs-a3b5a004932c6715
running 0 tests
```

这有点令人困惑，但其中一个是你的库，为测试而编译，另一个是你的二进制文件，为测试而编译。两者都不包含任何测试。两者名称中都有“kvs”的原因是，你的库和你的二进制文件都被称为“kvs”。

所有这些测试输出很烦人，有两种方法可以让 cargo 安静下来：使用命令行参数，以及更改清单。

以下是相关的命令行标志：

- `cargo test --lib` —— 只测试库内部的测试
- `cargo test --doc` —— 测试库中的文档测试
- `cargo test --bins` —— 测试项目中的所有二进制文件
- `cargo test --bin foo` —— 只测试 `foo` 二进制文件
- `cargo test --test foo` —— 测试测试文件 `foo` 中的测试

这些很方便快速隐藏测试输出，但如果一个项目不包含某种类型的测试，最好不要处理它们。如果你还记得 Cargo Book 的[清单描述][m]，可以应用两个键：`test = false` 和 `doctest = false`。它们放在 `[lib]` 和 `[[bin]]` 部分。考虑更新你的清单。

[m]: https://doc.rust-lang.org/cargo/reference/manifest.html

另一个快速的事情，如果你之前没有做过的话。运行这个：

```
cargo test -- --help
```

就做吧。很酷。你看到的是_包含你编译的测试的可执行文件_的帮助信息（那个被空格包围的 `--` 告诉 cargo 将所有后续参数传递给测试二进制文件）。它不是当你运行 `cargo test --help` 时显示的信息。这是两个不同的东西：cargo 通过向你的测试二进制文件传递所有这些各种参数来运行它。

如果你想，你可以做完全相同的事情。让我们再回到我们的 `cargo test` 示例。我们看到了这一行：

```
     Running target/debug/deps/kvs-b03a01e7008067f6
```

那是 cargo 告诉你测试二进制文件的名称。你可以自己运行它，像 `target/debug/deps/kvs-b03a01e7008067f6 --help`。

`target` 目录包含很多酷东西。浏览它可以教会你很多关于 Rust 工具链实际在做什么。

在实践中，特别是对于大型项目，在开发单个功能时你不会运行整个测试套件。为了缩小到我们关心的测试集，运行以下命令：

```
cargo test cli_no_args
```

这将运行名为 `cli_no_args` 的测试。事实上，它将运行任何名称中包含 `cli_no_args` 的测试，所以如果，例如，你想运行所有 CLI 测试，你可以运行 `cargo test cli`。这可能就是你在完成项目过程中自己运行测试的方式，否则你会被许多你还没有修复的失败测试分散注意力。不幸的是，这种模式是简单的子字符串匹配，而不是像正则表达式那样的花哨东西。

<!-- TODO: 需要借口来解释 `cargo test --test suite`。
根据 https://github.com/pingcap/talent-plan/pull/129#issuecomment-498477590
我们可能会按项目部分将测试套件组织到测试文件中。
然后我们可以谈论 `cargo test --test part-1` -->

请注意，在撰写本文时，本课程项目的测试用例没有以明确显示哪些测试用例应该对项目的任何特定部分完成的方式组织——只有在最后整个套件应该通过。你需要阅读测试的名称和实现，以弄清楚你_认为_哪些应该在特定时间通过。

## 第 2 部分：接受命令行参数

在本课程中，键/值存储都通过命令行客户端控制。在此项目中，命令行客户端非常简单，因为键值存储的状态只存储在内存中，而不是持久化到磁盘。

在这一部分，你将使 `cli_*` 测试用例通过。

回忆一下如何从前面的小节运行单个测试用例。

同样，CLI 的接口是：

- `kvs set <KEY> <VALUE>`

  将字符串键的值设置为字符串

- `kvs get <KEY>`

  获取给定字符串键的字符串值

- `kvs rm <KEY>`

  删除给定的键

- `kvs -V`

  打印版本

但在这个迭代中，`get` 和 `set` 命令将向 stderr 打印字符串“unimplemented”，并以非零退出代码退出，表示错误。

你将使用 `clap` crate 来处理命令行参数。

_找到 `clap` crate 的最新版本并将其添加到 `Cargo.toml` 中的依赖项。_ 有几种方法可以找到并导入一个 crate，但专业提示：查看内置的 [`cargo search`] 和插件 [`cargo edit`]。

[`cargo search`]: https://doc.rust-lang.org/cargo/commands/cargo-search.html
[`cargo edit`]: https://github.com/killercup/cargo-edit

<!-- note: 上面基本上是假设到现在为止他们知道 crates.io，所以用两个 CLI 命令扩展他们的世界。 -->

<i>接下来使用 [crates.io]、[lib.rs] 或 [docs.rs] 找到 `clap` crate 的文档，并实现命令行接口，使 `cli_*` 测试用例通过。</i>

<!-- note: 上面随意提及 lib.rs 和 docs.rs 以确保他们知道。 -->

当你测试时，使用 `cargo run`；不要直接从 `target/` 目录运行可执行文件。当向程序传递参数时，用两个破折号 `--` 将它们与 `cargo run` 命令分开，像 `cargo run -- get key1`。

[crates.io]: https://crates.io
[lib.rs]: https://lib.rs
[docs.rs]: https://docs.rs

## 第 3 部分：Cargo 环境变量

当你设置 `clap` 来解析你的命令行参数时，你可能设置了名称、版本、作者和描述（如果没有，请这样做）。这些信息在 `Cargo.toml` 中提供的值是冗余的。Cargo 设置了可以通过 Rust 源代码在构建时访问的环境变量。

_修改你的 `clap` 设置，从标准的 cargo 环境变量设置这些值。_

## 第 4 部分：在内存中存储值

现在你的命令行脚手架已经完成，让我们转向 `KvStore` 的实现，并使剩余的测试用例通过。

`KvStore` 方法的行为完全通过测试用例本身定义——你不需要任何进一步的描述来完成此项目的代码。

_通过实现 `KvStore` 上的方法使剩余的测试用例通过。_

## 第 5 部分：文档

你已经实现了项目的功能，但在它成为一个可以贡献或发布的精美 Rust 软件之前，还有一些事情要做。

首先，公共项目通常应该有文档注释。

文档注释显示在一个 crate 的 API 文档中。可以使用命令 `cargo doc` 生成 API 文档，它会将它们渲染为 HTML 到 `target/doc` 文件夹。但请注意，`target/doc` 文件夹不包含 `index.html`。在此项目中，你的 crate 文档将位于 `target/doc/kvs/index.html`。你可以使用 `cargo doc --open` 在 web 浏览器中启动该位置。`cargo doc --open` 并不总是有效，例如，如果你通过 ssh 连接到云实例。但如果它不起作用，命令将打印它无法打开的 html 文件的名称——这仅仅是为了找到你的 API 文档的位置很有用。

[好的文档注释][gdc] 不仅仅是重复函数的名称，也不重复从类型签名中获得的信息。它们解释为什么以及如何使用一个函数，成功和失败时返回值是什么，错误和恐慌条件。你编写的库非常简单，所以文档也可以很简单。如果你真的想不出通过文档注释添加任何有用的东西，那么不添加文档注释也可以（这是一个偏好问题）。如果没有文档注释，应该从名称和类型签名本身就可以明显看出如何使用该类型或函数。

文档注释包含示例，这些示例可以使用 `cargo test --doc` 进行测试。

_在 `src/lib.rs` 的顶部添加 `#![deny(missing_docs)]` 以强制所有公共项目都有文档注释。然后在你的库中的类型和方法上添加文档注释。遵循[文档指南][gdc]。给每个一个示例，并确保它们通过 `cargo test --doc`。_

[gdc]: https://rust-lang-nursery.github.io/api-guidelines/documentation.html

## 第 6 部分：使用 `clippy` 和 `rustfmt` 确保良好风格

[`clippy`] 和 [`rustfmt`] 是强制执行常见 Rust 风格的工具。`clippy` 帮助确保代码使用现代习语，并防止通常导致错误的模式。`rustfmt` 强制执行代码格式一致。现在没有必要，但你可能点击那些链接并阅读它们的文档。它们都是复杂的工具，能够做的比下面描述的要多得多。

[`clippy`]: https://github.com/rust-lang/rust-clippy
[`rustfmt`]: https://github.com/rust-lang/rustfmt

这两个工具都包含在 Rust 工具链中，但不是默认安装的。可以使用以下 [`rustup`] 命令安装它们：

```
rustup component add clippy
rustup component add rustfmt
```

[`rustup`]: https://github.com/rust-lang/rustup.rs/blob/master/README.md

_现在就做那件事。_

这两个工具都作为 cargo 子命令调用，`clippy` 作为 `cargo clippy`，`rustfmt` 作为 `cargo fmt`。请注意，`cargo fmt` 会修改你的源代码，所以在运行它之前提交你的工作，以避免意外地进行不需要的更改，之后你可以使用 `git commit --amend` 将这些更改包含在之前的提交中。或者只是将它们作为自己的格式化提交提交——在一系列提交之后，例如，在提交拉取请求之前，通常会对 `clippy` 和 `rustfmt` 都这样做。

_对你的项目运行 `cargo clippy` 并进行任何建议的更改。对你的项目运行 `cargo fmt` 并提交它所做的任何更改。_

值得阅读 [`rustup`]、[`clippy`] 和 [`rustfmt`] 文档，因为这些都是你将频繁使用的工具。

恭喜，你已经完成了第一个项目！如果你愿意，可以完成剩余的“扩展”。它们是可选的。

<!-- TODO 添加关于发现组件和用 rg 过滤的文本 -->

编码愉快，朋友。享受一个美好的休息。

---

<!--

TODO ## 附注：探索 Rust 工具链组件

rust component list
rust component list | rg -v std # 介绍 rg 的机会

-->

## 扩展 1：`structopt`

在此项目中，我们使用 `clap` 来解析命令行参数。通常将程序的解析命令行参数表示为一个结构体，可能命名为 `Config` 或 `Options`。这样做需要在 `clap` 的 `ArgMatches` 类型上调用适当的方法。对于更大的程序，这两个步骤都需要_大量_样板代码。`structopt` crate 通过允许你定义一个 `Config` 结构体，并注解以自动产生一个 `clap` 命令行解析器，该解析器产生该结构体，大大减少了样板。有些人发现这种方法比显式编写 `clap` 代码更好。

_修改你的程序以使用 `structopt` 来解析命令行参数，而不是直接使用 `clap`。_

<!--

## TODOs

- 设置二进制文件的名称
- 询问这种 main.rs 设置的优缺点
  - 解释为什么我们要做这种设置
    （使 main 可测试）尽管这会
    随着他们完成测试而变得明显
- 文档注释
- 确保有足够的背景阅读来支持项目
- 资源（是否/在哪里放置这些？）
  - https://docs.rs/clap/2.32.0/clap/
  - https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo
  - https://rust-lang-nursery.github.io/api-guidelines/documentation.html#documentation
  - https://doc.rust-lang.org/std/macro.env.html
  - https://github.com/rust-lang/rust-clippy/blob/master/README.md
  - https://github.com/rust-lang/rustfmt/blob/master/README.md
- 做范围查找（`scan`）？
- README.md？
- GitHub CI 设置？
- 添加建议阅读 clippy 和 rustfmt 文档
- 使 clippy / rustfmt 文档成为阅读材料？

-->