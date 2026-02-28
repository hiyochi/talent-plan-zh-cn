## 简介

这是 PingCAP 人才计划在线课程第二周的 Map-Reduce 作业。

你将面对一个未完成的 Map-Reduce 框架，你需要补全它，并使用该框架从数据文件中提取出出现频率最高的 10 个 URL。

## 了解源代码

简单的 Map-Reduce 框架定义在 `mapreduce.go` 文件中。

该框架尚未完成，你需要在注释 `YOUR CODE HERE` 下方填写你的代码。

Map 和 Reduce 函数的定义与 MIT 6.824 实验一相同：
```
type ReduceF func(key string, values []string) string
type MapF func(filename string, contents string) []KeyValue
```

在 `urltop10_example.go` 中提供了一个示例，用于提取出现频率最高的 10 个 URL。

完成框架后，你可以通过 `make test_example` 运行该示例。

然后，请在 `urltop10.go` 中实现你自己的 `MapF` 和 `ReduceF` 函数以完成此任务。

填写完代码后，请使用 `make test_homework` 进行测试。

所有数据文件将在运行时自动生成，你可以使用 `make cleanup` 清理所有测试数据。

请按字典序输出 URL，并确保你的结果格式与测试数据一致，以便通过所有测试。

每个测试用例具有**不同的数据分布**，你需考虑这一点。

## 要求与评分标准

* （40%）性能优于 `urltop10_example`。
* （20%）通过所有测试用例。
* （30%）提供一份文档，描述你的设计思路，并使用 `pprof` 记录性能优化过程（包括框架和你自己的代码）。
* （10%）代码风格良好。

注意：**必须使用 go 1.12 或更高版本**

## 使用方法

在 `mapreduce.go` 文件中，于注释 `YOUR CODE HERE` 下方填写代码，以完成该框架。

在 `urltop10.go` 中实现你自己的 `MapF` 和 `ReduceF` 函数，并使用 `make test_homework` 进行测试。

`urltop10_test.go` 中已内置单元测试，但你仍可自行编写单元测试。

运行示例的方法：
```
make test_example
```

测试你的实现的方法：
```
make test_homework
```

清理所有测试数据的方法：
```
make cleanup
```

重新生成测试数据的方法：
```
make gendata
```