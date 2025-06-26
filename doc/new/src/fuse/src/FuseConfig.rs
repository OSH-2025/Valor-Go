use crate::FuseAppConfig::{ConfigBase, KeyValue};
use std::collections::HashMap;
use std::time::Duration;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct FuseConfig {
    pub cluster_id: String,
    pub token_file: String,
    pub mountpoint: String,
    pub allow_other: bool,
    // pub ib_devices: IBDeviceConfig, // 占位
    // pub log: LogConfig, // 占位
    // pub monitor: MonitorConfig, // 占位
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
    pub max_idle_threads: i32,
    pub max_threads: i32,
    pub max_readahead: usize,
    pub max_background: i32,
    pub enable_writeback_cache: bool,
    // pub client: ClientConfig, // 占位
    // pub mgmtd: MgmtdClientConfig, // 占位
    // pub storage: StorageClientConfig, // 占位
    // pub meta: MetaClientConfig, // 占位
    pub remount_prefix: Option<String>,
    pub iov_limit: usize,
    pub io_jobq_size: usize,
    pub batch_io_coros: usize,
    pub rdma_buf_pool_size: usize,
    pub time_granularity: Duration,
    pub check_rmrf: bool,
    pub notify_inval_threads: i32,
    pub max_uid: u64,
    pub chunk_size_limit: usize,
    pub io_jobq_sizes: HashMap<String, usize>,
    pub io_worker_coros: HashMap<String, usize>,
    pub io_job_deq_timeout: Duration,
    // pub storage_io: StorageIoOptions, // 占位
    pub submit_wait_jitter: Duration,
    pub max_jobs_per_ioring: usize,
    pub io_bufs: HashMap<String, usize>,
    pub flush_on_stat: bool,
    pub sync_on_stat: bool,
    pub dryrun_bench_mode: bool,
    pub periodic_sync: PeriodSync,
}

#[derive(Debug, Clone)]
pub struct PeriodSync {
    pub enable: bool,
    pub interval: Duration,
    pub limit: u32,
    pub flush_write_buf: bool,
    // pub worker: CoroutinesPoolConfig, // 占位
}

impl Default for PeriodSync {
    fn default() -> Self {
        Self {
            enable: true,
            interval: Duration::from_secs(30),
            limit: 1000,
            flush_write_buf: true,
            // worker: Default::default(),
        }
    }
}

impl Default for FuseConfig {
    fn default() -> Self {
        let mut io_jobq_sizes = HashMap::new();
        io_jobq_sizes.insert("hi".to_string(), 32);
        io_jobq_sizes.insert("lo".to_string(), 4096);
        let mut io_worker_coros = HashMap::new();
        io_worker_coros.insert("hi".to_string(), 8);
        io_worker_coros.insert("lo".to_string(), 8);
        let mut io_bufs = HashMap::new();
        io_bufs.insert("max_buf_size".to_string(), 1024 * 1024);
        io_bufs.insert("max_readahead".to_string(), 256 * 1024);
        io_bufs.insert("write_buf_size".to_string(), 1024 * 1024);
        Self {
            cluster_id: String::new(),
            token_file: String::new(),
            mountpoint: String::new(),
            allow_other: true,
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
            remount_prefix: None,
            iov_limit: 1024 * 1024,
            io_jobq_size: 1024,
            batch_io_coros: 128,
            rdma_buf_pool_size: 1024,
            time_granularity: Duration::from_secs(1),
            check_rmrf: true,
            notify_inval_threads: 32,
            max_uid: 1_000_000,
            chunk_size_limit: 0,
            io_jobq_sizes,
            io_worker_coros,
            io_job_deq_timeout: Duration::from_millis(1),
            submit_wait_jitter: Duration::from_millis(1),
            max_jobs_per_ioring: 32,
            io_bufs,
            flush_on_stat: true,
            sync_on_stat: true,
            dryrun_bench_mode: false,
            periodic_sync: PeriodSync::default(),
        }
    }
}

impl ConfigBase for FuseConfig {
    fn init(&mut self, file_path: &str, dump: bool, updates: Vec<KeyValue>) -> Result<(), String> {
        // 1. 读取文件
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        // 2. 反序列化
        let mut loaded: FuseConfig = toml::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;

        // 3. 应用 updates
        for kv in updates {
            match kv.key.as_str() {
                "token" => loaded.token_file = kv.value,
                "mountpoint" => loaded.mountpoint = kv.value,
                // 你可以继续补充其它字段
                _ => {}
            }
        }

        // 4. dump
        if dump {
            println!("{:#?}", loaded); // 或 serde_json::to_string_pretty(&loaded)
            std::process::exit(0);
        }

        // 5. 覆盖 self
        *self = loaded;
        Ok(())
    }
} 
