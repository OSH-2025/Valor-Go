// Rust 版本的 FuseLauncherConfig

#[derive(Debug, Clone, Default)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct IBDeviceConfig;
#[derive(Debug, Clone, Default)]
pub struct ClientConfig;
#[derive(Debug, Clone, Default)]
pub struct MgmtdClientConfig;

#[derive(Debug, Clone)]
pub struct FuseLauncherConfig {
    pub cluster_id: String,
    pub ib_devices: IBDeviceConfig,
    pub client: ClientConfig,
    pub mgmtd_client: MgmtdClientConfig,
    pub mountpoint: String,
    pub allow_other: bool,
    pub token_file: String,
}

impl Default for FuseLauncherConfig {
    fn default() -> Self {
        Self {
            cluster_id: String::new(),
            ib_devices: IBDeviceConfig::default(),
            client: ClientConfig::default(),
            mgmtd_client: MgmtdClientConfig::default(),
            mountpoint: String::new(),
            allow_other: true,
            token_file: String::new(),
        }
    }
}

impl FuseLauncherConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, file_path: &str, dump: bool, updates: Vec<KeyValue>) {
        // stub: 实际应从文件加载配置并应用 updates
        // 这里只做简单模拟
        if dump {
            println!("[Dump] Would load config from: {}", file_path);
        }
        for kv in updates {
            println!("Update config: {} = {}", kv.key, kv.value);
        }
    }
}

// 单元测试示例
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_launcher_config_init() {
        let mut config = FuseLauncherConfig::new();
        let updates = vec![KeyValue { key: "mountpoint".to_string(), value: "/mnt".to_string() }];
        config.init("/tmp/launcher.toml", true, updates);
        assert_eq!(config.allow_other, true);
    }
} 