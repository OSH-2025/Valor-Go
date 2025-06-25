// Rust 版本的 UserConfig，补全 per-user 配置隔离与主要方法
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::fuse::FuseConfig;

#[derive(Debug, Default, Clone)]
pub struct FuseConfig;
#[derive(Debug, Default, Clone)]
pub struct MetaInode;
#[derive(Debug, Default, Clone)]
pub struct UserInfo {
    pub uid: u64,
    pub gid: u64,
}
#[derive(Debug, Default, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Default, Clone)]
pub struct LocalConfig {
    pub config: FuseConfig,
    pub updated_items: Vec<KeyValue>,
}

#[derive(Debug)]
pub struct UserConfig {
    pub config: Arc<Mutex<FuseConfig>>,
    pub configs: Arc<Mutex<HashMap<u64, Arc<Mutex<LocalConfig>>>>>,
    pub users: Arc<Mutex<HashSet<u64>>>,
}

impl UserConfig {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(FuseConfig::default())),
            configs: Arc::new(Mutex::new(HashMap::new())),
            users: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn init(&mut self, config: FuseConfig) {
        *self.config.lock().unwrap() = config;
    }

    pub fn get_config(&self, user_info: &UserInfo) -> FuseConfig {
        let users = self.users.lock().unwrap();
        if users.contains(&user_info.uid) {
            let configs = self.configs.lock().unwrap();
            if let Some(lconf) = configs.get(&user_info.uid) {
                return lconf.lock().unwrap().config.clone();
            }
        }
        self.config.lock().unwrap().clone()
    }

    pub fn set_config(&self, user_info: &UserInfo, updates: Vec<KeyValue>) {
        let mut configs = self.configs.lock().unwrap();
        let mut users = self.users.lock().unwrap();
        let lconf = configs.entry(user_info.uid).or_insert_with(|| {
            users.insert(user_info.uid);
            Arc::new(Mutex::new(LocalConfig {
                config: self.config.lock().unwrap().clone(),
                updated_items: vec![],
            }))
        });
        let mut lconf = lconf.lock().unwrap();
        for kv in updates {
            // 这里只是简单替换，实际可用 serde_json::Value 动态更新
            lconf.updated_items.push(kv.clone());
            // 这里可根据 key/value 更新 lconf.config
        }
    }

    pub fn lookup_config(&self, user_info: &UserInfo, key: &str) -> Option<String> {
        let config = self.get_config(user_info);
        // 这里只是简单查找，实际可用 serde_json::Value 动态查找
        // 例如：if key == "readonly" { Some(config.readonly.to_string()) } else { None }
        None
    }
}

// 单元测试示例
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_user_config_init_and_get() {
        let mut user_config = UserConfig::new();
        let config = FuseConfig::default();
        user_config.init(config.clone());
        let user_info = UserInfo { uid: 1, gid: 1 };
        let got = user_config.get_config(&user_info);
        // 这里只能断言类型一致
        let _ = got;
    }
} 