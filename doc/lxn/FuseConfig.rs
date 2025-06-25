// Rust 版本的 FuseConfig，补全配置文件加载与保存
use serde::{Serialize, Deserialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientConfig;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MgmtdClientConfig;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageClientConfig;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaClientConfig;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IoOptions;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IBDeviceConfig;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogConfig;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitorConfig;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoroutinesPoolConfig {
    pub coroutines_num: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoJobqSizes {
    pub hi: u32,
    pub lo: u32,
}
impl Default for IoJobqSizes {
    fn default() -> Self {
        Self { hi: 32, lo: 4096 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoWorkerCoros {
    pub hi: u32,
    pub lo: u32,
}
impl Default for IoWorkerCoros {
    fn default() -> Self {
        Self { hi: 8, lo: 8 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoBufs {
    pub max_buf_size: u32,
    pub max_readahead: u32,
    pub write_buf_size: u32,
}
impl Default for IoBufs {
    fn default() -> Self {
        Self {
            max_buf_size: 1024 * 1024,
            max_readahead: 256 * 1024,
            write_buf_size: 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodSync {
    pub enable: bool,
    pub interval: u64,
    pub limit: u32,
    pub flush_write_buf: bool,
    pub worker: CoroutinesPoolConfig,
}
impl Default for PeriodSync {
    fn default() -> Self {
        Self {
            enable: true,
            interval: 30,
            limit: 1000,
            flush_write_buf: true,
            worker: CoroutinesPoolConfig { coroutines_num: 4 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuseConfig {
    // 基础配置项
    pub cluster_id: String,
    pub token_file: String,
    pub mountpoint: String,
    pub allow_other: bool,
    pub ib_devices: IBDeviceConfig,
    pub log: LogConfig,
    pub monitor: MonitorConfig,
    // 热更新项
    pub enable_priority: bool,
    pub enable_interrupt: bool,
    pub attr_timeout: f64,
    pub entry_timeout: f64,
    pub negative_timeout: f64,
    pub symlink_timeout: f64,
    pub readonly: bool,
    pub memset_before_read: bool,
    pub enable_read_cache: bool,
    pub fsync_length_hint: bool,
    pub fdatasync_update_length: bool,
    pub max_idle_threads: u32,
    pub max_threads: u32,
    pub max_readahead: u32,
    pub max_background: u32,
    pub enable_writeback_cache: bool,
    pub client: ClientConfig,
    pub mgmtd: MgmtdClientConfig,
    pub storage: StorageClientConfig,
    pub meta: MetaClientConfig,
    pub remount_prefix: Option<String>,
    pub iov_limit: u32,
    pub io_jobq_size: u32,
    pub batch_io_coros: u32,
    pub rdma_buf_pool_size: u32,
    pub time_granularity: u64,
    pub check_rmrf: bool,
    pub notify_inval_threads: u32,
    pub max_uid: u32,
    pub chunk_size_limit: u32,
    pub io_jobq_sizes: IoJobqSizes,
    pub io_worker_coros: IoWorkerCoros,
    pub io_job_deq_timeout: u64,
    pub storage_io: IoOptions,
    pub submit_wait_jitter: u64,
    pub max_jobs_per_ioring: u32,
    pub io_bufs: IoBufs,
    pub flush_on_stat: bool,
    pub sync_on_stat: bool,
    pub dryrun_bench_mode: bool,
    pub periodic_sync: PeriodSync,
}

impl Default for FuseConfig {
    fn default() -> Self {
        Self {
            cluster_id: String::new(),
            token_file: String::new(),
            mountpoint: String::new(),
            allow_other: true,
            ib_devices: IBDeviceConfig::default(),
            log: LogConfig::default(),
            monitor: MonitorConfig::default(),
            enable_priority: false,
            enable_interrupt: false,
            attr_timeout: 30.0,
            entry_timeout: 30.0,
            negative_timeout: 5.0,
            symlink_timeout: 5.0,
            readonly: false,
            memset_before_read: false,
            enable_read_cache: true,
            fsync_length_hint: false,
            fdatasync_update_length: false,
            max_idle_threads: 10,
            max_threads: 256,
            max_readahead: 16 * 1024 * 1024,
            max_background: 32,
            enable_writeback_cache: false,
            client: ClientConfig::default(),
            mgmtd: MgmtdClientConfig::default(),
            storage: StorageClientConfig::default(),
            meta: MetaClientConfig::default(),
            remount_prefix: None,
            iov_limit: 1024 * 1024,
            io_jobq_size: 1024,
            batch_io_coros: 128,
            rdma_buf_pool_size: 1024,
            time_granularity: 1,
            check_rmrf: true,
            notify_inval_threads: 32,
            max_uid: 1_000_000,
            chunk_size_limit: 0,
            io_jobq_sizes: IoJobqSizes::default(),
            io_worker_coros: IoWorkerCoros::default(),
            io_job_deq_timeout: 1,
            storage_io: IoOptions::default(),
            submit_wait_jitter: 1,
            max_jobs_per_ioring: 32,
            io_bufs: IoBufs::default(),
            flush_on_stat: true,
            sync_on_stat: true,
            dryrun_bench_mode: false,
            periodic_sync: PeriodSync::default(),
        }
    }
}

impl FuseConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = fs::File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

// 单元测试示例
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fuse_config_default() {
        let config = FuseConfig::default();
        assert_eq!(config.allow_other, true);
        assert_eq!(config.max_threads, 256);
        assert_eq!(config.periodic_sync.enable, true);
    }
} 