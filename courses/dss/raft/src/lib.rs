#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[allow(unused_imports)]
#[macro_use]
extern crate prost_derive;

pub mod kvraft;
mod proto;
pub mod raft;

/// 用于抑制未使用变量警告的占位符函数。
fn your_code_here<T>(_: T) -> ! {
    unimplemented!()
}