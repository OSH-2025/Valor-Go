#[cfg(feature = "enable_fuse_application")]
mod fuse_application {
    use std::env;
    use std::process;

    pub fn main() {
        // 假设 FuseApplication 是一个外部定义的模块
        let args: Vec<String> = env::args().collect();
        let exit_code = fuse_application::FuseApplication::run(&args);
        process::exit(exit_code);
    }
}

#[cfg(not(feature = "enable_fuse_application"))]
mod main_loop {
    use std::env;
    use std::process;
    use std::thread;
    use std::time::{Duration, Instant};
    use log::{info, fatal, critical};

    // 假设这些模块和类型是外部定义的
    mod fuse_config {
        pub struct FuseConfig {
            // 配置字段
        }

        impl FuseConfig {
            pub fn init(&self, _argc: &mut i32, _argv: &mut Vec<String>) {
                // 初始化逻辑
            }

            // 其他方法
        }
    }

    mod logging {
        pub fn generate_log_config(_config: &str, _name: &str) -> String {
            // 生成日志配置
            String::new()
        }

        pub fn init_or_die(_config: &str) {
            // 初始化日志或退出
        }
    }

    mod sys_resource {
        pub fn hostname(physical_machine_name: bool) -> Option<String> {
            // 获取主机名
            Some(String::new())
        }

        pub fn pid() -> i32 {
            // 获取进程 ID
            0
        }
    }

    mod client_id {
        pub struct ClientId;

        impl ClientId {
            pub fn random(_hostname: &str) -> Self {
                ClientId
            }
        }
    }

    mod fuse_clients {
        pub struct FuseClients;

        impl FuseClients {
            pub fn init(&self, _app_info: &AppInfo, _mountpoint: &str, _token_file: &str, _config: &FuseConfig) -> Result<(), String> {
                // 初始化逻辑
                Ok(())
            }

            pub fn stop(&self) {
                // 停止逻辑
            }
        }

        pub fn get_fuse_clients_instance() -> FuseClients {
            FuseClients
        }
    }

    mod fuse_main_loop {
        pub fn run(_argv0: &str, _allow_other: bool, _mountpoint: &str, _max_buf_size: u32, _cluster_id: &str) -> i32 {
            // 主循环逻辑
            0
        }
    }

    pub struct AppInfo {
        pub cluster_id: String,
        pub hostname: String,
        pub pid: i32,
        pub release_version: String,
    }

    pub fn main() {
        let mut args: Vec<String> = env::args().collect();
        let mut argc = args.len() as i32;

        let mut config = fuse_config::FuseConfig {};
        config.init(&mut argc, &mut args);

        let ib_result = net::IBManager::start(&config.ib_devices());
        if let Err(e) = ib_result {
            fatal!("Failed to start IBManager: {}", e);
            process::exit(1);
        }
        defer!(net::IBManager::stop());

        let log_config_str = logging::generate_log_config(&config.log(), "hf3fs_fuse");
        info!("LogConfig: {}", log_config_str);
        logging::init_or_die(&log_config_str);
        info!("{}", version_info::full());

        let physical_hostname_res = sys_resource::hostname(true);
        if let None = physical_hostname_res {
            fatal!("Get physical hostname failed");
            process::exit(1);
        }
        let physical_hostname = physical_hostname_res.unwrap();

        let container_hostname_res = sys_resource::hostname(false);
        if let None = container_hostname_res {
            fatal!("Get container hostname failed");
            process::exit(1);
        }
        let container_hostname = container_hostname_res.unwrap();

        let client_id = client_id::ClientId::random(&physical_hostname);

        let app_info = AppInfo {
            cluster_id: config.cluster_id().to_string(),
            hostname: physical_hostname,
            pid: sys_resource::pid(),
            release_version: flat::ReleaseVersion::from_version_info().to_string(),
        };

        let fuse_clients = fuse_clients::get_fuse_clients_instance();
        if let Err(e) = fuse_clients.init(&app_info, &config.mountpoint(), &config.token_file(), &config) {
            fatal!("Init fuse clients failed: {}", e);
            process::exit(1);
        }
        defer!(fuse_clients.stop());

        fuse_main_loop::run(&args[0], config.allow_other(), &config.mountpoint(), config.io_bufs().max_buf_size(), &config.cluster_id());
    }
}

fn main() {
    #[cfg(feature = "enable_fuse_application")]
    fuse_application::main();

    #[cfg(not(feature = "enable_fuse_application"))]
    main_loop::main();
}