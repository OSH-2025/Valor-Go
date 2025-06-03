// src/fuse/user_config.rs

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
    vec::Vec,
};

use crate::{
    common::{
        Result,
        StatusCode,
        MetaCode,
        AtomicSharedPtrTable,
    },
    meta::{
        Inode,
        InodeId,
        InodeData,
        Symlink,
        Acl,
        Permission,
        UserInfo,
        Uid,
        Gid,
    },
    config::{
        FuseConfig,
        KeyValue,
    },
};

/// 用户配置管理结构体
pub struct UserConfig {
    config: Arc<RwLock<FuseConfig>>,
    configs: AtomicSharedPtrTable<LocalConfig>,
    users: Mutex<HashSet<Uid>>,
    storage_max_conc_xmit: std::sync::atomic::AtomicI32,
}

/// 本地配置结构体
struct LocalConfig {
    mtx: Mutex<()>,
    config: FuseConfig,
    updated_items: Vec<KeyValue>,
}

impl UserConfig {
    /// 创建新的 UserConfig 实例
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(FuseConfig::default())),
            configs: AtomicSharedPtrTable::new(0),
            users: Mutex::new(HashSet::new()),
            storage_max_conc_xmit: std::sync::atomic::AtomicI32::new(0),
        }
    }

    /// 初始化配置
    pub fn init(&mut self, config: FuseConfig) {
        let max_uid = config.max_uid();
        self.configs = AtomicSharedPtrTable::new(max_uid + 1);
        
        self.storage_max_conc_xmit.store(
            config.storage().net_client().rdma_control().max_concurrent_transmission(),
            std::sync::atomic::Ordering::SeqCst,
        );

        // 添加配置更新回调
        let config_arc = Arc::clone(&self.config);
        let users = Arc::new(Mutex::new(HashSet::new()));
        let configs = Arc::new(self.configs.clone());
        let storage_max_conc_xmit = Arc::new(self.storage_max_conc_xmit.clone());

        config.add_callback_guard(move |new_config| {
            storage_max_conc_xmit.store(
                new_config.storage().net_client().rdma_control().max_concurrent_transmission(),
                std::sync::atomic::Ordering::SeqCst,
            );

            let users = users.lock().unwrap();
            for &uid in users.iter() {
                if let Some(lconf) = configs.table[uid.to_under_type()].load() {
                    let mut conf2 = new_config.clone();
                    let _lock = lconf.mtx.lock().unwrap();
                    conf2.atomically_update(&lconf.updated_items, true);
                    lconf.config = conf2;
                }
            }
        });

        *self.config.write().unwrap() = config;
    }

    /// 解析配置键
    fn parse_key(&self, key: &str) -> Result<(bool, usize)> {
        if key.starts_with("sys.") {
            if let Some(idx) = SYSTEM_KEYS.iter().position(|&k| k == &key[4..]) {
                Ok((true, idx))
            } else {
                Err(StatusCode::kInvalidArg.into())
            }
        } else if key.starts_with("usr.") {
            if let Some(idx) = USER_KEYS.iter().position(|&k| k == &key[4..]) {
                Ok((false, idx))
            } else {
                Err(StatusCode::kInvalidArg.into())
            }
        } else {
            Err(StatusCode::kInvalidArg.into())
        }
    }

    /// 设置配置
    pub fn set_config(&mut self, key: &str, val: &str, ui: &UserInfo) -> Result<Inode> {
        let (is_sys, kidx) = self.parse_key(key)?;
        let key = &key[4..];

        if is_sys {
            if key == "storage.net_client.rdma_control.max_concurrent_transmission" {
                let n = val.parse::<i32>().map_err(|_| StatusCode::kInvalidArg)?;
                if n <= 0 || n > 2 * self.storage_max_conc_xmit.load(std::sync::atomic::Ordering::SeqCst) {
                    return Err(StatusCode::kInvalidArg.into());
                }
            }

            self.config.write().unwrap().atomically_update(
                &[KeyValue::new(key.to_string(), val.to_string())],
                true,
            )?;

            Ok(Inode::new(
                self.config_iid(false, true, kidx),
                InodeData::new(
                    Symlink::new(val.to_string()),
                    Acl::new(ui.uid, ui.gid, Permission::new(0o400)),
                ),
            ))
        } else {
            if key == "readonly" && val != "true" && self.config.read().unwrap().readonly() {
                return Err(StatusCode::kInvalidArg.into());
            }

            let uid = ui.uid;
            let mut users = self.users.lock().unwrap();
            let uidx = uid.to_under_type();

            if !users.contains(&uid) {
                if uidx >= self.configs.table.len() {
                    return Err(MetaCode::kNoPermission.into());
                }

                self.configs.table[uidx].store(Arc::new(LocalConfig {
                    mtx: Mutex::new(()),
                    config: self.config.read().unwrap().clone(),
                    updated_items: Vec::new(),
                }));
                users.insert(uid);
            }

            let lconf = self.configs.table[uidx].load().unwrap();
            let kv = KeyValue::new(key.to_string(), val.to_string());
            
            let _lock = lconf.mtx.lock().unwrap();
            lconf.config.atomically_update(&[kv.clone()], true)?;
            lconf.updated_items.push(kv);

            Ok(Inode::new(
                self.config_iid(false, false, kidx),
                InodeData::new(
                    Symlink::new(val.to_string()),
                    Acl::new(ui.uid, ui.gid, Permission::new(0o400)),
                ),
            ))
        }
    }

    /// 查找配置
    pub fn lookup_config(&self, key: &str, ui: &UserInfo) -> Result<Inode> {
        let (is_sys, kidx) = self.parse_key(key)?;
        self.stat_config(self.config_iid(true, is_sys, kidx), ui)
    }

    /// 获取用户配置
    pub fn get_config(&self, ui: &UserInfo) -> Arc<RwLock<FuseConfig>> {
        let uid = ui.uid;
        let users = self.users.lock().unwrap();
        
        if users.contains(&uid) {
            if let Some(lconf) = self.configs.table[uid.to_under_type()].load() {
                Arc::new(RwLock::new(lconf.config.clone()))
            } else {
                Arc::clone(&self.config)
            }
        } else {
            Arc::clone(&self.config)
        }
    }

    /// 获取配置状态
    pub fn stat_config(&self, iid: InodeId, ui: &UserInfo) -> Result<Inode> {
        let kidx = (InodeId::get_conf().u64() - 1 - iid.u64()) as i64;
        if kidx < 0 || kidx >= (SYSTEM_KEYS.len() + USER_KEYS.len()) as i64 {
            return Err(MetaCode::kNotFound.into());
        }

        let is_sys = kidx < SYSTEM_KEYS.len() as i64;
        let kidx = if !is_sys {
            (kidx - SYSTEM_KEYS.len() as i64) as usize
        } else {
            kidx as usize
        };

        let config = if is_sys {
            self.config.read().unwrap().clone()
        } else {
            self.get_config(ui).read().unwrap().clone()
        };

        let key = if is_sys { SYSTEM_KEYS[kidx] } else { USER_KEYS[kidx] };
        let val = config.find(key).unwrap().to_string();

        Ok(Inode::new(
            iid,
            InodeData::new(
                Symlink::new(val),
                Acl::new(
                    ui.uid,
                    ui.gid,
                    Permission::new(if is_sys { 0o444 } else { 0o400 }),
                ),
            ),
        ))
    }

    /// 列出所有配置
    pub fn list_config(&self, ui: &UserInfo) -> (Vec<DirEntry>, Vec<Option<Inode>>) {
        let mut des = Vec::with_capacity(SYSTEM_KEYS.len() + USER_KEYS.len());
        let mut ins = Vec::with_capacity(SYSTEM_KEYS.len() + USER_KEYS.len());

        for k in SYSTEM_KEYS.iter() {
            let key = format!("sys.{}", k);
            des.push(DirEntry::new(InodeId::get_conf(), key));
            ins.push(Some(self.lookup_config(&key, ui).unwrap()));
        }

        for k in USER_KEYS.iter() {
            let key = format!("usr.{}", k);
            des.push(DirEntry::new(InodeId::get_conf(), key));
            ins.push(Some(self.lookup_config(&key, ui).unwrap()));
        }

        (des, ins)
    }

    /// 生成配置 inode ID
    fn config_iid(&self, is_get: bool, is_sys: bool, kidx: usize) -> InodeId {
        let base = if is_get {
            InodeId::get_conf()
        } else {
            InodeId::set_conf()
        };
        
        InodeId::new(
            base.u64() - 1 - (if is_sys { 0 } else { SYSTEM_KEYS.len() }) - kidx
        )
    }
}

/// 系统配置键
const SYSTEM_KEYS: &[&str] = &[
    "storage.net_client.rdma_control.max_concurrent_transmission",
    "periodic_sync.enable",
    "periodic_sync.interval",
    "periodic_sync.flush_write_buf",
    "io_worker_coros.hi",
    "io_worker_coros.lo",
    "max_jobs_per_ioring",
    "io_job_deq_timeout",
];

/// 用户配置键
const USER_KEYS: &[&str] = &[
    "enable_read_cache",
    "readonly",
    "dryrun_bench_mode",
    "flush_on_stat",
    "sync_on_stat",
    "attr_timeout",
    "entry_timeout",
    "negative_timeout",
    "symlink_timeout",
];

/// 目录项结构体
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub inode_id: InodeId,
    pub name: String,
}

impl DirEntry {
    pub fn new(inode_id: InodeId, name: String) -> Self {
        Self { inode_id, name }
    }
}

/// 权限结构体
#[derive(Debug, Clone)]
pub struct Permission(u32);

impl Permission {
    pub fn new(mode: u32) -> Self {
        Self(mode)
    }
}

/// 错误码定义
#[derive(Debug)]
pub enum StatusCode {
    kInvalidArg = 1,
}

#[derive(Debug)]
pub enum MetaCode {
    kNotFound = 1,
    kNoPermission = 2,
}

/// 配置键值对
#[derive(Debug, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

impl KeyValue {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}