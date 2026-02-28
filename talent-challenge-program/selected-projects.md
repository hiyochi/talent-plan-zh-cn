## 第一季精选项目

以下列出了第一季的精选项目。希望参与某个项目的学员请参考 [学员选拔流程](README.md#mentees)。

项目难度分为三个等级：非常困难、困难和中等，对应的税前奖金分别为：非常困难项目 10,000 元人民币，困难项目 8,000 元人民币，中等项目 5,000 元人民币。

### 模板

```
#### TiDB 生态系统项目名称
##### 标题
- 描述：
- 推荐技能：
- 导师：
- 上游 Issue 或 RFC（链接）：
- 难度：
```

## 精选项目列表

#### BR

##### BR 导出功能

- 描述：在 BR 备份功能基础上，实现“导出”功能，生成 CSV 和 SQL 转储文件。
- 推荐技能：Rust、Go、行编码（MVCC 和 TiDB）、gRPC。
- 导师：@kennytm
- 上游 Issue 或 RFC（链接）：https://github.com/pingcap/br/issues/351
- 难度：困难

#### TiCDC

##### TiCDC 云存储支持

* 描述：某些使用场景需要将变更事件发送到低成本存储介质（如 S3），以便长期保存数据并异步消费历史变更事件。我们需要为 TiCDC 开放协议设计并实现云存储策略，并基于云存储方案提供相应的消费策略。
* 推荐技能：Go、云存储服务。
* 导师：@amyangfei
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/ticdc/issues/655
* 难度：中等

##### TiCDC 快照级一致性复制

* 描述：在许多场景中，用户希望确保下游能复制到全局一致的状态。由于 TiCDC 支持最终事务一致性，而 TiDB 支持快照读取，我们可以结合这两个特性，提供一种快照级一致性复制策略。
* 推荐技能：Go、事务。
* 导师：@amyangfei
* 学员：[@Colins110](https://github.com/Colins110)
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/ticdc/issues/658
* 难度：中等

##### TiCDC 支持 Avro Sink 和 Kafka 连接器

* 描述：Apache Kafka 提供了灵活的连接器机制，广泛应用于变更数据捕获场景。我们希望实现一个 Avro Sink，使 TiCDC 能够兼容 Kafka 连接器生态系统。
* 推荐技能：Go、Kafka。
* 导师：@amyangfei, @liuzx
* 学员：[@qinggniq](https://github.com/qinggniq)
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/ticdc/issues/660
* 难度：中等

#### TiDB

##### 实时执行计划

* 描述：提供一种查询正在运行的 SQL 当前执行细节的方式。此功能类似于 SQL Server 中的“实时查询统计”。SQL Server 允许查看任何查询的实时执行计划。
* 推荐技能：Go、SQL
* 导师：@crazycs520 @breeswish @qw4990
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/tidb/issues/17692
* 难度：中等

##### 在 SQL 中定义放置规则

* 描述：TiDB 支持放置规则，但目前只能通过配置文件定义。如果能通过 SQL 语句配置放置规则，将显著提升可用性。
* 推荐技能：Go、数据定义语言（DDL）
* 导师：@djshow832
* 学员：[@xhe](https://github.com/xhebox)
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/tidb/issues/18030
* 难度：困难

##### 在 TiDB 中支持 SAVEPOINT

* 描述：SAVEPOINT 是主流传统数据库（如 Oracle、DB2、MySQL 和 PostgreSQL）普遍支持的功能，其作用是在事务执行过程中部分回滚多个语句。TiDB 目前尚不支持此功能。
* 推荐技能：Go
* 导师：@bobotu
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/tidb/issues/6840
* 难度：非常困难

##### 支持分区表的全局索引

* 描述：目前 TiDB 仅支持与 MySQL 兼容的本地分区索引，但其他数据库（如 Oracle）也支持全局分区索引。
* 推荐技能：Go、DDL、SQL
* 导师：@tiancaiamao
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/tidb/issues/18032
* 难度：困难

##### 降低统计信息数据的内存消耗

* 描述：当 TiDB 集群中存在大量表时，将所有统计信息缓存在单个 TiDB 服务器中可能导致服务器启动时内存占用过高，增加 OOM 风险。
* 推荐技能：Go
* 导师：@SunRunAway
* 学员：@[miamiaoxyz](https://github.com/miamia0)
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/tidb/issues/16572
* 难度：中等

##### 支持 utf8_unicode_ci / utf8mb4_unicode_ci 排序规则

* 描述：实现 utf8_unicode_ci / utf8mb4_unicode_ci 排序规则算法并进行优化。
* 推荐技能：Go、C++、Rust
* 导师：@wjhuang2016
* 学员：@[xiongjiwei](https://github.com/xiongjiwei)
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/tidb/issues/17596
* 难度：中等

#### TiKV

##### SQL 语句级别统计

* 描述：为了识别导致热点区域的根源语句，我们需要在 TiKV 中添加本地统计功能，统计前 K 个热点语句，并将结果上报给 PD。
* 推荐技能：Rust、Go、算法 / 数据结构
* 导师：@breeswish @HundunDM
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap-incubator/tidb-dashboard/issues/574
* 难度：困难

##### Zetta 上的 HBase 协议支持

* 描述：Zetta 是构建在 TiKV 之上的 BigTable 实现，用于支持结构化的无模式数据模型。为更好地融入现有 HBase 生态系统，我们计划为 Zetta 实现 HBase 协议适配器。为此，我们需要：
	1. 实现一个模拟的 ZooKeeper，以消除外部依赖；
	2. 实现 HBase RegionServer 的 RPC 协议；
	3. 在 Zetta 之上实现 HBase 功能。
* 推荐技能：Go、Java、HBase RPC
* 导师：pseudocodes, baiyuqing
* 学员：[@BowenXiao1999](https://github.com/BowenXiao1999)
* 上游 Issue 或 RFC（链接）：https://github.com/zhihu/zetta/issues/2（中文）
* 难度：困难

#### Chaos Mesh

##### 支持通过插件扩展调度器

* 描述：在许多场景中，用户需要自定义选择器。例如，当我们的 TiDB 集群包含多个 PD 时，我们只想向 Leader 注入故障。因此，我们需要提供一个插件机制，允许用户通过代码自定义选择器。例如，我们可以创建一个名为 `pd-selector.go` 的文件，并将其设置在选择器字段中。
* 推荐技能：Go、Kubernetes
* 导师：@cwen0
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/chaos-mesh/issues/193
* 难度：中等

##### 支持注入 HTTP 故障

* 描述：在许多场景中，用户需要对特定应用程序注入 HTTP 延迟故障或 HTTP 中断故障。例如，我们只想对 `/api/xxx` 路由注入延迟故障。
* 推荐技能：Go、Kubernetes
* 导师：@Yisaer
* 上游 Issue 或 RFC（链接）：https://github.com/pingcap/chaos-mesh/issues/651
* 难度：困难