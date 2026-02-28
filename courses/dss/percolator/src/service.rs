```rust
use crate::msg::{
    CommitRequest, CommitResponse, GetRequest, GetResponse, PrewriteRequest, PrewriteResponse,
    TimestampRequest, TimestampResponse,
};

// 定义时间戳服务
labrpc::service! {
    service timestamp {
        rpc get_timestamp(TimestampRequest) returns (TimestampResponse);
    }
}

// 导出时间戳服务相关结构体和函数
pub use timestamp::{add_service as add_tso_service, Client as TSOClient, Service};

// 定义事务服务
labrpc::service! {
    service transaction {
        rpc get(GetRequest) returns (GetResponse);
        rpc prewrite(PrewriteRequest) returns (PrewriteResponse);
        rpc commit(CommitRequest) returns (CommitResponse);
    }
}

// 导出事务服务相关结构体和函数
pub use transaction::{add_service as add_transaction_service, Client as TransactionClient};
```