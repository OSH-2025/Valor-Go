// Rust 版本的 hf3fs_fuse 应用入口
use std::sync::Mutex;

// 假设这些类型和函数已在其他文件实现
use crate::fuse::{FuseConfig, FuseClients, get_fuse_clients_instance, fuse_main_loop};

fn main() {
    // 1. 解析命令行参数（stub）
    let args: Vec<String> = std::env::args().collect();
    println!("[hf3fs_fuse] 启动参数: {:?}", args);

    // 2. 初始化配置（stub）
    let mut hf3fs_config = FuseConfig::default();
    // 这里可根据 args 解析配置
    println!("[hf3fs_fuse] 配置已初始化");

    // 3. 初始化全局资源（IBManager、日志、监控等，stub）
    println!("[hf3fs_fuse] IBManager/日志/监控初始化完成");

    // 4. 初始化 AppInfo（stub）
    let app_info = "app_info_stub";
    println!("[hf3fs_fuse] AppInfo: {}", app_info);

    // 5. 初始化 FUSE 客户端
    {
        let mut d = get_fuse_clients_instance();
        // 这里只是模拟，实际应传递 app_info、mountpoint、token_file、config
        if let Err(e) = d.init(&app_info, "/mnt", "/tmp/token", &hf3fs_config) {
            eprintln!("[hf3fs_fuse] Init fuse clients failed: {}", e);
            return;
        }
        println!("[hf3fs_fuse] FuseClients 初始化完成");
    }

    // 6. 进入主循环
    let ret = fuse_main_loop(
        &args.get(0).cloned().unwrap_or_else(|| "hf3fs_fuse".to_string()),
        true, // allow_other
        "/mnt", // mountpoint
        1024 * 1024, // maxbufsize
        "test_cluster", // cluster_id
    );
    std::process::exit(ret);
} 