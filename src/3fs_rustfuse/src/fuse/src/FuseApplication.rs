use crate::FuseAppConfig::KeyValue;


use std::collections::HashMap;

// 模拟配置对象
pub trait ConfigBase {
    fn init(&mut self, file_path: &str, dump: bool, updates: Vec<KeyValue>);
}

// FuseAppConfig 结构体（已在 fuse_app_config.rs 中定义）
pub struct FuseAppConfig {
    pub node_id: u64,
}

impl FuseAppConfig {
    pub fn new() -> Self {
        FuseAppConfig { node_id: 0 }
    }

    pub fn init(&mut self, file_path: &str, dump: bool, updates: Vec<KeyValue>) {
        // 模拟配置初始化
        if dump {
            println!("Dumping default config...");
        }

        for kv in &updates {
            println!("Update: {} = {}", kv.key, kv.value);
        }

        self.node_id = 123;
    }
}

// AppInfo 结构体，用于返回应用信息
#[derive(Debug)]
pub struct AppInfo {
    pub node_id: u64,
    pub hostname: String,
}

// ApplicationBase trait：模拟 C++ 的抽象类接口
pub trait ApplicationBase {
    fn parse_flags(&mut self, argc: i32, argv: Vec<String>) -> Result<(), String>;
    fn init_application(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn main_loop(&self) -> i32;
    fn get_config(&self) -> &dyn ConfigBase;
    fn info(&self) -> Option<&AppInfo>;
    fn config_pushable(&self) -> bool;
    fn on_config_updated(&self);
}

// FuseApplication 实现 ApplicationBase 接口
pub struct FuseApplication {
    impl_: FuseApplicationImpl,
}

struct FuseApplicationImpl {
    hf3fs_config: FuseAppConfig,
    app_info: AppInfo,
    config_flags: HashMap<String, String>,
    program_name: String,
    allow_other: bool,
    mountpoint: String,
    max_buf_size: usize,
    cluster_id: String,
}

impl FuseApplication {
    pub fn new() -> Self {
        Self {
            impl_: FuseApplicationImpl {
                hf3fs_config: FuseAppConfig::new(),
                app_info: AppInfo {
                    node_id: 0,
                    hostname: "localhost".to_string(),
                },
                config_flags: HashMap::new(),
                program_name: "rust_fuse".to_string(),
                allow_other: false,
                mountpoint: "/mnt/hf3fs".to_string(),
                max_buf_size: 1024 * 1024,
                cluster_id: "default".to_string(),
            },
        }
    }
}

impl ApplicationBase for FuseApplication {
    fn parse_flags(&mut self, argc: i32, argv: Vec<String>) -> Result<(), String> {
        // 简单模拟命令行参数解析
        for arg in &argv {
            if arg == "--allow-other" {
                self.impl_.allow_other = true;
            } else if arg.starts_with("--mountpoint=") {
                let parts: Vec<&str> = arg.split('=').collect();
                if parts.len() == 2 {
                    self.impl_.mountpoint = parts[1].to_string();
                }
            } else if arg.starts_with("--cluster-id=") {
                let parts: Vec<&str> = arg.split('=').collect();
                if parts.len() == 2 {
                    self.impl_.cluster_id = parts[1].to_string();
                }
            }
        }

        // 模拟加载配置标志
        self.impl_.config_flags.insert(
            "test_key".to_string(),
            "test_value".to_string(),
        );

        Ok(())
    }

    fn init_application(&mut self) -> Result<(), String> {
        // 模拟打印默认配置
        if self.config_pushable() {
            println!("Dumping default config...");
            println!("{}", self.impl_.hf3fs_config_to_string());
        }

        // 初始化组件
        println!("Init common components...");
        self.impl_.app_info.node_id = 123;

        // 模拟配置初始化
        self.impl_.hf3fs_config.init(&self.impl_.mountpoint, true, vec![]);

        // 模拟持久化配置
        self.on_config_updated();

        Ok(())
    }

    fn stop(&mut self) {
        // 停止 fuse 客户端
        println!("Stopping fuse clients...");
    }

    fn main_loop(&self) -> i32 {
        // 主循环逻辑
        println!("Running fuse main loop...");
        std::thread::sleep(std::time::Duration::from_secs(5));
        println!("Main loop exited.");
        0
    }

    fn get_config(&self) -> &dyn ConfigBase {
        &self.impl_.hf3fs_config
    }

    fn info(&self) -> Option<&AppInfo> {
        Some(&self.impl_.app_info)
    }

    fn config_pushable(&self) -> bool {
        // 模拟是否可以推送配置
        true
    }

    fn on_config_updated(&self) {
        // 配置更新回调
        println!("Persisting config...");
    }
}

impl FuseApplicationImpl {
    fn hf3fs_config_to_string(&self) -> String {
        format!(
            "Mountpoint: {}\nMaxBufSize: {}\nClusterID: {}",
            self.mountpoint, self.max_buf_size, self.cluster_id
        )
    }
}

// 给FuseAppConfig实现ConfigBase trait
impl ConfigBase for FuseAppConfig {
    fn init(&mut self, file_path: &str, dump: bool, updates: Vec<KeyValue>) {
        self.init(file_path, dump, updates)
    }
}
