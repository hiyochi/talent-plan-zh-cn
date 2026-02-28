fn main() {
    // 编译 proto/fixture.proto 文件，指定 proto 目录为搜索路径
    prost_build::compile_protos(&["proto/fixture.proto"], &["proto"]).unwrap();
    // 当 proto 目录中的文件发生变化时，重新运行构建脚本
    println!("cargo:rerun-if-changed=proto");
}