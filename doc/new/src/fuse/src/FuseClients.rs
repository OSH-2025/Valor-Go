use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, SystemTime};
use crate::FuseAppConfig::FuseAppConfig;
use crate::FuseApplication::FuseApplication;
use crate::FuseConfig::FuseConfig;
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use tokio::select;

// 假设这些类型已在其它模块实现或用占位符
// use crate::fuse_config::FuseConfig;
// use crate::user_config::UserConfig;
// use crate::io_ring::{IoRingJob, IoRingTable};
// use crate::iov_table::IovTable;
// use crate::coroutines_pool::CoroutinesPool;
// use crate::background_runner::BackgroundRunner;
// use crate::meta_client::MetaClient;
// use crate::mgmtd_client::MgmtdClientForClient;
// use crate::storage_client::StorageClient;
// use crate::rdma_buf_pool::RDMABufPool;

// 你需要根据实际项目把上面这些use补全

#[derive(Debug)]
pub struct InodeWriteBuf {
    pub buf: Vec<u8>,
    // pub memh: Option<Arc<IOBuffer>>, // 需要你自己定义 IOBuffer
    pub off: i64,
    pub len: usize,
}

#[derive(Debug)]
pub struct RcInode {
    pub inode: u64, // 占位，实际应为 Inode 类型
    pub refcount: i32,
    pub opened: AtomicU64,
    pub write_buf: Mutex<Option<InodeWriteBuf>>,
    // pub dynamic_attr: Mutex<DynamicAttr>,
    // pub extend_stripe_lock: Mutex<()>,
}

#[derive(Debug)]
pub struct FileHandle {
    pub rcinode: Arc<RcInode>,
    pub o_direct: bool,
    pub session_id: u128, // 占位，实际应为 Uuid
}

#[derive(Debug)]
pub struct DirHandle {
    pub dir_id: usize,
    pub pid: i32,
    pub iov_dir: bool,
}

#[derive(Debug)]
pub struct DirEntryVector {
    pub dir_entries: Arc<Vec<u64>>, // 占位，实际应为 DirEntry
}

#[derive(Debug)]
pub struct DirEntryInodeVector {
    pub dir_entries: Arc<Vec<u64>>, // 占位，实际应为 DirEntry
    pub inodes: Arc<Vec<Option<u64>>>, // 占位，实际应为 Option<Inode>
}

// 占位类型
pub struct Client;
pub struct MgmtdClientForClient;
pub struct StorageClient;
pub struct MetaClient;
pub struct RDMABufPool;
pub struct BackgroundRunner;
pub struct CoroutinesPool<T>(std::marker::PhantomData<T>);
pub struct FuseConfig;
pub struct UserConfig;
pub struct IovTable;
pub struct IoRingTable;
pub struct IoRingJob {
    pub id: u64,
    pub prio: usize, // 0:高 1:中 2:低
}
pub struct BoundedQueue<T>(std::marker::PhantomData<T>);

impl BoundedQueue<IoRingJob> {
    pub fn new() -> Self { Self(std::marker::PhantomData) }
}

pub struct MultiPrioQueue {
    senders: Vec<mpsc::Sender<IoRingJob>>,
    receivers: Vec<mpsc::Receiver<IoRingJob>>,
}

impl MultiPrioQueue {
    pub fn new(queue_size: usize, prio_count: usize) -> Self {
        let mut senders = Vec::new();
        let mut receivers = Vec::new();
        for _ in 0..prio_count {
            let (tx, rx) = mpsc::channel(queue_size);
            senders.push(tx);
            receivers.push(rx);
        }
        Self { senders, receivers }
    }

    pub fn sender(&self, prio: usize) -> mpsc::Sender<IoRingJob> {
        self.senders[prio].clone()
    }

    pub fn receiver(&mut self, prio: usize) -> &mut mpsc::Receiver<IoRingJob> {
        &mut self.receivers[prio]
    }
}

#[derive(Default)]
pub struct FuseClients {
    pub client: Option<Arc<Client>>,
    pub mgmtd_client: Option<Arc<MgmtdClientForClient>>,
    pub storage_client: Option<Arc<StorageClient>>,
    pub meta_client: Option<Arc<MetaClient>>,

    pub fuse_token: String,
    pub fuse_mount: String,
    pub fuse_mountpoint: PathBuf,
    pub fuse_remount_pref: Option<PathBuf>,
    pub memset_before_read: Arc<AtomicBool>,
    pub max_idle_threads: i32,
    pub max_threads: i32,
    pub enable_writeback_cache: bool,

    pub inodes: Arc<Mutex<HashMap<u64, Arc<()>>>>, // TODO: RcInode
    pub readdirplus_results: Arc<Mutex<HashMap<u64, ()>>>, // TODO: DirEntryInodeVector
    pub dir_handle: Arc<AtomicU64>,
    pub buf_pool: Option<Arc<RDMABufPool>>,
    pub jitter: Arc<Mutex<Duration>>,
    pub iovs: Option<Arc<IovTable>>,
    pub iors: Option<Arc<IoRingTable>>,
    pub iojqs: Vec<Arc<Mutex<BoundedQueue<IoRingJob>>>>,
    pub io_watches: Vec<JoinHandle<()>>,
    pub cancel_ios: Arc<AtomicBool>,
    pub user_config: Option<Arc<Mutex<UserConfig>>>,
    pub dirty_inodes: Arc<Mutex<HashSet<u64>>>,
    pub last_synced: Arc<AtomicU64>,
    pub periodic_sync_runner: Option<Arc<BackgroundRunner>>,
    pub periodic_sync_worker: Option<Arc<CoroutinesPool<u64>>>,
    pub notify_inval_exec: Option<Arc<tokio::runtime::Runtime>>,
    pub config: Option<Arc<FuseConfig>>,
    pub running: Arc<AtomicBool>,
}

impl FuseClients {
    pub fn new() -> Self {
        let mut inodes = HashMap::new();
        inodes.insert(0, Arc::new(RcInode {
            inode: 0,
            refcount: 2,
            opened: AtomicU64::new(0),
            write_buf: Mutex::new(None),
        }));
        Self {
            client: None,
            mgmtd_client: None,
            storage_client: None,
            meta_client: None,
            fuse_token: String::new(),
            fuse_mount: String::new(),
            fuse_mountpoint: PathBuf::new(),
            fuse_remount_pref: None,
            memset_before_read: Arc::new(AtomicBool::new(false)),
            max_idle_threads: 0,
            max_threads: 0,
            enable_writeback_cache: false,
            inodes: Arc::new(Mutex::new(inodes)),
            readdirplus_results: Arc::new(Mutex::new(HashMap::new())),
            dir_handle: Arc::new(AtomicU64::new(0)),
            jitter: Arc::new(Mutex::new(Duration::from_millis(1))),
            dirty_inodes: Arc::new(Mutex::new(HashSet::new())),
            last_synced: Arc::new(AtomicU64::new(0)),
            buf_pool: None,
            iovs: None,
            iors: None,
            iojqs: Vec::new(),
            io_watches: Vec::new(),
            cancel_ios: Arc::new(AtomicBool::new(false)),
            user_config: None,
            periodic_sync_runner: None,
            periodic_sync_worker: None,
            notify_inval_exec: None,
            config: None,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn init(&mut self, config: &FuseConfig, mount_point: &str, token_file: &str) -> bool {
        // 1. 配置字段赋值
        self.config = Some(Arc::new(config.clone()));
        self.fuse_mount = config.cluster_id.clone(); // 假设有 cluster_id 字段
        if self.fuse_mount.len() >= 32 {
            eprintln!("FUSE only support mount name shorter than 32 characters, but {} got.", self.fuse_mount);
            return false;
        }
        self.fuse_mountpoint = PathBuf::from(mount_point);

        // 2. remount_prefix
        self.fuse_remount_pref = config.remount_prefix.clone().map(PathBuf::from);

        // 3. token
        if let Ok(token) = std::env::var("HF3FS_FUSE_TOKEN") {
            println!("Use token from env var");
            self.fuse_token = token;
        } else {
            println!("Use token from config");
            match std::fs::read_to_string(token_file) {
                Ok(token) => {
                    self.fuse_token = token.trim().to_string();
                }
                Err(e) => {
                    eprintln!("读取 token 文件失败: {}", e);
                    return false;
                }
            }
        }

        // 4. 其它配置字段
        self.enable_writeback_cache = config.enable_writeback_cache;
        self.memset_before_read.store(config.memset_before_read, Ordering::Relaxed);
        self.max_idle_threads = config.max_idle_threads;
        let logical_cores = num_cpus::get();
        self.max_threads = if logical_cores != 0 {
            std::cmp::min(config.max_threads, (logical_cores as i32 + 1) / 2)
        } else {
            config.max_threads
        };

        // 5. buf_pool、iovs、iors、user_config 初始化
        self.buf_pool = Some(Arc::new(RDMABufPool::create(
            config.io_bufs.max_buf_size,
            config.rdma_buf_pool_size,
        )));

        let mountpoint = self.fuse_remount_pref.as_ref().unwrap_or(&self.fuse_mountpoint);
        self.iovs = Some(Arc::new(IovTable::new(
            mountpoint,
            config.iov_limit,
        )));

        self.iors = Some(Arc::new(IoRingTable::new(
            config.iov_limit,
        )));

        self.user_config = Some(Arc::new(Mutex::new(UserConfig::new(config))));

        // 6. IO 队列
        self.iojqs = vec![
            Arc::new(Mutex::new(BoundedQueue::new())),
            Arc::new(Mutex::new(BoundedQueue::new())),
            Arc::new(Mutex::new(BoundedQueue::new())),
        ];

        // 7. client 初始化
        self.client = Some(Arc::new(Client::new(config.client_config.clone())));
        if let Some(client) = &self.client {
            // client.start().await?; // 如果是异步启动
            // ctx_creator = ... // 你可以用闭包或函数指针
        }
        self.mgmtd_client = Some(Arc::new(MgmtdClientForClient::new(
            config.cluster_id.clone(),
            /* stub_factory */ None, // 你可以实现 stub_factory 逻辑
            config.mgmtd_config.clone(),
        )));
        self.storage_client = Some(Arc::new(StorageClient::new(
            /* client_id */ 0, // 你可以生成 client_id
            config.storage_config.clone(),
            self.mgmtd_client.as_ref().unwrap().clone(),
        )));
        self.meta_client = Some(Arc::new(MetaClient::new(
            /* client_id */ 0,
            config.meta_config.clone(),
            /* stub_factory */ None,
            self.mgmtd_client.as_ref().unwrap().clone(),
            self.storage_client.as_ref().unwrap().clone(),
            true, // dynStripe
        )));

        // 8. 启动 worker、watch、periodic_sync 等异步任务
        self.start_io_workers(self.max_threads as usize);
        self.start_periodic_sync();

        true
    }

    pub async fn stop(&mut self) {
    // 1. 停止并释放 notifyInvalExec 线程池
    if let Some(exec) = self.notify_inval_exec.take() {
        // 如果你有自定义线程池的 stop 方法，可以调用
        // exec.stop().await;
        drop(exec);
    }

    // 2. 释放 onFuseConfigUpdated 回调（如果有）
    // Rust 通常用 Option<Arc<...>> 或 Option<Box<...>>，直接 take/drop 即可
    // self.on_fuse_config_updated = None; // 如果有类似字段

    // 3. 请求取消所有异步任务
    self.cancel_ios.store(true, std::sync::atomic::Ordering::Relaxed);

    // 4. 停止所有 watcher/worker 线程
    for handle in self.io_watches.drain(..) {
        handle.abort(); // 立即终止任务
        // 或者：let _ = handle.await; // 等待任务自然结束
    }

    // 5. 停止并释放 periodicSyncRunner、periodicSyncWorker
    if let Some(runner) = self.periodic_sync_runner.take() {
        // 如果有 async 停止方法可以调用 runner.stop_all().await;
        drop(runner);
    }
    if let Some(worker) = self.periodic_sync_worker.take() {
        // 如果有 async 停止方法可以调用 worker.stop_and_join().await;
        drop(worker);
    }

    // 6. 停止并释放 metaClient、storageClient、mgmtdClient、client
    if let Some(meta) = self.meta_client.take() {
        // 如果有 async 停止方法可以调用 meta.stop().await;
        drop(meta);
    }
    if let Some(storage) = self.storage_client.take() {
        // 如果有 async 停止方法可以调用 storage.stop().await;
        drop(storage);
    }
    if let Some(mgmtd) = self.mgmtd_client.take() {
        // 如果有 async 停止方法可以调用 mgmtd.stop().await;
        drop(mgmtd);
    }
    if let Some(client) = self.client.take() {
        // 如果有 async 停止方法可以调用 client.stop_and_join().await;
        drop(client);
    }

    println!("FuseClients stopped.");
}

    pub fn start_io_workers(&mut self, n: usize) {
        let running = self.running.clone();
        for i in 0..n {
            let running = running.clone();
            let handle = tokio::spawn(async move {
                while running.load(Ordering::Relaxed) {
                    // TODO: IO 任务逻辑
                    println!("ioRingWorker {} running...", i);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                println!("ioRingWorker {} exit.", i);
            });
            self.io_watches.push(handle);
        }
    }

    pub fn start_periodic_sync(&mut self) {
        let running = self.running.clone();
        let handle = tokio::spawn(async move {
            while running.load(Ordering::Relaxed) {
                // TODO: 定时同步逻辑
                println!("periodicSyncScan running...");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            println!("periodicSyncScan exit.");
        });
        self.io_watches.push(handle);
    }
}

pub async fn io_ring_worker(
    mut queues: Vec<mpsc::Receiver<IoRingJob>>,
    worker_id: usize,
    running: Arc<std::sync::atomic::AtomicBool>,
) {
    while running.load(Ordering::Relaxed) {
        // 优先级从高到低依次 select
        select! {
            Some(job) = queues[0].recv() => {
                println!("[Worker {}] 高优先级处理 job {:?}", worker_id, job);
            }
            Some(job) = queues[1].recv() => {
                println!("[Worker {}] 中优先级处理 job {:?}", worker_id, job);
            }
            Some(job) = queues[2].recv() => {
                println!("[Worker {}] 低优先级处理 job {:?}", worker_id, job);
            }
            else => {
                // 所有队列都空，休眠一会
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }
    println!("[Worker {}] 退出", worker_id);
}

#[tokio::main]
async fn main() {
    let prio_count = 3;
    let mut queue = MultiPrioQueue::new(100, prio_count);
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));

    // 启动 worker
    let mut receivers = vec![];
    for i in 0..prio_count {
        receivers.push(queue.receiver(i));
    }
    let running_clone = running.clone();
    tokio::spawn(io_ring_worker(
        receivers.into_iter().map(|r| r.clone()).collect(),
        0,
        running_clone,
    ));

    // 投递不同优先级任务
    for i in 0..10 {
        let prio = i % 3;
        queue.sender(prio).send(IoRingJob { id: i, prio }).await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    running.store(false, Ordering::Relaxed);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}
