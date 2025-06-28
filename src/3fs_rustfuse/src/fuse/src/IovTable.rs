use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use uuid::Uuid;
use anyhow::{Result, bail};
use tokio_uring::fs::File;

#[derive(Clone, Debug)]
pub struct UserInfo {
    pub uid: u32,
    pub gid: u32,
}

#[derive(Clone, Debug)]
pub struct IovAttrs {
    pub id: Uuid,
    pub block_size: usize,
    pub is_io_ring: bool,
    pub for_read: bool,
    pub io_depth: usize,
    pub timeout: Option<Duration>,
    pub flags: Option<u64>,
    pub priority: Option<u8>,
}

impl Default for IovAttrs {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            block_size: 0,
            is_io_ring: false,
            for_read: true,
            io_depth: 0,
            timeout: None,
            flags: None,
            priority: None,
        }
    }
}

/// 解析key字符串，提取IovAttrs
pub fn parse_key(key: &str) -> Result<IovAttrs> {
    let mut attrs = IovAttrs::default();
    let parts: Vec<&str> = key.split('.').collect();
    attrs.id = Uuid::parse_str(parts[0])?;
    for part in &parts[1..] {
        match &part[..1] {
            "b" => attrs.block_size = part[1..].parse()?,
            "r" | "w" => {
                attrs.is_io_ring = true;
                attrs.for_read = &part[..1] == "r";
                attrs.io_depth = part[1..].parse()?;
            }
            "t" => attrs.timeout = Some(Duration::from_millis(part[1..].parse()?)),
            "f" => attrs.flags = Some(u64::from_str_radix(&part[1..], 2)?),
            "p" => attrs.priority = match &part[1..] {
                "l" => Some(2),
                "h" => Some(0),
                "n" | "" => Some(1),
                _ => bail!("invalid priority"),
            },
            _ => {}
        }
    }
    Ok(attrs)
}

#[derive(Debug)]
pub struct ShmBuf {
    pub id: Uuid,
    pub path: PathBuf,
    pub size: usize,
    pub block_size: usize,
    pub user: u32,
    pub is_io_ring: bool,
    pub for_read: bool,
    pub io_depth: usize,
    pub priority: Option<u8>,
    pub timeout: Option<Duration>,
    pub flags: Option<u64>,
    pub file: Arc<File>,
}

pub struct IovTable {
    pub mount_name: String,
    pub iovs: Arc<Mutex<HashMap<String, Arc<ShmBuf>>>>,
    pub shms_by_id: Arc<Mutex<HashMap<Uuid, String>>>,
}

impl IovTable {
    pub fn new(mount_name: &str) -> Self {
        Self {
            mount_name: mount_name.to_string(),
            iovs: Arc::new(Mutex::new(HashMap::new())),
            shms_by_id: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 添加一个 IOV
    pub async fn add_iov(
        &self,
        key: &str,
        shm_path: &Path,
        user: &UserInfo,
    ) -> Result<Arc<ShmBuf>> {
        let attrs = parse_key(key)?;
        let meta = tokio::fs::metadata(shm_path).await?;
        if !meta.is_file() {
            bail!("shm_path is not a file");
        }
        if attrs.block_size > meta.len() as usize {
            bail!("block_size too large");
        }
        // 打开文件
        let file = File::open(shm_path).await?;
        let shmbuf = Arc::new(ShmBuf {
            id: attrs.id,
            path: shm_path.to_path_buf(),
            size: meta.len() as usize,
            block_size: attrs.block_size,
            user: user.uid,
            is_io_ring: attrs.is_io_ring,
            for_read: attrs.for_read,
            io_depth: attrs.io_depth,
            priority: attrs.priority,
            timeout: attrs.timeout,
            flags: attrs.flags,
            file: Arc::new(file),
        });
        self.iovs.lock().await.insert(key.to_string(), shmbuf.clone());
        self.shms_by_id.lock().await.insert(attrs.id, key.to_string());
        Ok(shmbuf)
    }

    /// 删除一个 IOV
    pub async fn rm_iov(&self, key: &str, user: &UserInfo) -> Result<Arc<ShmBuf>> {
        let mut iovs = self.iovs.lock().await;
        let shmbuf = iovs.get(key).ok_or_else(|| anyhow::anyhow!("not found"))?;
        if shmbuf.user != user.uid {
            bail!("no permission");
        }
        let shmbuf = iovs.remove(key).unwrap();
        self.shms_by_id.lock().await.remove(&shmbuf.id);
        Ok(shmbuf)
    }

    /// 查找一个 IOV
    pub async fn lookup_iov(&self, key: &str, user: &UserInfo) -> Result<Arc<ShmBuf>> {
        let iovs = self.iovs.lock().await;
        let shmbuf = iovs.get(key).ok_or_else(|| anyhow::anyhow!("not found"))?;
        if shmbuf.user != user.uid {
            bail!("no permission");
        }
        Ok(shmbuf.clone())
    }

    /// 列出所有 IOV
    pub async fn list_iovs(&self, user: &UserInfo) -> Vec<Arc<ShmBuf>> {
        let iovs = self.iovs.lock().await;
        iovs.values().filter(|shm| shm.user == user.uid).cloned().collect()
    }
}