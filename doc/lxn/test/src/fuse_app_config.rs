// src/fuse/FuseAppConfig.rs
/*
use std::string::String;
use std::vec::Vec;

// KeyValue 结构定义
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

impl KeyValue {
    pub fn new(key: String, value: String) -> Self {
        KeyValue { key, value }
    }
}

// ConfigBase trait 定义
pub trait ConfigBase {
    fn init(&mut self, file_path: &str, dump: bool, updates: Vec<KeyValue>) -> Result<(), String>;
}

// FuseAppConfig 结构定义
pub struct FuseAppConfig {
    // ConfigBase 的字段
    node_id: u64,
}

impl FuseAppConfig {
    pub fn new() -> Self {
        FuseAppConfig {
            node_id: 0,
        }
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

// 实现 ConfigBase trait
impl ConfigBase for FuseAppConfig {
    fn init(&mut self, file_path: &str, dump: bool, updates: Vec<KeyValue>) -> Result<(), String> {
        let res = ApplicationBase::init_config(self, file_path, dump, updates);
        if let Err(e) = res {
            return Err(format!("Init app config failed: {:?}. filePath: {}. dump: {}", e, file_path, dump));
        }
        Ok(())
    }
}

// ApplicationBase 结构定义
pub struct ApplicationBase;

impl ApplicationBase {
    pub fn init_config(
        config: &mut FuseAppConfig,
        file_path: &str,
        dump: bool,
        updates: Vec<KeyValue>
    ) -> Result<(), String> {
        // 这里实现配置初始化的具体逻辑
        // 目前返回 Ok(()) 作为占位
        Ok(())
    }
}
*/
use std::string::String;
use std::vec::Vec;
/*
#[derive(Debug)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}*/
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

pub struct FuseAppConfig {
    node_id: u64,
}

impl FuseAppConfig {
    pub fn new() -> Self {
        FuseAppConfig { node_id: 0 }
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