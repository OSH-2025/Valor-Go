// Rust 版本的 FuseOps，补全 FUSE 回调主流程（移除 fuser 依赖）
use std::sync::Mutex;
use once_cell::sync::Lazy;

// 假设 FuseClients 已在其他文件定义
use crate::fuse::FuseClients;

// FUSE 操作表实现（简化版本，不依赖 fuser）
pub struct Hf3fsFuseOps;

impl Hf3fsFuseOps {
    pub fn new() -> Self {
        Self
    }
    
    pub fn lookup(&self, _parent: u64, _name: &str) -> i32 {
        // 示例：返回未找到
        -2 // ENOENT
    }

    pub fn getattr(&self, _ino: u64) -> i32 {
        // 示例：返回成功
        0
    }

    pub fn read(&self, _ino: u64, _offset: i64, _size: u32) -> i32 {
        // 示例：返回成功
        0
    }

    pub fn write(&self, _ino: u64, _offset: i64, _data: &[u8]) -> i32 {
        // 示例：返回写入的字节数
        _data.len() as i32
    }

    pub fn readdir(&self, _ino: u64, _offset: i64) -> i32 {
        // 示例：返回成功
        0
    }
}

// 全局 FuseClients 实例
static FUSE_CLIENTS_INSTANCE: Lazy<Mutex<FuseClients>> = Lazy::new(|| Mutex::new(FuseClients::new()));
// 全局 FUSE 操作表实例
static FUSE_OPS_INSTANCE: Lazy<Hf3fsFuseOps> = Lazy::new(|| Hf3fsFuseOps::new());

pub fn get_fuse_clients_instance() -> std::sync::MutexGuard<'static, FuseClients> {
    FUSE_CLIENTS_INSTANCE.lock().unwrap()
}

pub fn get_fuse_ops() -> &'static Hf3fsFuseOps {
    &FUSE_OPS_INSTANCE
}

// 单元测试略 