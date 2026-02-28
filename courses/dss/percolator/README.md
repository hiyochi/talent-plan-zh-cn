# Percolator 实验

## 什么是 Percolator

Percolator 是 Google 构建的一个系统，用于在超大规模数据集上进行增量处理。Percolator 还提供了一个分布式事务协议，支持 ACID 快照隔离语义。你可以在这篇论文中找到更多细节：[使用分布式事务和通知进行大规模增量处理](https://storage.googleapis.com/pub-tools-public-publication-data/pdf/36726.pdf)。

## 实验前提条件

要开始本实验，你需要满足以下前提条件：

1. 熟悉 Rust（你也可以从我们的 Rust 培训课程中学到一些知识）

2. 了解 protobuf 的工作原理

3. 对 RPC 的工作原理有基本的了解

4. 对分布式事务有基本的了解

## 实验概念

### 服务器

本实验中有两种服务器，它们提供不同的服务：TSO 服务器和存储服务器。

#### TSO 服务器

Percolator 依赖于一个名为*时间戳预言机*的服务。由 `TimestampOracle` 实现的 TSO 服务器可以按严格递增的顺序生成时间戳。所有事务都需要获取唯一的时间戳来指示执行顺序。

#### 存储服务器

Percolator 构建在 Bigtable 之上，Bigtable 向用户呈现一个多维排序映射。在本实验中，由 `MemoryStorage` 实现的存储服务器用于模拟 Bigtable，它包含三列。这些由 `BTreeMap` 实现的列类似于 Bigtable 中的列。特别是，`MemoryStorage` 有三列：`Write`、`Data`、`Lock`，以保持与 Bigtable 的一致性。

此外，存储还需要提供基本的 `read`、`write` 和 `erase` 操作来操作其中存储的数据。

### 客户端

客户端将 `begin` 一个事务，该事务包含一组操作，如 `get` 和 `set`，并调用 `commit` 来提交事务。此外，客户端还将调用 `get_timestamp` 来获取时间戳。

更多实现细节可以在论文中找到。

## 编写你自己的实现

本项目中有一些注释，如“此处为你的定义”或“此处为你的代码”。你需要根据论文自行编写代码。没有太多严格的限制，因此你可以根据需要定义任意数量的变量，无论是在 *struct* 还是 *proto* 中。

## 测试你的工作

你可以直接在当前目录中运行以下命令：

```sh
make test_percolator
```