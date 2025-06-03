// src/fuse_clients.rs
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, Semaphore};
use anyhow::{Result, Context};
use uuid::Uuid;
use log::{info, warn, error};
use async_trait::async_trait;
use dashmap::DashMap;
use parking_lot::RwLock as PLRwLock;
use std::collections::{HashSet, HashMap};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::sleep;
use futures::future::select_all;
use std::pin::Pin;
use std::future::Future;
use tokio::runtime::{Runtime, Handle};
use tokio::task::JoinHandle;

use crate::{
    AppInfo, ClientId, ClientSessionData, ClientSessionPayload, FuseConfig, 
    InodeId, NodeType, UserInfo
};

// IO相关结构
#[derive(Debug, Clone)]
pub struct IoRing {
    pub priority: usize,
    pub jobs: Arc<PLRwLock<Vec<IoRingJob>>>,
}

#[derive(Debug, Clone)]
pub struct IoRingJob {
    pub ior: Arc<IoRing>,
    pub sqe_proc_tail: usize,
    pub to_proc: usize,
    pub file_iid: InodeId,
    pub buf_id: [u8; 16],
    pub buf_off: usize,
    pub io_len: usize,
}

#[derive(Debug, Clone)]
pub struct IoArgs {
    pub file_iid: InodeId,
    pub buf_id: [u8; 16],
    pub buf_off: usize,
    pub io_len: usize,
}

#[derive(Debug, Clone)]
pub struct IoSqe {
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct RcInode {
    pub id: InodeId,
    // 其他inode相关字段
}

#[derive(Debug, Clone)]
pub struct ShmBuf {
    pub size: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ShmBufForIO {
    pub shm: Arc<ShmBuf>,
    pub offset: usize,
}

impl IoRing {
    pub fn new(priority: usize) -> Self {
        Self {
            priority,
            jobs: Arc::new(PLRwLock::new(Vec::new())),
        }
    }

    pub fn jobs_to_proc(&self, max_jobs: usize) -> Vec<IoRingJob> {
        let mut jobs = self.jobs.write();
        let count = std::cmp::min(max_jobs, jobs.len());
        jobs.drain(..count).collect()
    }
}

// 客户端会话管理
#[async_trait]
pub trait MgmtdClient: Send + Sync {
    async fn extend_client_session(&self) -> Result<()>;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn refresh_routing_info(&self, force: bool) -> Result<()>;
    fn set_client_session_payload(&self, payload: ClientSessionPayload);
    fn set_config_listener(&self, listener: Box<dyn Fn() + Send + Sync>);
}

// 存储客户端
#[async_trait]
pub trait StorageClient: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn process_io(&self, args: &[IoArgs], sqes: &[IoSqe], sqec: usize) -> Result<()>;
}

// 元数据客户端
#[async_trait]
pub trait MetaClient: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn sync_inode(&self, inode_id: InodeId) -> Result<()>;
}

// 新增trait和结构体
#[async_trait]
pub trait Client: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    fn get_runtime_handle(&self) -> Handle;
}

pub struct PeriodicSyncWorker {
    worker_count: usize,
    inode_queue: async_channel::Sender<InodeId>,
    meta_client: Arc<dyn MetaClient>,
}

impl PeriodicSyncWorker {
    pub fn new(worker_count: usize, meta_client: Arc<dyn MetaClient>) -> Self {
        let (sender, _) = async_channel::bounded(1000);
        Self {
            worker_count,
            inode_queue: sender,
            meta_client,
        }
    }

    pub async fn start(&self) -> Result<()> {
        for _ in 0..self.worker_count {
            let queue = self.inode_queue.clone();
            let meta_client = self.meta_client.clone();
            
            tokio::spawn(async move {
                while let Ok(inode_id) = queue.recv().await {
                    if let Err(e) = meta_client.sync_inode(inode_id).await {
                        error!("Failed to sync inode {}: {}", inode_id, e);
                    }
                }
            });
        }
        Ok(())
    }
}

pub struct PeriodicSyncRunner {
    interval: Duration,
    worker: Arc<PeriodicSyncWorker>,
    dirty_inodes: Arc<RwLock<HashSet<InodeId>>>,
    last_synced: InodeId,
}

impl PeriodicSyncRunner {
    pub fn new(
        interval: Duration,
        worker: Arc<PeriodicSyncWorker>,
        dirty_inodes: Arc<RwLock<HashSet<InodeId>>>,
    ) -> Self {
        Self {
            interval,
            worker,
            dirty_inodes,
            last_synced: 0,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            sleep(self.interval).await;
            self.run_sync().await?;
        }
    }

    async fn run_sync(&mut self) -> Result<()> {
        let mut dirty = HashSet::new();
        {
            let mut guard = self.dirty_inodes.write().await;
            let limit = 1000; // TODO: 从配置中获取
            
            if guard.len() <= limit {
                dirty = std::mem::take(&mut *guard);
            } else {
                let mut iter = guard.iter().skip_while(|&&id| id <= self.last_synced);
                
                while dirty.len() < limit {
                    if let Some(&inode) = iter.next() {
                        self.last_synced = inode;
                        guard.remove(&inode);
                        dirty.insert(inode);
                    } else {
                        iter = guard.iter();
                    }
                }
            }
        }

        for inode_id in dirty {
            if let Err(e) = self.worker.inode_queue.send(inode_id).await {
                error!("Failed to enqueue inode for sync: {}", e);
            }
        }

        Ok(())
    }
}

pub struct FuseClients {
    config: Arc<FuseConfig>,
    fuse_mount: String,
    fuse_mountpoint: PathBuf,
    fuse_remount_pref: Option<PathBuf>,
    fuse_token: String,
    enable_writeback_cache: bool,
    memset_before_read: bool,
    max_idle_threads: usize,
    max_threads: usize,
    
    // 客户端组件
    mgmtd_client: Option<Arc<dyn MgmtdClient>>,
    storage_client: Option<Arc<dyn StorageClient>>,
    meta_client: Option<Arc<dyn MetaClient>>,
    
    // IO相关
    io_rings: Arc<DashMap<usize, Arc<IoRing>>>,
    io_job_queues: Vec<async_channel::Sender<IoRingJob>>,
    jitter: Duration,
    
    // 同步相关
    dirty_inodes: Arc<RwLock<HashSet<InodeId>>>,
    last_synced: InodeId,
    
    // 控制标志
    running: AtomicBool,
    
    // 新增字段
    inodes: Arc<RwLock<HashMap<InodeId, Arc<RcInode>>>>,
    shm_bufs: Arc<RwLock<HashMap<Uuid, Arc<ShmBuf>>>>,
    io_semaphores: Vec<Semaphore>,
    cancel_token: Arc<AtomicBool>,

    // 补充缺失的字段
    client: Option<Arc<dyn Client>>,
    periodic_sync_worker: Option<Arc<PeriodicSyncWorker>>,
    periodic_sync_runner: Option<Arc<PeriodicSyncRunner>>,
    notify_inval_exec: Option<Runtime>,
    io_worker_handles: Vec<JoinHandle<()>>,
    watch_handles: Vec<JoinHandle<()>>,
    runtime: Option<Runtime>,
}

impl FuseClients {
    pub fn new() -> Self {
        Self {
            config: Arc::new(FuseConfig::new()),
            fuse_mount: String::new(),
            fuse_mountpoint: PathBuf::new(),
            fuse_remount_pref: None,
            fuse_token: String::new(),
            enable_writeback_cache: false,
            memset_before_read: false,
            max_idle_threads: 0,
            max_threads: 0,
            mgmtd_client: None,
            storage_client: None,
            meta_client: None,
            io_rings: Arc::new(DashMap::new()),
            io_job_queues: Vec::new(),
            jitter: Duration::from_millis(1),
            dirty_inodes: Arc::new(RwLock::new(HashSet::new())),
            last_synced: 0,
            running: AtomicBool::new(false),
            inodes: Arc::new(RwLock::new(HashMap::new())),
            shm_bufs: Arc::new(RwLock::new(HashMap::new())),
            io_semaphores: vec![
                Semaphore::new(1),
                Semaphore::new(1),
                Semaphore::new(1),
            ],
            cancel_token: Arc::new(AtomicBool::new(false)),
            client: None,
            periodic_sync_worker: None,
            periodic_sync_runner: None,
            notify_inval_exec: None,
            io_worker_handles: Vec::new(),
            watch_handles: Vec::new(),
            runtime: None,
        }
    }

    pub async fn init(
        &mut self,
        app_info: &AppInfo,
        mount_point: &str,
        token_file: &str,
        fuse_config: FuseConfig,
    ) -> Result<()> {
        self.config = Arc::new(fuse_config);
        
        // 验证挂载点名称长度
        if app_info.cluster_id.len() >= 32 {
            anyhow::bail!("FUSE only support mount name shorter than 32 characters");
        }
        self.fuse_mount = app_info.cluster_id.clone();
        
        // 设置挂载点路径
        self.fuse_mountpoint = Path::new(mount_point).to_path_buf();
        if let Some(prefix) = self.config.remount_prefix.as_ref() {
            self.fuse_remount_pref = Some(prefix.clone());
        }

        // 获取token
        if let Ok(token) = std::env::var("HF3FS_FUSE_TOKEN") {
            info!("Use token from env var");
            self.fuse_token = token;
        } else {
            info!("Use token from config");
            self.fuse_token = tokio::fs::read_to_string(token_file)
                .await
                .context("Failed to read token file")?;
        }

        // 设置配置参数
        self.enable_writeback_cache = self.config.enable_writeback_cache;
        self.memset_before_read = self.config.memset_before_read;
        self.max_idle_threads = self.config.max_idle_threads;
        
        // 计算最大线程数
        let logical_cores = thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);
        self.max_threads = std::cmp::min(
            self.config.max_threads,
            (logical_cores + 1) / 2
        );

        // 初始化IO组件
        self.init_io_components().await?;

        // 初始化客户端组件
        self.init_clients(app_info).await?;

        self.running.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn init_io_components(&mut self) -> Result<()> {
        // 初始化IO环
        for priority in 0..3 {
            self.io_rings.insert(priority, Arc::new(IoRing::new(priority)));
        }

        // 初始化作业队列
        self.io_job_queues = vec![
            async_channel::bounded(self.config.io_jobq_sizes.hi).0,
            async_channel::bounded(self.config.io_jobq_size).0,
            async_channel::bounded(self.config.io_jobq_sizes.lo).0,
        ];

        self.jitter = self.config.submit_wait_jitter;

        // 启动IO工作线程
        for i in 0..self.config.batch_io_coros {
            let client = self.clone();
            let handle = tokio::spawn(async move {
                client.io_ring_worker(i, self.config.batch_io_coros).await;
            });
            self.io_worker_handles.push(handle);
        }

        // 启动IO监视器
        for i in 0..3 {
            let client = self.clone();
            let handle = tokio::spawn(async move {
                client.watch(i).await;
            });
            self.watch_handles.push(handle);
        }

        Ok(())
    }

    async fn io_ring_worker(&self, worker_id: usize, total_workers: usize) {
        let mut check_higher = true;

        while !self.cancel_token.load(Ordering::SeqCst) {
            let hi_ths = self.config.io_worker_coros.hi;
            let lo_ths = self.config.io_worker_coros.lo;
            let prio = if worker_id < hi_ths {
                0
            } else if worker_id < (total_workers - lo_ths) {
                1
            } else {
                2
            };

            let job = if !self.config.enable_priority {
                self.io_job_queues[prio].recv().await.unwrap()
            } else {
                let mut job = None;
                let mut got_job = false;

                while !got_job {
                    if check_higher {
                        for nprio in 0..prio {
                            if let Ok(j) = self.io_job_queues[nprio].try_recv() {
                                check_higher = false;
                                got_job = true;
                                job = Some(j);
                                break;
                            }
                        }
                    }

                    if !got_job {
                        for nprio in (if check_higher { 0 } else { prio }..=prio).rev() {
                            match tokio::time::timeout(
                                self.config.io_job_deq_timeout,
                                self.io_job_queues[nprio].recv()
                            ).await {
                                Ok(Ok(j)) => {
                                    if !check_higher && nprio == prio {
                                        check_higher = true;
                                    }
                                    got_job = true;
                                    job = Some(j);
                                    break;
                                }
                                _ => continue,
                            }
                        }
                    }
                }
                job.unwrap()
            };

            // 处理IO作业
            if let Some(storage_client) = &self.storage_client {
                if let Err(e) = self.process_io_job(storage_client, &job).await {
                    error!("Failed to process IO job: {}", e);
                }
            }
        }
    }

    async fn watch(&self, priority: usize) {
        while !self.cancel_token.load(Ordering::SeqCst) {
            let jitter = self.jitter;
            sleep(jitter).await;

            let mut got_jobs = false;
            loop {
                got_jobs = false;
                for i in 0..self.io_rings.len() {
                    if let Some(ring) = self.io_rings.get(&i) {
                        if ring.priority == priority {
                            let jobs = ring.jobs_to_proc(self.config.max_jobs_per_ioring);
                            for job in jobs {
                                if let Err(e) = self.io_job_queues[priority].send(job).await {
                                    error!("Failed to enqueue job: {}", e);
                                }
                                got_jobs = true;
                            }
                        }
                    }
                }
                if !got_jobs {
                    break;
                }
            }
        }
    }

    async fn process_io_job(&self, storage_client: &dyn StorageClient, job: &IoRingJob) -> Result<()> {
        let mut inodes = Vec::new();
        let mut bufs = Vec::new();

        // 查找文件
        {
            let inodes_guard = self.inodes.read().await;
            let mut last_iid = 0u64;

            for i in 0..job.to_proc {
                let idn = job.file_iid;
                if i > 0 && idn == last_iid {
                    inodes.push(inodes.last().unwrap().clone());
                    continue;
                }

                last_iid = idn;
                if let Some(inode) = inodes_guard.get(&idn) {
                    inodes.push(inode.clone());
                } else {
                    inodes.push(Arc::new(RcInode { id: idn }));
                }
            }
        }

        // 查找缓冲区
        {
            let bufs_guard = self.shm_bufs.read().await;
            let mut last_id = Uuid::nil();
            let mut last_shm = None;

            for i in 0..job.to_proc {
                let id = Uuid::from_bytes(job.buf_id);

                let shm = if i > 0 && id == last_id {
                    last_shm.clone()
                } else {
                    if let Some(shm) = bufs_guard.get(&id) {
                        last_id = id;
                        last_shm = Some(shm.clone());
                        Some(shm.clone())
                    } else {
                        None
                    }
                };

                if let Some(shm) = shm {
                    if shm.size < job.buf_off + job.io_len {
                        return Err(anyhow::anyhow!("Invalid buffer offset or IO length"));
                    }
                    bufs.push(ShmBufForIO {
                        shm,
                        offset: job.buf_off,
                    });
                } else {
                    return Err(anyhow::anyhow!("Buffer not found"));
                }
            }
        }

        // 处理IO
        let args = vec![IoArgs {
            file_iid: job.file_iid,
            buf_id: job.buf_id,
            buf_off: job.buf_off,
            io_len: job.io_len,
        }];

        let sqes = vec![IoSqe { index: 0 }];

        storage_client.process_io(&args, &sqes, job.to_proc).await
    }

    async fn establish_client_session(&self) -> Result<()> {
        if let Some(mgmtd_client) = &self.mgmtd_client {
            let mut retry_interval = Duration::from_millis(10);
            let max_retry_interval = Duration::from_millis(1000);
            
            for i in 0..40 {
                match mgmtd_client.extend_client_session().await {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        error!("Try to establish client session failed: {}\nretryCount: {}", e, i);
                        sleep(retry_interval).await;
                        retry_interval = std::cmp::min(retry_interval * 2, max_retry_interval);
                    }
                }
            }
            anyhow::bail!("Failed to establish client session after 40 retries");
        }
        Ok(())
    }

    async fn init_clients(&mut self, app_info: &AppInfo) -> Result<()> {
        // 获取主机名
        let physical_hostname = tokio::process::Command::new("hostname")
            .arg("-f")
            .output()
            .await
            .context("Failed to get physical hostname")?;
        let physical_hostname = String::from_utf8_lossy(&physical_hostname.stdout).trim().to_string();

        let container_hostname = tokio::process::Command::new("hostname")
            .output()
            .await
            .context("Failed to get container hostname")?;
        let container_hostname = String::from_utf8_lossy(&container_hostname.stdout).trim().to_string();

        // 创建客户端ID
        let client_id = ClientId::random(physical_hostname.clone());

        // 创建客户端会话数据
        let session_data = ClientSessionData::create(
            physical_hostname.clone(),
            format!("fuse: {}", container_hostname),
            app_info.service_groups.clone(),
            app_info.release_version.clone(),
        );

        // 创建会话负载
        let session_payload = ClientSessionPayload {
            client_id: client_id.uuid.to_string(),
            node_type: NodeType::FUSE,
            session_data,
            user_info: UserInfo {}, // TODO: 使用真实的用户信息
        };

        // 初始化运行时
        self.runtime = Some(Runtime::new()?);

        // 初始化管理客户端
        if self.mgmtd_client.is_none() {
            // TODO: 实现具体的MgmtdClient创建
            // 这里需要根据具体的客户端实现来完成
            // 原始C++代码使用了RealStubFactory和MgmtdServiceStub
        }

        if let Some(mgmtd_client) = &self.mgmtd_client {
            mgmtd_client.set_client_session_payload(session_payload);
            mgmtd_client.set_config_listener(Box::new(|| {
                // TODO: 实现配置更新回调
            }));

            // 启动管理客户端
            mgmtd_client.start().await?;
            mgmtd_client.refresh_routing_info(false).await?;
        }

        // 建立客户端会话
        self.establish_client_session().await?;

        // 初始化存储客户端
        if self.storage_client.is_none() {
            // TODO: 实现具体的StorageClient创建
            // 原始C++代码使用了StorageClient::create
        }

        // 初始化元数据客户端
        if self.meta_client.is_none() {
            // TODO: 实现具体的MetaClient创建
            // 原始C++代码使用了MetaClient的构造函数
        }

        if let Some(meta_client) = &self.meta_client {
            meta_client.start().await?;
        }

        // 初始化定期同步工作器
        if let Some(meta_client) = &self.meta_client {
            let worker = Arc::new(PeriodicSyncWorker::new(
                self.config.periodic_sync.worker,
                meta_client.clone(),
            ));
            worker.start().await?;
            self.periodic_sync_worker = Some(worker.clone());

            // 初始化定期同步运行器
            let runner = Arc::new(PeriodicSyncRunner::new(
                self.config.periodic_sync.interval,
                worker,
                self.dirty_inodes.clone(),
            ));
            self.periodic_sync_runner = Some(runner);
        }

        // 初始化通知失效执行器
        self.notify_inval_exec = Some(Runtime::new()?);

        Ok(())
    }

    pub async fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        // 停止所有工作线程
        for handle in self.io_worker_handles.drain(..) {
            let _ = handle.abort();
        }

        for handle in self.watch_handles.drain(..) {
            let _ = handle.abort();
        }

        // 停止所有客户端
        if let Some(meta_client) = self.meta_client.take() {
            if let Err(e) = meta_client.stop().await {
                error!("Failed to stop meta client: {}", e);
            }
        }

        if let Some(storage_client) = self.storage_client.take() {
            if let Err(e) = storage_client.stop().await {
                error!("Failed to stop storage client: {}", e);
            }
        }

        if let Some(mgmtd_client) = self.mgmtd_client.take() {
            if let Err(e) = mgmtd_client.stop().await {
                error!("Failed to stop mgmtd client: {}", e);
            }
        }

        if let Some(client) = self.client.take() {
            if let Err(e) = client.stop().await {
                error!("Failed to stop client: {}", e);
            }
        }

        // 关闭运行时
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_timeout(Duration::from_secs(1));
        }

        self.running.store(false, Ordering::SeqCst);
    }

    pub async fn periodic_sync_scan(&self) -> Result<()> {
        if !self.config.periodic_sync.enable || self.config.readonly {
            return Ok(());
        }

        info!("periodicSyncScan run");
        let mut dirty = HashSet::new();
        
        {
            let mut guard = self.dirty_inodes.write().await;
            let limit = self.config.periodic_sync.limit;
            
            if guard.len() <= limit {
                dirty = std::mem::take(&mut *guard);
            } else {
                warn!("dirty inodes {} > limit {}", guard.len(), limit);
                let mut iter = guard.iter().skip_while(|&&id| id <= self.last_synced);
                
                while dirty.len() < limit {
                    if let Some(&inode) = iter.next() {
                        self.last_synced = inode;
                        guard.remove(&inode);
                        dirty.insert(inode);
                    } else {
                        iter = guard.iter();
                    }
                }
            }
        }

        // 同步处理
        if let Some(meta_client) = &self.meta_client {
            for inode_id in dirty {
                if let Err(e) = meta_client.sync_inode(inode_id).await {
                    error!("Failed to sync inode {}: {}", inode_id, e);
                }
            }
        }

        Ok(())
    }
}

impl Drop for FuseClients {
    fn drop(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            self.cancel_token.store(true, Ordering::SeqCst);
            let _ = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(self.stop());
        }
    }
} 