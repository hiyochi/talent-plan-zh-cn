#[allow(unused_imports)]
#[macro_use]
extern crate log;

// 完成实现后，应移除 `#[allow(unused)]`。
#[allow(dead_code, unused)]
mod client;
#[allow(unused)]
mod server;
mod service;
#[cfg(test)]
mod tests;

// 这与 `msg.proto` 中描述的 protobuf 相关。
mod msg {
    include!(concat!(env!("OUT_DIR"), "/msg.rs"));
}