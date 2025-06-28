use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, atomic::{AtomicI32, Ordering}};
use anyhow::{Result, anyhow};

// 假设这些类型是模拟的
pub mod meta {
    use std::sync::atomic::{AtomicU64, Ordering};
    pub type Uid = u32;
    pub type Gid = u32;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct InodeId(pub u64);
    impl InodeId {
        pub const fn getConf() -> Self { InodeId(0xFFFF_FFFF_FFFF_FFF0) }
        pub const fn setConf() -> Self { InodeId(0xFFFF_FFFF_FFFF_FFF1) }

        pub const fn u64(self) -> u64 { self.0 }
    }

    #[derive(Debug, Clone)]
    pub struct UserInfo {
        pub uid: Uid,
        pub gid: Gid,
    }

    #[derive(Debug, Clone)]
    pub struct Inode {
        pub id: InodeId,
        pub data: SymlinkWithAcl,
    }

    #[derive(Debug, Clone)]
    pub struct SymlinkWithAcl {
        pub symlink: String,
        pub acl: Acl,
    }

    #[derive(Debug, Clone)]
    pub struct Acl {
        pub owner_uid: Uid,
        pub owner_gid: Gid,
        pub perm: Permission,
    }

    #[derive(Debug, Clone)]
    pub struct Permission(u16);
    impl Permission {
        pub const fn new(val: u16) -> Self { Permission(val) }
    }

    #[derive(Debug, Clone)]
    pub struct DirEntry {
        pub name: String,
        pub id: InodeId,
    }
}

#[derive(Clone)]
pub struct FuseConfig {
    storage_max_conc_xmit: i32,
    readonly: bool,

    // 简化表示配置项
    values: HashMap<String, String>,
}

impl FuseConfig {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        values.insert("storage.net_client.rdma_control.max_concurrent_transmission".into(), "128".into());
        values.insert("readonly".into(), "false".into());

        Self {
            storage_max_conc_xmit: 128,
            readonly: false,
            values,
        }
    }

    pub fn atomically_update(&mut self, kvs: Vec<(&str, &str)>, _apply: bool) -> Result<()> {
        for (k, v) in kvs {
            if k == "storage.net_client.rdma_control.max_concurrent_transmission" {
                let n = v.parse::<i32>().map_err(|_| anyhow!("invalid number"))?;
                if n <= 0 || n > 2 * self.storage_max_conc_xmit {
                    return Err(anyhow!("invalid value"));
                }
                self.storage_max_conc_xmit = n;
            } else if k == "readonly" {
                if v == "true" && self.readonly {
                    return Err(anyhow!("cannot turn off readonly mode when it is turned on by the sys admin"));
                }
                self.readonly = v == "true";
            }

            self.values.insert(k.to_string(), v.to_string());
        }
        Ok(())
    }

    pub fn find(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }
}

pub struct LocalConfig {
    config: FuseConfig,
    updated_items: Vec<(String, String)>,
}

pub struct UserConfig {
    system_keys: Vec<&'static str>,
    user_keys: Vec<&'static str>,

    global_config: Arc<Mutex<FuseConfig>>,
    local_configs: dashmap::DashMap<u32, Arc<Mutex<LocalConfig>>>,

    storage_max_conc_xmit: AtomicI32,
    users: Mutex<HashSet<meta::Uid>>,
}

impl UserConfig {
    pub fn new() -> Self {
        Self {
            system_keys: vec![
                "storage.net_client.rdma_control.max_concurrent_transmission",
                "periodic_sync.enable",
                "periodic_sync.interval",
                "periodic_sync.flush_write_buf",
                "io_worker_coros.hi",
                "io_worker_coros.lo",
                "max_jobs_per_ioring",
                "io_job_deq_timeout",
            ],
            user_keys: vec![
                "enable_read_cache",
                "readonly",
                "dryrun_bench_mode",
                "flush_on_stat",
                "sync_on_stat",
                "attr_timeout",
                "entry_timeout",
                "negative_timeout",
                "symlink_timeout",
            ],
            global_config: Arc::new(Mutex::new(FuseConfig::new())),
            local_configs: dashmap::DashMap::new(),
            storage_max_conc_xmit: AtomicI32::new(128),
            users: Mutex::new(HashSet::new()),
        }
    }

    pub fn init(&mut self, config: FuseConfig) {
        let mut gconf = self.global_config.lock().unwrap();
        *gconf = config.clone();
        self.storage_max_conc_xmit.store(config.storage_max_conc_xmit, Ordering::Relaxed);

        // Simulate callback guard
        let global_config_clone = self.global_config.clone();
        std::thread::spawn(move || {
            let mut new_conf = (*global_config_clone.lock().unwrap()).clone();
            let mut updated = false;
            if let Some(val) = new_conf.find("storage.net_client.rdma_control.max_concurrent_transmission") {
                if let Ok(n) = val.parse::<i32>() {
                    new_conf.storage_max_conc_xmit = n;
                    updated = true;
                }
            }
            if updated {
                *global_config_clone.lock().unwrap() = new_conf;
            }
        });
    }

    pub fn parse_key(&self, key: &str) -> Result<(bool, usize)> {
        if key.starts_with("sys.") {
            let key = &key[4..];
            if let Some(i) = self.system_keys.iter().position(|&k| k == key) {
                Ok((true, i))
            } else {
                Err(anyhow!("no such system key"))
            }
        } else if key.starts_with("usr.") {
            let key = &key[4..];
            if let Some(i) = self.user_keys.iter().position(|&k| k == key) {
                Ok((false, i))
            } else {
                Err(anyhow!("no such user key"))
            }
        } else {
            Err(anyhow!("key must start with 'sys.' or 'usr.'"))
        }
    }

    pub fn set_config(&self, key: &str, val: &str, ui: &meta::UserInfo) -> Result<meta::Inode> {
        let (is_sys, idx) = self.parse_key(key)?;
        let key = if is_sys { &self.system_keys[idx] } else { &self.user_keys[idx] };

        if is_sys {
            if *key == "storage.net_client.rdma_control.max_concurrent_transmission" {
                let n = val.parse::<i32>().map_err(|_| anyhow!("invalid number"))?;
                if n <= 0 || n > 2 * self.storage_max_conc_xmit.load(Ordering::Relaxed) {
                    return Err(anyhow!("invalid value"));
                }
            }

            let mut conf = self.global_config.lock().unwrap();
            conf.atomically_update(vec![(key, val)], true)?;
            Ok(meta::Inode {
                id: self.config_iid(true, is_sys, idx),
                data: meta::SymlinkWithAcl {
                    symlink: val.to_string(),
                    acl: meta::Acl {
                        owner_uid: ui.uid,
                        owner_gid: ui.gid,
                        perm: meta::Permission::new(0o400),
                    },
                },
            })
        } else {
            if *key == "readonly" && val != "true" && self.global_config.lock().unwrap().readonly {
                return Err(anyhow!("cannot turn off readonly mode when it is turned on by the sys admin"));
            }

            let mut users = self.users.lock().unwrap();
            let uid = ui.uid;
            users.insert(uid);

            let mut lconf = self.local_configs.entry(uid).or_insert_with(|| {
                let base_config = self.global_config.lock().unwrap();
                Arc::new(Mutex::new(LocalConfig {
                    config: base_config.clone(),
                    updated_items: vec![],
                }))
            });

            let mut lc = lconf.value().lock().unwrap();
            lc.config.atomically_update(vec![(key, val)], true)?;
            lc.updated_items.push((key.to_string(), val.to_string()));

            Ok(meta::Inode {
                id: self.config_iid(true, is_sys, idx),
                data: meta::SymlinkWithAcl {
                    symlink: val.to_string(),
                    acl: meta::Acl {
                        owner_uid: ui.uid,
                        owner_gid: ui.gid,
                        perm: meta::Permission::new(0o400),
                    },
                },
            })
        }
    }

    pub fn lookup_config(&self, key: &str, ui: &meta::UserInfo) -> Result<meta::Inode> {
        let (is_sys, idx) = self.parse_key(key)?;
        Ok(self.stat_config(self.config_iid(true, is_sys, idx), ui)?)
    }

    pub fn stat_config(&self, iid: meta::InodeId, ui: &meta::UserInfo) -> Result<meta::Inode> {
        let kidx = meta::InodeId::getConf().u64() - 1 - iid.u64();
        if kidx < 0 || kidx as usize >= self.system_keys.len() + self.user_keys.len() {
            return Err(anyhow!("not a config entry"));
        }

        let is_sys = kidx < self.system_keys.len() as u64;
        let kidx = if is_sys {
            kidx as usize
        } else {
            kidx as usize - self.system_keys.len()
        };
        let key = if is_sys {
            self.system_keys[kidx]
        } else {
            self.user_keys[kidx]
        };

        let config = if is_sys {
            self.global_config.lock().unwrap().clone()
        } else {
            let uid = ui.uid;
            if let Some(lconf) = self.local_configs.get(&uid) {
                lconf.value().lock().unwrap().config.clone()
            } else {
                self.global_config.lock().unwrap().clone()
            }
        };

        let val = config.find(key).ok_or_else(|| anyhow!("key not found"))?;

        Ok(meta::Inode {
            id: iid,
            data: meta::SymlinkWithAcl {
                symlink: val.to_string(),
                acl: meta::Acl {
                    owner_uid: ui.uid,
                    owner_gid: ui.gid,
                    perm: meta::Permission::new(if is_sys { 0o444 } else { 0o400 }),
                },
            },
        })
    }

    pub fn list_config(&self, ui: &meta::UserInfo) -> (Vec<meta::DirEntry>, Vec<Option<meta::Inode>>) {
        let mut des = Vec::new();
        let mut ins = Vec::new();

        for (i, key) in self.system_keys.iter().enumerate() {
            let full_key = format!("sys.{}", key);
            let inode = self.lookup_config(full_key.as_str(), ui).ok().map(|inode| {
                des.push(meta::DirEntry { name: full_key, id: inode.id });
                inode
            });
            ins.push(inode);
        }

        for (i, key) in self.user_keys.iter().enumerate() {
            let full_key = format!("usr.{}", key);
            let inode = self.lookup_config(full_key.as_str(), ui).ok().map(|inode| {
                des.push(meta::DirEntry { name: full_key, id: inode.id });
                inode
            });
            ins.push(inode);
        }

        (des, ins)
    }

    fn config_iid(&self, is_get: bool, is_sys: bool, kidx: usize) -> meta::InodeId {
        let base = if is_get {
            meta::InodeId::getConf().u64()
        } else {
            meta::InodeId::setConf().u64()
        };
        let offset = if is_sys {
            0
        } else {
            self.system_keys.len()
        } + kidx;
        meta::InodeId(base - 1 - offset as u64)
    }
}