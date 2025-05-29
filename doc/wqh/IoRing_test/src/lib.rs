use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, atomic::{AtomicI32, Ordering}};
use anyhow::{Result, anyhow};
use once_cell::sync::OnceCell;

// Mock FuseConfig and related structs
#[derive(Clone, Debug)]
pub struct FuseConfig {
    data: HashMap<String, String>,
    max_uid: u32,
    pub storage_max_conc_xmit: i32,
}

impl FuseConfig {
    pub fn new() -> Self {
        let mut data = HashMap::new();
        data.insert("storage.net_client.rdma_control.max_concurrent_transmission".to_string(), "64".to_string());
        data.insert("readonly".to_string(), "false".to_string());

        Self {
            data,
            max_uid: 1000,
            storage_max_conc_xmit: 64,
        }
    }

    pub fn max_uid(&self) -> u32 {
        self.max_uid
    }

    pub fn readonly(&self) -> bool {
        self.data.get("readonly").map(|v| v == "true").unwrap_or(false)
    }

    pub fn atomically_update(&mut self, items: Vec<(String, String)>, _apply: bool) -> Result<()> {
        for (k, v) in items {
            if k == "storage.net_client.rdma_control.max_concurrent_transmission" && v.parse::<i32>().unwrap_or(0) <= 0 {
                return Err(anyhow!("Invalid value for max_concurrent_transmission"));
            }
            self.data.insert(k, v);
        }
        Ok(())
    }

    pub fn find(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
}

// UserInfo mock
#[derive(Clone)]
pub struct UserInfo {
    pub uid: Uid,
    pub gid: Gid,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uid(pub u32);

impl Uid {
    pub fn to_under_type(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy)]
pub struct Gid(pub u32);

// Inode mock
#[derive(Clone, Copy)]
pub struct InodeId(u64);

impl InodeId {
    pub const fn get_conf() -> Self {
        Self(0x1000_0000)
    }

    pub const fn set_conf() -> Self {
        Self(0x2000_0000)
    }

    pub const fn u64(&self) -> u64 {
        self.0
    }
}

#[derive(Clone)]
pub struct Acl {
    pub owner: Uid,
    pub group: Gid,
    pub perm: Permission,
}

#[derive(Clone)]
pub struct Permission(u16);

#[derive(Clone)]
pub struct Symlink {
    pub target: String,
}

#[derive(Clone)]
pub struct Inode {
    pub id: InodeId,
    pub symlink: Symlink,
    pub acl: Acl,
}

pub type DirEntry = Inode;
pub type MetaInode = Inode;

pub struct UserConfig {
    config: OnceCell<Arc<Mutex<FuseConfig>>>,
    system_keys: Vec<&'static str>,
    user_keys: Vec<&'static str>,
    configs: Mutex<Vec<Option<Arc<Mutex<LocalConfig>>>>>,
    users: Mutex<HashSet<Uid>>,
    storage_max_conc_xmit: AtomicI32,
}

#[derive(Debug)]
struct LocalConfig {
    config: FuseConfig,
    updated_items: Vec<(String, String)>,
}

impl UserConfig {
    pub fn new() -> Self {
        Self {
            config: OnceCell::new(),
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
            configs: Mutex::new(vec![]),
            users: Mutex::new(HashSet::new()),
            storage_max_conc_xmit: AtomicI32::new(64),
        }
    }

    pub fn init(&self, global_config: FuseConfig) {
        let mut configs = self.configs.lock().unwrap();
        configs.resize(global_config.max_uid() as usize + 1, None);
        self.storage_max_conc_xmit
            .store(global_config.storage_max_conc_xmit, Ordering::Relaxed);
        self.config.set(Arc::new(Mutex::new(global_config))).unwrap();
    }

    pub fn parse_key(&self, key: &str) -> Result<(bool, usize)> {
        if let Some(rest) = key.strip_prefix("sys.") {
            if let Some(index) = self.system_keys.iter().position(|&k| k == rest) {
                return Ok((true, index));
            } else {
                return Err(anyhow!("No such system key or not customizable: {}", key));
            }
        } else if let Some(rest) = key.strip_prefix("usr.") {
            if let Some(index) = self.user_keys.iter().position(|&k| k == rest) {
                return Ok((false, index));
            } else {
                return Err(anyhow!("No such user key or not customizable: {}", key));
            }
        } else {
            return Err(anyhow!("Key must be prefixed with 'sys.' or 'usr.': {}", key));
        }
    }

    pub fn set_config(&self, key: &str, val: &str, ui: &UserInfo) -> Result<Inode> {
        let (is_sys, idx) = self.parse_key(key)?;
        let key = key.split_at(4).1.to_string();

        if is_sys {
            if key == "storage.net_client.rdma_control.max_concurrent_transmission" {
                let n = val.parse::<i32>().unwrap_or(0);
                let limit = self.storage_max_conc_xmit.load(Ordering::Relaxed) * 2;
                if n <= 0 || n > limit {
                    return Err(anyhow!("Invalid value for key '{}'", key));
                }
            }

            let mut config = self.config.get().unwrap().lock().unwrap();
            config.atomically_update(vec![(key, val.to_string())], true)?;

            Ok(Inode {
                id: self.config_iid(true, true, idx),
                symlink: Symlink { target: val.to_string() },
                acl: Acl {
                    owner: ui.uid,
                    group: ui.gid,
                    perm: Permission(0o400),
                },
            })
        } else {
            if key == "readonly" && val != "true" && self.config.get().unwrap().lock().unwrap().readonly() {
                return Err(anyhow!("Cannot turn off readonly mode when sys admin turned it on."));
            }

            let mut users = self.users.lock().unwrap();
            let uidx = ui.uid.to_under_type();
            if uidx >= self.configs.lock().unwrap().len() {
                return Err(anyhow!("UID too large for user config."));
            }

            let mut configs = self.configs.lock().unwrap();
            if configs[uidx].is_none() {
                let base_config = self.config.get().unwrap().lock().unwrap().clone();
                let mut fuse_config = FuseConfig {
                    data: base_config.data.clone(),
                    max_uid: base_config.max_uid,
                    storage_max_conc_xmit: base_config.storage_max_conc_xmit,
                };
                let kv = (key.clone(), val.to_string());
                fuse_config.atomically_update(vec![kv], true)?;
                configs[uidx] = Some(Arc::new(Mutex::new(LocalConfig {
                    config: fuse_config,
                    updated_items: vec![],
                })));
                users.insert(ui.uid);
            }

            let local_config = Arc::clone(configs[uidx].as_ref().unwrap());
            let mut lc = local_config.lock().unwrap();
            let kv = (key.clone(), val.to_string());
            lc.config.atomically_update(vec![kv.clone()], true)?;
            lc.updated_items.push(kv);

            Ok(Inode {
                id: self.config_iid(true, false, idx),
                symlink: Symlink { target: val.to_string() },
                acl: Acl {
                    owner: ui.uid,
                    group: ui.gid,
                    perm: Permission(0o400),
                },
            })
        }
    }

    pub fn lookup_config(&self, key: &str, ui: &UserInfo) -> Result<Inode> {
        let (is_sys, idx) = self.parse_key(key)?;
        self.stat_config(self.config_iid(true, is_sys, idx), ui)
    }

    pub fn stat_config(&self, iid: InodeId, ui: &UserInfo) -> Result<Inode> {
        let conf_base = InodeId::get_conf().u64();
        let kidx = (conf_base - iid.u64() - 1) as isize;
        if kidx < 0 || kidx >= (self.system_keys.len() + self.user_keys.len()) as isize {
            return Err(anyhow!("iid not a config entry"));
        }

        let is_sys = kidx < self.system_keys.len() as isize;
        let kidx = if is_sys {
            kidx as usize
        } else {
            (kidx - self.system_keys.len() as isize) as usize
        };

        let key = if is_sys {
            self.system_keys[kidx].to_string()
        } else {
            self.user_keys[kidx].to_string()
        };

        let config = if is_sys {
            self.config.get().unwrap().lock().unwrap().clone()
        } else {
            self.get_config(ui)
        };

        let value = config.find(&key).ok_or_else(|| anyhow!("Key not found"))?;

        Ok(Inode {
            id: iid,
            symlink: Symlink { target: value.to_string() },
            acl: Acl {
                owner: ui.uid,
                group: ui.gid,
                perm: Permission(if is_sys { 0o444 } else { 0o400 }),
            },
        })
    }

    pub fn get_config(&self, ui: &UserInfo) -> FuseConfig {
        let users = self.users.lock().unwrap();
        if !users.contains(&ui.uid) {
            return self.config.get().unwrap().lock().unwrap().clone();
        }

        let configs = self.configs.lock().unwrap();
        let idx = ui.uid.to_under_type();
        if let Some(Some(ref lc)) = configs.get(idx) {
            lc.lock().unwrap().config.clone()
        } else {
            self.config.get().unwrap().lock().unwrap().clone()
        }
    }

    pub fn list_config(&self, ui: &UserInfo) -> (Vec<DirEntry>, Vec<Option<MetaInode>>) {
        let mut des = vec![];
        let mut ins = vec![];

        for (i, k) in self.system_keys.iter().enumerate() {
            let full_key = format!("sys.{}", k);
            match self.lookup_config(full_key.as_str(), ui) {
                Ok(inode) => {
                    des.push(inode.clone());
                    ins.push(Some(inode));
                }
                _ => {}
            }
        }

        for (i, k) in self.user_keys.iter().enumerate() {
            let full_key = format!("usr.{}", k);
            match self.lookup_config(full_key.as_str(), ui) {
                Ok(inode) => {
                    des.push(inode.clone());
                    ins.push(Some(inode));
                }
                _ => {}
            }
        }

        (des, ins)
    }

    fn config_iid(&self, is_get: bool, is_sys: bool, kidx: usize) -> InodeId {
        let base = if is_get { InodeId::get_conf() } else { InodeId::set_conf() };
        let offset = if is_sys {
            kidx
        } else {
            kidx + self.system_keys.len()
        };
        InodeId(base.u64() - 1 - offset as u64)
    }
}