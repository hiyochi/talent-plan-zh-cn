fn main() {
    // 定义包含 proto 文件的目录路径
    let includes = &[std::path::PathBuf::from("src/proto")];
    // 创建一个向量用于存储找到的 proto 文件路径
    let mut protos = Vec::new();
    // 遍历每个包含目录
    for include in includes {
        // 读取目录中的所有条目
        for file in std::fs::read_dir(include).unwrap() {
            let file = file.unwrap();
            // 如果是目录则跳过
            if file.file_type().unwrap().is_dir() {
                continue;
            }
            // 获取文件路径
            let path = file.path();
            // 检查文件扩展名是否为 proto
            if path.extension().unwrap() == "proto" {
                // 将 proto 文件路径添加到向量中
                protos.push(path);
            }
        }
    }
    // 使用 prost_build 编译找到的 proto 文件
    prost_build::compile_protos(&protos, includes).unwrap();
    // 为每个 proto 文件添加构建脚本重新运行的条件
    for p in protos {
        println!("cargo:rerun-if-changed={}", p.display());
    }
}