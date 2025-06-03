// src/fuse/iov_table.rs

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    path::{Path, PathBuf},
    time::{Duration, Instant},
    fs,
    os::unix::fs::FileTypeExt,
};

// 类型定义
#[derive(Debug, Clone)]
pub struct Uuid([u8; 16]);

impl Uuid {
    pub fn from_hex_string(s: &str) -> Result<Self, String> {
        if s.len() != 32 {
            return Err("Invalid UUID length".to_string());
        }
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&s[i*2..i*2+2], 16)
                .map_err(|_| "Invalid hex string".to_string())?;
        }
        Ok(Uuid(bytes))
    }
}

#[derive(Debug, Clone)]
pub struct MetaUserInfo {
    pub uid: u32,
    pub gid: u32,
}

#[derive(Debug, Clone)]
pub struct InodeId {
    value: u64,
}

impl InodeId {
    pub fn u64(&self) -> u64 {
        self.value
    }

    pub fn iov_dir() -> Self {
        Self { value: 0x1000 } // 示例值
    }

    pub fn iov(i: i32) -> Self {
        Self { value: 0x1000 - i as u64 }
    }
}

#[derive(Debug, Clone)]
pub struct InodeData {
    pub symlink: Symlink,
    pub acl: Acl,
}

#[derive(Debug, Clone)]
pub struct Symlink {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Acl {
    pub uid: Uid,
    pub gid: Gid,
    pub permission: Permission,
}

#[derive(Debug, Clone)]
pub struct Uid(pub u32);

#[derive(Debug, Clone)]
pub struct Gid(pub u32);

#[derive(Debug, Clone)]
pub struct Permission(pub u32);

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub inode_id: InodeId,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MetaInode {
    pub id: InodeId,
    pub data: InodeData,
}

#[derive(Debug, Clone)]
pub struct IorAttrs {
    pub timeout: Duration,
    pub flags: u64,
    pub priority: i32,
}

impl Default for IorAttrs {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(0),
            flags: 0,
            priority: 1,
        }
    }
}

#[derive(Debug)]
pub struct ShmBuf {
    pub path: PathBuf,
    pub size: usize,
    pub block_size: usize,
    pub id: Uuid,
    pub key: String,
    pub user: u32,
    pub pid: i32,
    pub is_io_ring: bool,
    pub for_read: bool,
    pub io_depth: i32,
    pub iora: Option<IorAttrs>,
}

impl ShmBuf {
    pub fn new(path: PathBuf, offset: usize, size: usize, block_size: usize, id: Uuid) -> Self {
        Self {
            path,
            size,
            block_size,
            id,
            key: String::new(),
            user: 0,
            pid: 0,
            is_io_ring: false,
            for_read: true,
            io_depth: 0,
            iora: None,
        }
    }

    pub async fn register_for_io(&self, exec: &str, sc: &str) -> Result<(), String> {
        // 实现 IO 注册逻辑
        Ok(())
    }

    pub async fn deregister_for_io(&self) -> Result<(), String> {
        // 实现 IO 注销逻辑
        Ok(())
    }
}

// 监控相关结构体
#[derive(Clone)]
struct DistributionRecorder {
    name: String,
    tags: Vec<(String, String)>,
}

impl DistributionRecorder {
    fn new(name: &str, tags: Vec<(String, String)>) -> Self {
        Self {
            name: name.to_string(),
            tags,
        }
    }

    fn add_sample(&self, value: u64, tags: Vec<(String, String)>) {
        // 实现监控记录逻辑
    }
}

#[derive(Clone)]
struct CountRecorder {
    name: String,
    tags: Vec<(String, String)>,
}

impl CountRecorder {
    fn new(name: &str, tags: Vec<(String, String)>) -> Self {
        Self {
            name: name.to_string(),
            tags,
        }
    }

    fn add_sample(&self, value: i64, tags: Vec<(String, String)>) {
        // 实现计数记录逻辑
    }
}

#[derive(Clone)]
struct LatencyRecorder {
    name: String,
    tags: Vec<(String, String)>,
}

impl LatencyRecorder {
    fn new(name: &str, tags: Vec<(String, String)>) -> Self {
        Self {
            name: name.to_string(),
            tags,
        }
    }

    fn add_sample(&self, duration: Duration, tags: Vec<(String, String)>) {
        // 实现延迟记录逻辑
    }
}

// 错误码枚举
#[derive(Debug)]
enum StatusCode {
    kInvalidArg = 1,
}

#[derive(Debug)]
enum MetaCode {
    kNotFound = 1,
    kNoPermission = 2,
}

#[derive(Debug)]
enum ClientAgentCode {
    kTooManyOpenFiles = 1,
    kIovShmFail = 2,
}

// 辅助函数
fn make_error<T>(code: i32, msg: &str) -> Result<T, String> {
    Err(format!("Error {}: {}", code, msg))
}

// IovTable 实现
pub struct IovTable {
    pub mount_name: String,
    pub shm_lock: RwLock<()>,
    pub shms_by_id: HashMap<Uuid, i32>,
    pub iovs: Arc<Mutex<Vec<Option<Arc<ShmBuf>>>>>,
    iovd_lock: RwLock<()>,
    iovds: HashMap<String, i32>,
}

impl IovTable {
    pub fn new() -> Self {
        Self {
            mount_name: String::new(),
            shm_lock: RwLock::new(()),
            shms_by_id: HashMap::new(),
            iovs: Arc::new(Mutex::new(Vec::new())),
            iovd_lock: RwLock::new(()),
            iovds: HashMap::new(),
        }
    }

    pub fn init(&mut self, mount: &Path, cap: i32) {
        self.mount_name = mount.to_string_lossy().to_string();
        let mut iovs = self.iovs.lock().unwrap();
        *iovs = vec![None; cap as usize];
    }

    pub fn add_iov(
        &self,
        key: &str,
        shm_path: &Path,
        pid: i32,
        ui: &MetaUserInfo,
        exec: &str,
        sc: &str,
    ) -> Result<(MetaInode, Option<Arc<ShmBuf>>), String> {
        // 监控记录器
        let map_times_count = DistributionRecorder::new(
            "fuse.iov.times",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let map_bytes_dist = DistributionRecorder::new(
            "fuse.iov.bytes",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let shm_size_count = CountRecorder::new(
            "fuse.iov.total_bytes",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let alloc_latency = LatencyRecorder::new(
            "fuse.iov.latency.map",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let ib_reg_bytes_dist = DistributionRecorder::new(
            "fuse.iov.bytes.ib_reg",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let ib_reg_latency = LatencyRecorder::new(
            "fuse.iov.latency.ib_reg",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );

        // 解析 key
        let iova = parse_key(key)?;

        // 构建共享内存路径
        let shm_open_path = PathBuf::from("/").join(
            shm_path.strip_prefix("/dev/shm").unwrap_or(shm_path)
        );

        // 检查文件状态
        let metadata = fs::metadata(shm_path)
            .map_err(|e| format!("Failed to stat shm path: {}", e))?;
        if !metadata.file_type().is_file() {
            return make_error(StatusCode::kInvalidArg as i32, "shm path is not a regular file");
        }

        if iova.block_size > metadata.len() as usize {
            return make_error(StatusCode::kInvalidArg as i32, "invalid block size set in shm key");
        }

        // 分配 iov 描述符
        let mut iovs = self.iovs.lock().unwrap();
        let iovd = iovs.iter().position(|x| x.is_none())
            .ok_or_else(|| make_error(ClientAgentCode::kTooManyOpenFiles as i32, "too many iovs allocated"))?;

        // 创建共享内存
        let start = Instant::now();
        let uids = ui.uid.to_string();

        let shm = Arc::new(ShmBuf::new(
            shm_open_path,
            0,
            metadata.len() as usize,
            iova.block_size,
            iova.id,
        ));

        // 设置共享内存属性
        let mut shm = shm;
        shm.key = key.to_string();
        shm.user = ui.uid;
        shm.pid = pid;
        shm.is_io_ring = iova.is_io_ring;
        shm.for_read = iova.for_read;
        shm.io_depth = iova.io_depth;
        shm.iora = iova.iora;

        // 存储共享内存
        iovs[iovd] = Some(shm.clone());

        // 记录监控数据
        alloc_latency.add_sample(start.elapsed(), vec![
            ("instance".to_string(), "alloc".to_string()),
            ("uid".to_string(), uids.clone()),
        ]);
        map_times_count.add_sample(1, vec![
            ("instance".to_string(), "alloc".to_string()),
            ("uid".to_string(), uids.clone()),
        ]);
        map_bytes_dist.add_sample(shm.size as u64, vec![
            ("instance".to_string(), "alloc".to_string()),
            ("uid".to_string(), uids.clone()),
        ]);
        shm_size_count.add_sample(shm.size as i64, vec![("uid".to_string(), uids.clone())]);

        // 注册 IO（非 IO 环）
        if !iova.is_io_ring {
            let start = Instant::now();
            shm.register_for_io(exec, sc).await
                .map_err(|e| format!("Failed to register for IO: {}", e))?;
            ib_reg_bytes_dist.add_sample(iova.block_size as u64, vec![
                ("instance".to_string(), "reg".to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
            ib_reg_latency.add_sample(start.elapsed(), vec![
                ("instance".to_string(), "reg".to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
        }

        // 更新索引
        {
            let mut iovds = self.iovd_lock.write().unwrap();
            iovds.insert(key.to_string(), iovd as i32);
        }
        {
            let mut shms_by_id = self.shm_lock.write().unwrap();
            shms_by_id.insert(iova.id, iovd as i32);
        }

        // 获取 inode 信息
        let stat_res = self.stat_iov(iovd as i32, ui)?;
        Ok((stat_res, if iova.is_io_ring { Some(shm) } else { None }))
    }

    pub fn rm_iov(&self, key: &str, ui: &MetaUserInfo) -> Result<Option<Arc<ShmBuf>>, String> {
        let res = self.lookup_iov(key, ui)?;
        {
            let mut iovds = self.iovd_lock.write().unwrap();
            iovds.remove(key);
        }
        {
            let res = parse_key(key)?;
            let mut shms_by_id = self.shm_lock.write().unwrap();
            shms_by_id.remove(&res.id);
        }
        let iovd = self.iov_desc(res.id)?;
        let mut iovs = self.iovs.lock().unwrap();
        let shm = iovs[iovd as usize].take();
        Ok(shm)
    }

    pub fn stat_iov(&self, iovd: i32, ui: &MetaUserInfo) -> Result<MetaInode, String> {
        if iovd < 0 || iovd >= self.iovs.lock().unwrap().len() as i32 {
            return make_error(MetaCode::kNotFound as i32, "invalid iov desc");
        }
        let iovs = self.iovs.lock().unwrap();
        let shm = iovs[iovd as usize].as_ref()
            .ok_or_else(|| make_error(MetaCode::kNotFound as i32, format!("iov desc {} not found", iovd)))?;
        if shm.user != ui.uid {
            return make_error(MetaCode::kNoPermission as i32, "iov not for user");
        }
        Ok(MetaInode {
            id: InodeId::iov(iovd),
            data: InodeData {
                symlink: Symlink {
                    path: PathBuf::from("/dev/shm").join(&shm.path),
                },
                acl: Acl {
                    uid: Uid(ui.uid),
                    gid: Gid(ui.gid),
                    permission: Permission(0o400),
                },
            },
        })
    }

    pub fn lookup_iov(&self, key: &str, ui: &MetaUserInfo) -> Result<MetaInode, String> {
        let iovd = {
            let iovds = self.iovd_lock.read().unwrap();
            *iovds.get(key).ok_or_else(|| make_error(MetaCode::kNotFound as i32, format!("iov key not found {}", key)))?
        };
        self.stat_iov(iovd, ui)
    }

    pub fn iov_desc(&self, iid: InodeId) -> Option<i32> {
        let iidn = iid.u64() as isize;
        let diid = InodeId::iov_dir().u64() as isize;
        if iidn >= 0 || iidn > diid - iov_iid_start() || iidn < diid - std::i32::MAX as isize {
            return None;
        }
        Some((diid - iidn - iov_iid_start()) as i32)
    }

    pub fn list_iovs(&self, ui: &MetaUserInfo) -> (Vec<DirEntry>, Vec<Option<MetaInode>>) {
        let mut des = Vec::new();
        let mut ins = Vec::new();
        let n = self.iovs.lock().unwrap().len();
        des.reserve(n + 3);
        ins.reserve(n + 3);

        // 添加信号量条目
        for prio in 0..=2 {
            let de = DirEntry {
                inode_id: InodeId::iov_dir(),
                name: io_ring_table::sem_name(prio),
            };
            des.push(de);
            let inode = io_ring_table::lookup_sem(prio);
            ins.push(Some(inode));
        }

        // 添加用户 iov 条目
        let acl = Acl {
            uid: Uid(ui.uid),
            gid: Gid(ui.gid),
            permission: Permission(0o400),
        };
        let iovs = self.iovs.lock().unwrap();
        for i in 0..n {
            if let Some(iov) = &iovs[i] {
                if iov.user != ui.uid {
                    continue;
                }
                let de = DirEntry {
                    inode_id: InodeId::iov(i as i32),
                    name: iov.key.clone(),
                };
                des.push(de);
                ins.push(Some(MetaInode {
                    id: InodeId::iov(i as i32),
                    data: InodeData {
                        symlink: Symlink {
                            path: PathBuf::from("/dev/shm").join(&iov.path),
                        },
                        acl: acl.clone(),
                    },
                }));
            }
        }

        (des, ins)
    }
}

// 辅助函数
fn parse_key(key: &str) -> Result<IovAttrs, String> {
    let mut iova = IovAttrs {
        id: Uuid([0; 16]),
        block_size: 0,
        is_io_ring: false,
        for_read: true,
        io_depth: 0,
        iora: None,
    };

    let parts: Vec<&str> = key.split('.').collect();
    iova.id = Uuid::from_hex_string(parts[0])?;

    for part in parts.iter().skip(1) {
        match part.chars().next() {
            Some('b') => {
                let size = part[1..].parse::<usize>()
                    .map_err(|_| make_error(StatusCode::kInvalidArg as i32, "invalid block size set in shm key"))?;
                if size <= 0 {
                    return make_error(StatusCode::kInvalidArg as i32, "invalid block size set in shm key");
                }
                iova.block_size = size;
            }
            Some('r') | Some('w') => {
                let depth = part[1..].parse::<i32>()
                    .map_err(|_| make_error(StatusCode::kInvalidArg as i32, "invalid io depth set in shm key"))?;
                iova.is_io_ring = true;
                iova.for_read = part.starts_with('r');
                iova.io_depth = depth;
            }
            Some('t') => {
                if iova.iora.is_none() {
                    iova.iora = Some(IorAttrs::default());
                }
                let timeout = part[1..].parse::<i32>()
                    .map_err(|_| make_error(StatusCode::kInvalidArg as i32, "invalid timeout set in shm key"))?;
                if timeout < 0 {
                    return make_error(StatusCode::kInvalidArg as i32, "invalid timeout set in shm key");
                }
                iova.iora.as_mut().unwrap().timeout = Duration::from_millis(timeout as u64);
            }
            Some('f') => {
                if iova.iora.is_none() {
                    iova.iora = Some(IorAttrs::default());
                }
                let flags = u64::from_str_radix(&part[1..], 2)
                    .map_err(|_| make_error(StatusCode::kInvalidArg as i32, "invalid flags set in shm key"))?;
                iova.iora.as_mut().unwrap().flags = flags;
            }
            Some('p') => {
                if iova.iora.is_none() {
                    iova.iora = Some(IorAttrs::default());
                }
                match part.chars().nth(1) {
                    Some('l') => iova.iora.as_mut().unwrap().priority = 2,
                    Some('h') => iova.iora.as_mut().unwrap().priority = 0,
                    Some('n') | None => iova.iora.as_mut().unwrap().priority = 1,
                    _ => return make_error(StatusCode::kInvalidArg as i32, "invalid priority set in shm key"),
                }
            }
            _ => {}
        }
    }

    if !iova.is_io_ring && iova.iora.is_some() {
        return make_error(StatusCode::kInvalidArg as i32, "ioring attrs set for non-ioring");
    }

    Ok(iova)
}

// 常量
fn iov_iid_start() -> isize {
    0 // 需根据实际定义调整
}

// 模块
mod io_ring_table {
    pub fn sem_name(prio: i32) -> String {
        format!("submit-ios{}", match prio {
            0 => ".ph",
            1 => "",
            2 => ".pl",
            _ => "",
        })
    }

    pub fn lookup_sem(prio: i32) -> super::MetaInode {
        super::MetaInode {
            id: super::InodeId::iov_dir(),
            data: super::InodeData {
                symlink: super::Symlink {
                    path: std::path::PathBuf::from("/dev/shm").join(format!("sem.{}", sem_name(prio))),
                },
                acl: super::Acl {
                    uid: super::Uid(0),
                    gid: super::Gid(0),
                    permission: super::Permission(0o666),
                },
            },
        }
    }
}

#[derive(Debug)]
struct IovAttrs {
    id: Uuid,
    block_size: usize,
    is_io_ring: bool,
    for_read: bool,
    io_depth: i32,
    iora: Option<IorAttrs>,
}