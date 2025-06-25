// src/fuse/FuseAppConfig.rs

use std::string::String;
use std::vec::Vec;

#[derive(Debug, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}
impl KeyValue {
    pub fn new(key: String, value: String) -> Self {
        KeyValue { key, value }
    }
}

pub trait ConfigBase {
    fn init(&mut self, file_path: &str, dump: bool, updates: Vec<KeyValue>) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct FuseAppConfig {
    pub token: String,
    pub mountpoint: String,
    node_id: u64,
}

impl FuseAppConfig {
    pub fn new() -> Self {
        FuseAppConfig { node_id: 0, token: String::new(), mountpoint: String::new() }
    }

    pub fn init(&mut self, file_path: &str, dump: bool, updates: Vec<KeyValue>) {
        let res = ApplicationBase::init_config(self, file_path, dump, updates);
        if let Err(e) = res {
            panic!("Init app config failed: {:?}. filePath: {}. dump: {}", e, file_path, dump);
        }
    }

    pub fn get_node_id(&self) -> u64 {
        self.node_id
    }
}

pub struct ApplicationBase;

impl ApplicationBase {
    pub fn init_config(
        config: &mut FuseAppConfig,
        file_path: &str,
        dump: bool,
        updates: Vec<KeyValue>,
    ) -> Result<(), String> {
        // TODO: 实际配置加载逻辑
        Ok(())
    }
}