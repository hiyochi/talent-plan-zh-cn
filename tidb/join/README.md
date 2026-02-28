## 引言

这是 PingCAP 人才计划在线课程第四周的作业。本作业是 [ACM SIGMOD 编程竞赛 2018](http://sigmod18contest.db.in.tum.de/index.shtml) 的简化版本。

任务是对一组预定义的关系执行批量连接查询。每个连接查询指定两个关系、多个（等值）连接条件，以及一个（求和）聚合操作。挑战在于充分利用 CPU 和内存资源，尽可能快速地执行查询。

**注意：必须使用 go 1.12**

## 详细说明

在 `join.go` 中定义了简单接口 `Join(f0, f1 string, offset0, offset1 []int) (sum uint64)`。我们的测试框架会每次向该接口传入两个关系及其两列的偏移量数组，并验证输出结果的正确性。该接口的四个输入参数和一个输出参数说明如下：

- **f0**：给定关系0的文件名。
- **f1**：给定关系1的文件名。
- **offset0**：关系0中用于连接的列的偏移量数组。
- **offset1**：关系1中用于连接的列的偏移量数组。
- **sum**（输出参数）：最终连接结果中 relation0.col0 的总和。

（等值）连接条件由 `offset0/1` 指定，其形式如下：
```go
relation0.cols[offset[0]] = relation1.cols[offset[0]] and relation0.cols[offset[1]] = relation1.cols[offset[1]]...
```

**示例**：`Join("/path/T0", "/path/T1", []int{0, 1}, []int{2, 3})`

等价于 SQL：

```sql
SELECT SUM(T0.COL0)
FROM T0, T1
ON T0.COL0=T1.COL2 AND T0.COL1=T1.COL3
```

我们提供了一个示例实现 `join_example.go: JoinExample`，它采用简单的哈希连接算法：使用关系0构建哈希表，然后对关系1中的每一行进行探测。

## 要求与评分标准

- （30%）通过所有测试用例。
- （20%）性能优于 `join_example.go:JoinExample`。
- （35%）提供文档，描述你的设计思路，并使用 `pprof` 记录性能优化过程。
- （15%）保持良好的代码风格。

**注意**：
1. 对于你的校验和，只要使用 64 位无符号整数，无需担心数值溢出。
2. 更多大型数据集请参见 [此处](https://drive.google.com/drive/u/1/folders/10-iJNGKmKXgMmvBYnKt88RTwC0iA1XM-)，可用于辅助分析程序性能。
3. 我们将使用 `benchmark_test.go` 中的 `BenchmarkJoin` 和 `BenchmarkJoinExample` 来评估你的程序。测试数据**不会超出我们提供的范围**。

## 使用方法

1. 请在 `join.go` 中实现你自己的 `Join` 函数以完成本任务。
2. 我们在目录 `t` 中提供了三个关系的 CSV 格式文件（.tbl 文件），你可以将它们加载到数据库管理系统中进行测试。
   1. **r0.tbl**：2 列 × 10,000 条记录
   2. **r1.tbl**：4 列 × 5,000 条记录
   3. **r2.tbl**：4 列 × 500 条记录
3. `join_test.go` 中已内置单元测试 `JoinTest`。你可以编写自己的单元测试，但请确保在提交前 `JoinTest` 能够通过。
4. 使用 `make test` 运行所有单元测试。
5. 使用 `make bench` 运行所有基准测试。