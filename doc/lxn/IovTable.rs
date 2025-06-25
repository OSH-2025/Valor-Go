// Rust 版本的 IovTable，补全 IO 映射表管理接口
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Default, Clone)]
pub struct ShmBuf;
#[derive(Debug, Default, Clone)]
pub struct MetaInode;
#[derive(Debug, Default, Clone)]
pub struct UserInfo {
    pub uid: u64,
    pub gid: u64,
}
#[derive(Debug, Default, Clone)]
pub struct StorageClient;

#[derive(Debug, Default)]
pub struct IovTable {
    pub mount_name: String,
    pub shms_by_id: Mutex<HashMap<String, i32>>, // Uuid->int
    pub iovs: Mutex<HashMap<i32, Arc<ShmBuf>>>,
    pub iovds: RwLock<HashMap<String, i32>>,
}

impl IovTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, mount: &str, cap: usize) {
        self.mount_name = mount.to_string();
        self.iovs = Mutex::new(HashMap::with_capacity(cap));
    }

    pub fn add_iov(&self, key: &str, shm: Arc<ShmBuf>) -> Result<i32, String> {
        let mut iovs = self.iovs.lock().unwrap();
        let idx = iovs.len() as i32 + 1;
        iovs.insert(idx, shm);
        let mut iovds = self.iovds.write().unwrap();
        iovds.insert(key.to_string(), idx);
        Ok(idx)
    }

    pub fn rm_iov(&self, key: &str) -> Result<(), String> {
        let mut iovds = self.iovds.write().unwrap();
        if let Some(idx) = iovds.remove(key) {
            let mut iovs = self.iovs.lock().unwrap();
            iovs.remove(&idx);
            Ok(())
        } else {
            Err(format!("key {} not found", key))
        }
    }

    pub fn lookup_iov(&self, key: &str) -> Option<Arc<ShmBuf>> {
        let iovds = self.iovds.read().unwrap();
        if let Some(idx) = iovds.get(key) {
            let iovs = self.iovs.lock().unwrap();
            iovs.get(idx).cloned()
        } else {
            None
        }
    }
}

// 单元测试示例
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_iovtable_init() {
        let mut table = IovTable::new();
        table.init("/mnt", 128);
        assert_eq!(table.mount_name, "/mnt");
    }
} 