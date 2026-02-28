## 项目创意

项目维护者和导师，请使用以下模板提交下方的创意。入选第一季的项目将列在 [入选项目](selected-projects.md) 页面中。

### 模板

```
#### TiDB 生态系统项目名称
##### 标题
- 描述：
- 推荐技能：
- 导师：
- 上游问题或 RFC（链接）：
```

### 提出的项目创意

#### BR

##### BR HTTP 存储
- 描述：支持将 HTTP(S) 服务器作为 BR 的数据源和目标，并允许 BR 自身作为经过身份验证的 HTTP(S) 服务器运行，以简化部署。
- 推荐技能：Go、Rust、HTTP 通信、TLS 处理。
- 导师：kennytm
- 上游问题或 RFC（链接）：
   - https://github.com/pingcap/br/issues/308,
   - https://github.com/pingcap/br/issues/212 

##### BR 导出
- 描述：在 BR 备份功能基础上，实现“导出”功能，生成 CSV 和 SQL 转储文件。
- 推荐技能：Rust、Go、行编码（MVCC 和 TiDB）、gRPC。
- 导师：kennytm
- 上游问题或 RFC（链接）：https://github.com/pingcap/br/issues/351 

#### TiUP Bench

##### 生成 BR 备份归档文件
- 描述：目前通过 tiup bench 准备的 TPC-C/TPC-H 数据可通过 SQL 插入或导出为 CSV 文件进行批量导入。这两种方法都比 BR 恢复慢得多。本项目希望直接生成 BR 备份归档文件，以便不关心准备步骤的基准测试能快速启动。
- 推荐技能：Go、行编码（TiDB）、TiKV/RocksDB SST 格式
- 导师：kennytm
- 上游问题或 RFC（链接）：https://github.com/pingcap/go-tpc/issues/46 

#### TiCDC

##### TiCDC 云存储
- 描述：某些使用场景需要将变更事件发送到低成本存储介质（如 S3），以便长期保存数据并异步消费历史变更事件。我们需要为 TiCDC 开放协议设计并实现云存储策略，并基于云存储架构提供消费策略。
- 推荐技能：Go、云存储服务。
- 导师：yangfei
- 上游问题或 RFC（链接）：https://github.com/pingcap/ticdc/issues/655

##### TiCDC 快照级一致性复制
- 描述：在许多场景中，用户希望确保下游复制到全局一致的状态。虽然 TiCDC 支持最终事务一致性，而 TiDB 支持快照读取，我们可以结合这两个特性，提供一种快照级一致性复制策略。
- 推荐技能：Go、事务。
- 导师：yangfei
- 上游问题或 RFC（链接）：https://github.com/pingcap/ticdc/issues/658

##### TiCDC 新的 ResolvedTS 机制
- 描述：TiCDC 需要一个时间戳（称为 ResolvedTS），以确保所有在此时间戳之前开始的事务均已完成并从 TiKV 发送到 TiCDC。目前，TiKV 必须在 Raft 组的领导者节点上推进 ResolvedTS。我们需要一种新机制来消除这一限制。
- 推荐技能：Go、Rust、Raft
- 导师：tangminghua
- 上游问题或 RFC（链接）：https://github.com/pingcap/ticdc/issues/657

##### TiCDC 支持 Avro Sink 和 Kafka Connector
- 描述：Apache Kafka 提供了灵活的连接器机制，广泛应用于变更数据捕获场景。我们希望实现一个 Avro Sink，并使 TiCDC 兼容 Kafka 连接器生态系统。
- 推荐技能：Go、Kafka。
- 导师：yangfei, liuzixiong
- 上游问题或 RFC（链接）：https://github.com/pingcap/ticdc/issues/660

##### TiCDC 为 Changefeed 提供状态机制
- 描述：在执行复制任务期间可能会遇到各种错误（例如下游连接失败、不兼容的 DDL 等）。我们希望提供一种机制，使用户能够快速了解当前复制任务的状态（正常或异常）以及错误原因。
- 推荐技能：Go、SQL。
- 导师：zhaoyilin
- 上游问题或 RFC（链接）：https://github.com/pingcap/ticdc/issues/664

#### PD

##### PD 存储与区域 UI
- 描述：为 PD 存储和区域添加显示与操作界面，以减少对 pd-ctl 的依赖，提升用户体验。
- 推荐技能：Go、Web 前端。
- 导师：@HundunDM @breeswish
- 上游问题或 RFC（链接）：https://docs.google.com/document/d/1moQVhvIgqu_FWuv_UMB76AM5tETJkyeWnzI04wyevhk/edit

##### PD 静态数据加密
- 描述：静态数据加密指数据在存储时即被加密。TiKV 已支持此功能，但 PD 尚未支持。PD 存储集群的元信息，特别是 Region 的 Key 信息，需要对其进行加密。本提案建议 PD 也支持加密功能。
- 推荐技能：Go、密码学
- 导师：@yiwu-arbug
- 上游问题或 RFC（链接）：
	* TiKV: https://github.com/tikv/rfcs/blob/929bf1f5d675b555c013d863599544afd9bfe812/text/2020-04-27-encryption-at-rest.md
	* PD: 进行中

#### TiDB

##### 实时执行计划
- 描述：提供一种查询正在运行 SQL 的实时执行详情的方式。此功能类似于 SQL Server 中的“实时查询统计”。SQL Server 允许查看任何查询的实时执行计划。
- 推荐技能：Go、SQL
- 导师：@crazycs520 @breeswish @qw4990
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/17692

##### 在日志中打印事务 ID 和查询 ID 以追踪事务/SQL 的完整生命周期
- 描述：目前 TiDB 在日志中输出的是 TxnStartTs 或 ConnId，信息不足。本任务旨在为每个事务分配并打印唯一的 TxnId，同时为每个查询分配并打印唯一的 QueryId，适用于 TiDB 和 TiKV。
- 推荐技能：Go、Rust
- 导师：@crazycs520 @breeswish @SunRunAway
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/17845

##### 在 SQL 中定义放置规则
- 描述：TiDB 支持放置规则，但目前只能在配置文件中定义。如果能通过 SQL 语句配置放置规则，可显著提升可用性。
- 推荐技能：Go、数据定义语言（DDL）
- 导师：@djshow832
- 上游问题或 RFC（链接）：
	* https://github.com/pingcap/tidb/issues/18030
	* https://docs.google.com/document/d/18Kdhi90dv33muF9k_VAIccNLeGf-DdQyUc8JlWF9Gok/edit#

##### 支持在单条语句中重命名多个表
- 描述：TiDB 目前支持重命名单个表，但不支持一次重命名多个表。本任务希望支持在一条语句中重命名多个表。
- 推荐技能：Go、DDL
- 导师：@zimulala
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/14766

##### 支持在单条语句中删除多个索引
- 描述：本任务希望支持在一条语句中删除多个索引。
- 推荐技能：Go、DDL
- 导师：@zimulala
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/14765

##### 删除包含索引的列
- 描述：支持删除包含索引的列的操作。
- 推荐技能：Go、DDL
- 导师：@zimulala
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/3364

##### 在 TiDB 中支持 SAVEPOINT
- 描述：SAVEPOINT 是主流传统数据库（如 Oracle、DB2、MySQL 和 PostgreSQL）普遍支持的功能，其作用是在执行事务时实现部分回滚。TiDB 目前尚不支持此功能。
- 推荐技能：Go
- 导师：@bobotu
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/6840

##### 支持 LIST COLUMNS 分区
- 描述：LIST COLUMNS 分区是 MySQL 8.0 普遍支持的功能，用于通过列表列定义表分区。TiDB 目前尚不支持此功能。
- 推荐技能：Go、DDL、SQL
- 导师：@imtbkcat
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/18052

##### 支持分区表的全局索引
- 描述：目前 TiDB 仅支持与 MySQL 兼容的本地分区索引，但其他数据库（如 Oracle）也支持全局分区索引。
- 推荐技能：Go、DDL、SQL
- 导师：@tiancaiamao
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/18032

##### 实现连接顺序提示（Join Order Hint）
- 描述：提供一种注释风格的 SQL 提示，用于指定查询中连接的顺序。
- 推荐技能：Go、SQL
- 导师：@eurekaka
- 上游问题或 RFC（链接）：进行中

##### 减少统计信息数据的内存消耗
- 描述：当 TiDB 集群中存在大量表时，在 TiDB 服务器启动时将所有统计信息缓存到单个 TiDB 服务器中可能导致内存消耗过高，增加 OOM 风险。
- 推荐技能：Go
- 导师：@SunRunAway
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/16572

##### 减少表元数据的内存消耗
- 描述：TiDB 在启动时会一次性加载所有 Schema 中的所有表，这会消耗大量内存，增加 TiDB 服务器的 OOM 风险。
- 推荐技能：Go
- 导师：@bb7133, @SunRunAway
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/16572

##### 支持 utf8_unicode_ci / utf8mb4_unicode_ci 排序规则
- 描述：实现 utf8_unicode_ci / utf8mb4_unicode_ci 排序规则算法并对其进行优化。
- 推荐技能：Go
- 导师：@wjhuang2016
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/17596

##### 实现更多诊断规则
- 描述：在 TiDB SQL 诊断中增加更多诊断规则。
- 推荐技能：Go、SQL
- 导师：@crazycs520
- 上游问题或 RFC（链接）：https://github.com/pingcap/tidb/issues/17927

#### TiKV

##### Witness（见证节点）
- 描述：支持 Raft 见证节点角色，并在双数据中心部署场景中使用，即使其中一个数据中心崩溃，TiDB 仍能继续提供服务。
- 推荐技能：Rust、Raft
- 导师：@busyjay
- 上游问题或 RFC（链接）：进行中

##### 松散的跟随者读取（Loose Follower Read）
- 描述：提供一种无需向领导者发送请求即可从跟随者读取数据的方式。
- 推荐技能：Rust、Raft
- 导师：@hicqu
- 上游问题或 RFC（链接）：进行中

##### 写入流量控制
- 描述：根据请求的延迟控制写入流量，使写入过程更加平稳。
- 推荐技能：Rust
- 导师：@Conner1996
- 上游问题或 RFC（链接）：https://docs.google.com/document/d/1rgm4rS2youwJpy_zpC39BJgxPpnwk7DeuF5LjvWrBZ8/edit#

##### SQL 语句级别统计
- 描述：为了识别导致热点 Region 的具体语句，我们需要在 TiKV 中添加本地统计功能，统计前 K 个热点语句，并将结果上报给 PD。
- 推荐技能：Rust、Go
- 导师：@breeswish @HundunDM
- 上游问题或 RFC（链接）：https://github.com/pingcap-incubator/tidb-dashboard/issues/574