// io_ring.rs

use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, Mutex, atomic::{AtomicI32, AtomicU64, Ordering}},
    time::{Duration, Instant},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    mem,
    ptr,
};

// 类型别名和占位类型
type Uuid = [u8; 16];
type Path = String;
type Result<T> = std::result::Result<T, String>;
type SteadyTime = Instant;

// 需要实现的类型
struct RcInode;
struct UserConfig;
struct ShmBuf;
struct IoOptions;
struct StorageClient;
struct IorAttrs;
struct MetaUserInfo;
struct ShmBufForIO;

// 监控相关结构
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
        // 实现监控记录逻辑
    }
}

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
        // 实现分布记录逻辑
    }
}

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

    fn add_sample(&self, value: u64, tags: Vec<(String, String)>) {
        // 实现计数记录逻辑
    }
}

#[derive(Clone)]
pub struct IoArgs {
    pub buf_id: Uuid,
    pub buf_off: usize,
    pub file_iid: u64,
    pub file_off: usize,
    pub io_len: u64,
    pub userdata: *const u8,
}

#[derive(Clone)]
pub struct IoSqe {
    pub index: i32,
    pub userdata: *const u8,
}

#[derive(Clone)]
pub struct IoCqe {
    pub index: i32,
    pub reserved: i32,
    pub result: i64,
    pub userdata: *const u8,
}

pub struct IoRingJob {
    pub ior: Arc<IoRing>,
    pub sqe_proc_tail: i32,
    pub to_proc: i32,
}

pub struct IoRing {
    pub name: String,
    pub mount_name: String,
    pub entries: i32,
    pub io_depth: i32,
    pub priority: i32,
    pub timeout: Duration,

    sqe_head: AtomicI32,
    sqe_tail: AtomicI32,
    cqe_head: AtomicI32,
    cqe_tail: AtomicI32,

    pub ring_section: Vec<IoArgs>,
    pub cqe_section: Vec<IoCqe>,
    pub sqe_section: Vec<IoSqe>,

    pub slots: usize,

    shm: Arc<ShmBuf>,
    user_info: MetaUserInfo,
    for_read: bool,
    flags: u64,
    cqe_mtx: Mutex<()>,
    sqe_proc_tail: i32,
    processing: i32,
    sqe_proc_tails: VecDeque<i32>,
    sqe_done_tails: HashSet<i32>,
    last_check: Option<SteadyTime>,
}

impl IoRing {
    pub fn ring_marker_size() -> usize {
        4
    }

    pub fn io_ring_entries(buf_size: usize) -> i32 {
        let n = Self::ring_marker_size();
        ((buf_size - 4096 - n * 4 - mem::size_of::<usize>()) / 
            (mem::size_of::<IoArgs>() + mem::size_of::<IoCqe>() + mem::size_of::<IoSqe>())) as i32 - 1
    }

    pub fn bytes_required(entries: i32) -> usize {
        let n = Self::ring_marker_size();
        n * 4 + mem::size_of::<usize>() + 
        (mem::size_of::<IoArgs>() + mem::size_of::<IoCqe>() + mem::size_of::<IoSqe>()) * (entries as usize + 1) + 4096
    }

    pub fn new(
        shm: Arc<ShmBuf>,
        name: String,
        user_info: MetaUserInfo,
        for_read: bool,
        buf: &mut [u8],
        size: usize,
        io_depth: i32,
        priority: i32,
        timeout: Duration,
        flags: u64,
        owner: bool,
    ) -> Self {
        let entries = Self::io_ring_entries(size) + 1;
        let n = Self::ring_marker_size();

        let mut ring = Self {
            name,
            mount_name: String::new(),
            entries,
            io_depth,
            priority,
            timeout,
            sqe_head: AtomicI32::new(0),
            sqe_tail: AtomicI32::new(0),
            cqe_head: AtomicI32::new(0),
            cqe_tail: AtomicI32::new(0),
            ring_section: Vec::with_capacity(entries as usize),
            cqe_section: Vec::with_capacity(entries as usize),
            sqe_section: Vec::with_capacity(entries as usize),
            slots: (entries - 1) as usize,
            shm,
            user_info,
            for_read,
            flags,
            cqe_mtx: Mutex::new(()),
            sqe_proc_tail: 0,
            processing: 0,
            sqe_proc_tails: VecDeque::new(),
            sqe_done_tails: HashSet::new(),
            last_check: None,
        };

        // 初始化内存布局
        let buf_ptr = buf.as_mut_ptr();
        unsafe {
            ring.sqe_head = AtomicI32::new(*(buf_ptr as *const i32));
            ring.sqe_tail = AtomicI32::new(*(buf_ptr.add(n) as *const i32));
            ring.cqe_head = AtomicI32::new(*(buf_ptr.add(n * 2) as *const i32));
            ring.cqe_tail = AtomicI32::new(*(buf_ptr.add(n * 3) as *const i32));
            
            let ring_section_ptr = buf_ptr.add(n * 4) as *mut IoArgs;
            let cqe_section_ptr = ring_section_ptr.add(entries as usize) as *mut IoCqe;
            let sqe_section_ptr = cqe_section_ptr.add(entries as usize) as *mut IoSqe;

            ring.ring_section = Vec::from_raw_parts(ring_section_ptr, entries as usize, entries as usize);
            ring.cqe_section = Vec::from_raw_parts(cqe_section_ptr, entries as usize, entries as usize);
            ring.sqe_section = Vec::from_raw_parts(sqe_section_ptr, entries as usize, entries as usize);
        }

        ring
    }

    pub fn cqe_count(&self) -> i32 {
        (self.cqe_head.load(Ordering::SeqCst) + self.entries - self.cqe_tail.load(Ordering::SeqCst)) % self.entries
    }

    pub fn add_sqe(&self, idx: i32, userdata: *const u8) -> bool {
        let h = self.sqe_head.load(Ordering::SeqCst);
        if (h + 1) % self.entries == self.sqe_tail.load(Ordering::SeqCst) {
            return false;
        }

        self.sqe_section[h as usize] = IoSqe { index: idx, userdata };
        self.sqe_head.store((h + 1) % self.entries, Ordering::SeqCst);
        true
    }

    pub fn add_cqe(&self, idx: i32, res: i64, userdata: *const u8) -> bool {
        let h = self.cqe_head.load(Ordering::SeqCst);
        if (h + 1) % self.entries == self.cqe_tail.load(Ordering::SeqCst) {
            return false;
        }

        self.cqe_section[h as usize] = IoCqe {
            index: idx,
            reserved: 0,
            result: res,
            userdata,
        };
        self.cqe_head.store((h + 1) % self.entries, Ordering::SeqCst);
        true
    }

    pub fn jobs_to_proc(&self, max_jobs: i32) -> Vec<IoRingJob> {
        let mut jobs = Vec::new();
        let _lock = self.cqe_mtx.lock().unwrap();
        
        let spt = self.sqe_proc_tail;
        let sqes = self.sqe_count();
        
        let cqe_avail = self.entries - 1 - self.processing - self.cqe_count();
        
        let mut current_spt = spt;
        let mut remaining_sqes = sqes;
        
        while remaining_sqes > 0 && (jobs.len() as i32) < max_jobs {
            let to_proc = if self.io_depth > 0 {
                let depth = self.io_depth;
                if depth > remaining_sqes || depth > cqe_avail {
                    break;
                }
                depth
            } else {
                let mut to_proc = std::cmp::min(remaining_sqes, cqe_avail);
                if self.io_depth < 0 {
                    let iod = -self.io_depth;
                    if to_proc > iod {
                        to_proc = iod;
                    } else if to_proc < iod && !self.timeout.is_zero() {
                        let now = Instant::now();
                        if let Some(last_check) = self.last_check {
                            if last_check + self.timeout > now {
                                break;
                            }
                        } else {
                            self.last_check = Some(now);
                            break;
                        }
                        self.last_check = None;
                    }
                }
                to_proc
            };

            if jobs.is_empty() {
                jobs.reserve(if self.io_depth != 0 {
                    std::cmp::min(max_jobs, remaining_sqes / self.io_depth.abs() + 1)
                } else {
                    1
                } as usize);
            }

            jobs.push(IoRingJob {
                ior: Arc::new(self.clone()),
                sqe_proc_tail: current_spt,
                to_proc,
            });

            current_spt = (current_spt + to_proc) % self.entries;
            self.sqe_proc_tails.push_back(current_spt);
            self.processing += to_proc;
            remaining_sqes -= to_proc;
        }

        self.sqe_proc_tail = current_spt;
        jobs
    }

    pub async fn process(
        &self,
        spt: i32,
        to_proc: i32,
        storage_client: &mut StorageClient,
        storage_io: &IoOptions,
        user_config: &mut UserConfig,
        lookup_files: impl Fn(&mut Vec<Arc<RcInode>>, &[IoArgs], &[IoSqe], i32),
        lookup_bufs: impl Fn(&mut Vec<Result<Arc<ShmBufForIO>>>, &[IoArgs], &[IoSqe], i32),
    ) {
        // 监控记录器
        let overall_latency = LatencyRecorder::new(
            "usrbio.piov.overall",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let prepare_latency = LatencyRecorder::new(
            "usrbio.piov.prepare",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let submit_latency = LatencyRecorder::new(
            "usrbio.piov.submit",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let complete_latency = LatencyRecorder::new(
            "usrbio.piov.complete",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let io_size_dist = DistributionRecorder::new(
            "usrbio.piov.io_size",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let io_depth_dist = DistributionRecorder::new(
            "usrbio.piov.io_depth",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let total_bytes_dist = DistributionRecorder::new(
            "usrbio.piov.total_bytes",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let distinct_files_dist = DistributionRecorder::new(
            "usrbio.piov.distinct_files",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let distinct_bufs_dist = DistributionRecorder::new(
            "usrbio.piov.distinct_bufs",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let bw_count = CountRecorder::new(
            "usrbio.piov.bw",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );

        let start = Instant::now();
        let overall_start = start;
        let io_type = if self.for_read { "read" } else { "write" };
        let uids = self.user_info.uid.to_string();

        let config = user_config.get_config(&self.user_info);

        let mut res = if !self.for_read && config.readonly() {
            vec![-StatusCode::kReadOnlyMode as i64; to_proc as usize]
        } else {
            vec![0; to_proc as usize]
        };

        if res[0] >= 0 {
            let mut iod = 0;
            let mut total_bytes = 0;
            let mut distinct_files = HashSet::new();
            let mut distinct_bufs = HashSet::new();

            let mut inodes = Vec::with_capacity(to_proc as usize);
            lookup_files(&mut inodes, &self.ring_section, &self.sqe_section, to_proc);

            let mut bufs = Vec::with_capacity(to_proc as usize);
            lookup_bufs(&mut bufs, &self.ring_section, &self.sqe_section, to_proc);

            // 处理每个IO请求
            for i in 0..to_proc {
                let idx = (spt + i) % self.entries;
                let sqe = &self.sqe_section[idx as usize];
                let args = &self.ring_section[sqe.index as usize];

                iod += 1;
                total_bytes += args.io_len;
                distinct_files.insert(args.file_iid);
                distinct_bufs.insert(args.buf_id);

                io_size_dist.add_sample(args.io_len, vec![
                    ("io".to_string(), io_type.to_string()),
                    ("uid".to_string(), uids.clone()),
                ]);

                if inodes[i as usize].is_none() {
                    res[i as usize] = -MetaCode::kNotFile as i64;
                    continue;
                }

                if let Err(e) = &bufs[i as usize] {
                    res[i as usize] = -e.code() as i64;
                    continue;
                }

                // 处理内存句柄
                let memh = bufs[i as usize].as_ref().unwrap().memh(args.io_len).await;
                if let Err(e) = memh {
                    res[i as usize] = -e.code() as i64;
                    continue;
                }

                let memh = memh.unwrap();
                if bufs[i as usize].as_ref().unwrap().ptr().is_none() || memh.is_none() {
                    res[i as usize] = -ClientAgentCode::kIovShmFail as i64;
                    continue;
                }

                // 处理写入操作
                if !self.for_read {
                    let begin_write = inodes[i as usize].as_ref().unwrap()
                        .begin_write(&self.user_info, storage_client, args.file_off, args.io_len)
                        .await;
                    
                    if let Err(e) = begin_write {
                        res[i as usize] = -e.code() as i64;
                        continue;
                    }
                }

                // 执行IO操作
                let io_result = if self.for_read {
                    storage_client.read(
                        &self.user_info,
                        inodes[i as usize].as_ref().unwrap().inode,
                        args.file_off,
                        args.io_len,
                        bufs[i as usize].as_ref().unwrap().ptr().unwrap(),
                        memh.unwrap(),
                    ).await
                } else {
                    storage_client.write(
                        &self.user_info,
                        inodes[i as usize].as_ref().unwrap().inode,
                        args.file_off,
                        args.io_len,
                        bufs[i as usize].as_ref().unwrap().ptr().unwrap(),
                        memh.unwrap(),
                    ).await
                };

                if let Err(e) = io_result {
                    res[i as usize] = -e.code() as i64;
                }
            }

            // 记录监控数据
            let now = Instant::now();
            prepare_latency.add_sample(now - start, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);

            io_depth_dist.add_sample(iod, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
            total_bytes_dist.add_sample(total_bytes, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
            distinct_files_dist.add_sample(distinct_files.len() as u64, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
            distinct_bufs_dist.add_sample(distinct_bufs.len() as u64, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
        }

        // 更新完成状态
        let new_spt = (spt + to_proc) % self.entries;
        let mut sqes = Vec::with_capacity(to_proc as usize);
        for i in 0..to_proc {
            sqes.push(self.sqe_section[(spt + i) % self.entries as usize].clone());
        }

        {
            let _lock = self.cqe_mtx.lock().unwrap();
            
            if self.sqe_proc_tails.is_empty() {
                panic!("bug?! sqe_proc_tails is empty");
            }

            if self.sqe_proc_tails[0] != new_spt {
                self.sqe_done_tails.insert(new_spt);
            } else {
                self.sqe_tail.store(new_spt, Ordering::SeqCst);
                self.sqe_proc_tails.pop_front();
                
                while !self.sqe_done_tails.is_empty() {
                    if self.sqe_proc_tails.is_empty() {
                        panic!("bug?! sqe_proc_tails is empty");
                    }
                    
                    let first = self.sqe_proc_tails[0];
                    if let Some(_) = self.sqe_done_tails.take(&first) {
                        self.sqe_tail.store(first, Ordering::SeqCst);
                        self.sqe_proc_tails.pop_front();
                    } else {
                        break;
                    }
                }
            }

            // 添加完成事件
            for i in 0..to_proc {
                let sqe = &sqes[i as usize];
                let r = res[i as usize];
                if !self.add_cqe(sqe.index, r, sqe.userdata) {
                    panic!("failed to add cqe");
                }
            }

            self.processing -= to_proc;
        }

        // 计算带宽
        let mut done_bytes = 0;
        for r in res {
            if r > 0 {
                done_bytes += r as u64;
            }
        }
        bw_count.add_sample(done_bytes, vec![
            ("io".to_string(), io_type.to_string()),
            ("uid".to_string(), uids.clone()),
        ]);

        // 记录延迟
        let now = Instant::now();
        complete_latency.add_sample(now - start, vec![
            ("io".to_string(), io_type.to_string()),
            ("uid".to_string(), uids.clone()),
        ]);
        overall_latency.add_sample(now - overall_start, vec![
            ("io".to_string(), io_type.to_string()),
            ("uid".to_string(), uids.clone()),
        ]);
    }
}

// IoRingTable 实现
pub struct IoRingTable {
    sems: Vec<Arc<tokio::sync::Semaphore>>,
    io_rings: Arc<Mutex<Vec<Option<Arc<IoRing>>>>>,
}

impl IoRingTable {
    pub fn new(cap: usize) -> Self {
        let mut sems = Vec::new();
        for prio in 0..=2 {
            let sem = Arc::new(tokio::sync::Semaphore::new(1));
            sems.push(sem);
        }

        Self {
            sems,
            io_rings: Arc::new(Mutex::new(vec![None; cap])),
        }
    }

    pub fn add_io_ring(
        &self,
        mount_name: Path,
        shm: Arc<ShmBuf>,
        name: String,
        user_info: MetaUserInfo,
        for_read: bool,
        buf: &mut [u8],
        size: usize,
        io_depth: i32,
        attrs: IorAttrs,
    ) -> Result<i32> {
        let mut io_rings = self.io_rings.lock().unwrap();
        
        // 查找空闲槽位
        let idx = io_rings.iter().position(|x| x.is_none())
            .ok_or_else(|| "too many io rings".to_string())?;

        let ior = Arc::new(IoRing::new(
            shm,
            name,
            user_info,
            for_read,
            buf,
            size,
            io_depth,
            attrs.priority,
            attrs.timeout,
            attrs.flags,
            true,
        ));
        ior.mount_name = mount_name.to_string();
        
        io_rings[idx] = Some(ior);
        Ok(idx as i32)
    }

    pub fn remove_io_ring(&self, idx: i32) {
        let mut io_rings = self.io_rings.lock().unwrap();
        if idx >= 0 && (idx as usize) < io_rings.len() {
            io_rings[idx as usize] = None;
        }
    }
}// io_ring.rs

use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, Mutex, atomic::{AtomicI32, AtomicU64, Ordering}},
    time::{Duration, Instant},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    mem,
    ptr,
};

// 类型别名和占位类型
type Uuid = [u8; 16];
type Path = String;
type Result<T> = std::result::Result<T, String>;
type SteadyTime = Instant;

// 需要实现的类型
struct RcInode;
struct UserConfig;
struct ShmBuf;
struct IoOptions;
struct StorageClient;
struct IorAttrs;
struct MetaUserInfo;
struct ShmBufForIO;

// 监控相关结构
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
        // 实现监控记录逻辑
    }
}

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
        // 实现分布记录逻辑
    }
}

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

    fn add_sample(&self, value: u64, tags: Vec<(String, String)>) {
        // 实现计数记录逻辑
    }
}

#[derive(Clone)]
pub struct IoArgs {
    pub buf_id: Uuid,
    pub buf_off: usize,
    pub file_iid: u64,
    pub file_off: usize,
    pub io_len: u64,
    pub userdata: *const u8,
}

#[derive(Clone)]
pub struct IoSqe {
    pub index: i32,
    pub userdata: *const u8,
}

#[derive(Clone)]
pub struct IoCqe {
    pub index: i32,
    pub reserved: i32,
    pub result: i64,
    pub userdata: *const u8,
}

pub struct IoRingJob {
    pub ior: Arc<IoRing>,
    pub sqe_proc_tail: i32,
    pub to_proc: i32,
}

pub struct IoRing {
    pub name: String,
    pub mount_name: String,
    pub entries: i32,
    pub io_depth: i32,
    pub priority: i32,
    pub timeout: Duration,

    sqe_head: AtomicI32,
    sqe_tail: AtomicI32,
    cqe_head: AtomicI32,
    cqe_tail: AtomicI32,

    pub ring_section: Vec<IoArgs>,
    pub cqe_section: Vec<IoCqe>,
    pub sqe_section: Vec<IoSqe>,

    pub slots: usize,

    shm: Arc<ShmBuf>,
    user_info: MetaUserInfo,
    for_read: bool,
    flags: u64,
    cqe_mtx: Mutex<()>,
    sqe_proc_tail: i32,
    processing: i32,
    sqe_proc_tails: VecDeque<i32>,
    sqe_done_tails: HashSet<i32>,
    last_check: Option<SteadyTime>,
}

impl IoRing {
    pub fn ring_marker_size() -> usize {
        4
    }

    pub fn io_ring_entries(buf_size: usize) -> i32 {
        let n = Self::ring_marker_size();
        ((buf_size - 4096 - n * 4 - mem::size_of::<usize>()) / 
            (mem::size_of::<IoArgs>() + mem::size_of::<IoCqe>() + mem::size_of::<IoSqe>())) as i32 - 1
    }

    pub fn bytes_required(entries: i32) -> usize {
        let n = Self::ring_marker_size();
        n * 4 + mem::size_of::<usize>() + 
        (mem::size_of::<IoArgs>() + mem::size_of::<IoCqe>() + mem::size_of::<IoSqe>()) * (entries as usize + 1) + 4096
    }

    pub fn new(
        shm: Arc<ShmBuf>,
        name: String,
        user_info: MetaUserInfo,
        for_read: bool,
        buf: &mut [u8],
        size: usize,
        io_depth: i32,
        priority: i32,
        timeout: Duration,
        flags: u64,
        owner: bool,
    ) -> Self {
        let entries = Self::io_ring_entries(size) + 1;
        let n = Self::ring_marker_size();

        let mut ring = Self {
            name,
            mount_name: String::new(),
            entries,
            io_depth,
            priority,
            timeout,
            sqe_head: AtomicI32::new(0),
            sqe_tail: AtomicI32::new(0),
            cqe_head: AtomicI32::new(0),
            cqe_tail: AtomicI32::new(0),
            ring_section: Vec::with_capacity(entries as usize),
            cqe_section: Vec::with_capacity(entries as usize),
            sqe_section: Vec::with_capacity(entries as usize),
            slots: (entries - 1) as usize,
            shm,
            user_info,
            for_read,
            flags,
            cqe_mtx: Mutex::new(()),
            sqe_proc_tail: 0,
            processing: 0,
            sqe_proc_tails: VecDeque::new(),
            sqe_done_tails: HashSet::new(),
            last_check: None,
        };

        // 初始化内存布局
        let buf_ptr = buf.as_mut_ptr();
        unsafe {
            ring.sqe_head = AtomicI32::new(*(buf_ptr as *const i32));
            ring.sqe_tail = AtomicI32::new(*(buf_ptr.add(n) as *const i32));
            ring.cqe_head = AtomicI32::new(*(buf_ptr.add(n * 2) as *const i32));
            ring.cqe_tail = AtomicI32::new(*(buf_ptr.add(n * 3) as *const i32));
            
            let ring_section_ptr = buf_ptr.add(n * 4) as *mut IoArgs;
            let cqe_section_ptr = ring_section_ptr.add(entries as usize) as *mut IoCqe;
            let sqe_section_ptr = cqe_section_ptr.add(entries as usize) as *mut IoSqe;

            ring.ring_section = Vec::from_raw_parts(ring_section_ptr, entries as usize, entries as usize);
            ring.cqe_section = Vec::from_raw_parts(cqe_section_ptr, entries as usize, entries as usize);
            ring.sqe_section = Vec::from_raw_parts(sqe_section_ptr, entries as usize, entries as usize);
        }

        ring
    }

    pub fn cqe_count(&self) -> i32 {
        (self.cqe_head.load(Ordering::SeqCst) + self.entries - self.cqe_tail.load(Ordering::SeqCst)) % self.entries
    }

    pub fn add_sqe(&self, idx: i32, userdata: *const u8) -> bool {
        let h = self.sqe_head.load(Ordering::SeqCst);
        if (h + 1) % self.entries == self.sqe_tail.load(Ordering::SeqCst) {
            return false;
        }

        self.sqe_section[h as usize] = IoSqe { index: idx, userdata };
        self.sqe_head.store((h + 1) % self.entries, Ordering::SeqCst);
        true
    }

    pub fn add_cqe(&self, idx: i32, res: i64, userdata: *const u8) -> bool {
        let h = self.cqe_head.load(Ordering::SeqCst);
        if (h + 1) % self.entries == self.cqe_tail.load(Ordering::SeqCst) {
            return false;
        }

        self.cqe_section[h as usize] = IoCqe {
            index: idx,
            reserved: 0,
            result: res,
            userdata,
        };
        self.cqe_head.store((h + 1) % self.entries, Ordering::SeqCst);
        true
    }

    pub fn jobs_to_proc(&self, max_jobs: i32) -> Vec<IoRingJob> {
        let mut jobs = Vec::new();
        let _lock = self.cqe_mtx.lock().unwrap();
        
        let spt = self.sqe_proc_tail;
        let sqes = self.sqe_count();
        
        let cqe_avail = self.entries - 1 - self.processing - self.cqe_count();
        
        let mut current_spt = spt;
        let mut remaining_sqes = sqes;
        
        while remaining_sqes > 0 && (jobs.len() as i32) < max_jobs {
            let to_proc = if self.io_depth > 0 {
                let depth = self.io_depth;
                if depth > remaining_sqes || depth > cqe_avail {
                    break;
                }
                depth
            } else {
                let mut to_proc = std::cmp::min(remaining_sqes, cqe_avail);
                if self.io_depth < 0 {
                    let iod = -self.io_depth;
                    if to_proc > iod {
                        to_proc = iod;
                    } else if to_proc < iod && !self.timeout.is_zero() {
                        let now = Instant::now();
                        if let Some(last_check) = self.last_check {
                            if last_check + self.timeout > now {
                                break;
                            }
                        } else {
                            self.last_check = Some(now);
                            break;
                        }
                        self.last_check = None;
                    }
                }
                to_proc
            };

            if jobs.is_empty() {
                jobs.reserve(if self.io_depth != 0 {
                    std::cmp::min(max_jobs, remaining_sqes / self.io_depth.abs() + 1)
                } else {
                    1
                } as usize);
            }

            jobs.push(IoRingJob {
                ior: Arc::new(self.clone()),
                sqe_proc_tail: current_spt,
                to_proc,
            });

            current_spt = (current_spt + to_proc) % self.entries;
            self.sqe_proc_tails.push_back(current_spt);
            self.processing += to_proc;
            remaining_sqes -= to_proc;
        }

        self.sqe_proc_tail = current_spt;
        jobs
    }

    pub async fn process(
        &self,
        spt: i32,
        to_proc: i32,
        storage_client: &mut StorageClient,
        storage_io: &IoOptions,
        user_config: &mut UserConfig,
        lookup_files: impl Fn(&mut Vec<Arc<RcInode>>, &[IoArgs], &[IoSqe], i32),
        lookup_bufs: impl Fn(&mut Vec<Result<Arc<ShmBufForIO>>>, &[IoArgs], &[IoSqe], i32),
    ) {
        // 监控记录器
        let overall_latency = LatencyRecorder::new(
            "usrbio.piov.overall",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let prepare_latency = LatencyRecorder::new(
            "usrbio.piov.prepare",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let submit_latency = LatencyRecorder::new(
            "usrbio.piov.submit",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let complete_latency = LatencyRecorder::new(
            "usrbio.piov.complete",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let io_size_dist = DistributionRecorder::new(
            "usrbio.piov.io_size",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let io_depth_dist = DistributionRecorder::new(
            "usrbio.piov.io_depth",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let total_bytes_dist = DistributionRecorder::new(
            "usrbio.piov.total_bytes",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let distinct_files_dist = DistributionRecorder::new(
            "usrbio.piov.distinct_files",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let distinct_bufs_dist = DistributionRecorder::new(
            "usrbio.piov.distinct_bufs",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );
        let bw_count = CountRecorder::new(
            "usrbio.piov.bw",
            vec![("mount_name".to_string(), self.mount_name.clone())],
        );

        let start = Instant::now();
        let overall_start = start;
        let io_type = if self.for_read { "read" } else { "write" };
        let uids = self.user_info.uid.to_string();

        let config = user_config.get_config(&self.user_info);

        let mut res = if !self.for_read && config.readonly() {
            vec![-StatusCode::kReadOnlyMode as i64; to_proc as usize]
        } else {
            vec![0; to_proc as usize]
        };

        if res[0] >= 0 {
            let mut iod = 0;
            let mut total_bytes = 0;
            let mut distinct_files = HashSet::new();
            let mut distinct_bufs = HashSet::new();

            let mut inodes = Vec::with_capacity(to_proc as usize);
            lookup_files(&mut inodes, &self.ring_section, &self.sqe_section, to_proc);

            let mut bufs = Vec::with_capacity(to_proc as usize);
            lookup_bufs(&mut bufs, &self.ring_section, &self.sqe_section, to_proc);

            // 处理每个IO请求
            for i in 0..to_proc {
                let idx = (spt + i) % self.entries;
                let sqe = &self.sqe_section[idx as usize];
                let args = &self.ring_section[sqe.index as usize];

                iod += 1;
                total_bytes += args.io_len;
                distinct_files.insert(args.file_iid);
                distinct_bufs.insert(args.buf_id);

                io_size_dist.add_sample(args.io_len, vec![
                    ("io".to_string(), io_type.to_string()),
                    ("uid".to_string(), uids.clone()),
                ]);

                if inodes[i as usize].is_none() {
                    res[i as usize] = -MetaCode::kNotFile as i64;
                    continue;
                }

                if let Err(e) = &bufs[i as usize] {
                    res[i as usize] = -e.code() as i64;
                    continue;
                }

                // 处理内存句柄
                let memh = bufs[i as usize].as_ref().unwrap().memh(args.io_len).await;
                if let Err(e) = memh {
                    res[i as usize] = -e.code() as i64;
                    continue;
                }

                let memh = memh.unwrap();
                if bufs[i as usize].as_ref().unwrap().ptr().is_none() || memh.is_none() {
                    res[i as usize] = -ClientAgentCode::kIovShmFail as i64;
                    continue;
                }

                // 处理写入操作
                if !self.for_read {
                    let begin_write = inodes[i as usize].as_ref().unwrap()
                        .begin_write(&self.user_info, storage_client, args.file_off, args.io_len)
                        .await;
                    
                    if let Err(e) = begin_write {
                        res[i as usize] = -e.code() as i64;
                        continue;
                    }
                }

                // 执行IO操作
                let io_result = if self.for_read {
                    storage_client.read(
                        &self.user_info,
                        inodes[i as usize].as_ref().unwrap().inode,
                        args.file_off,
                        args.io_len,
                        bufs[i as usize].as_ref().unwrap().ptr().unwrap(),
                        memh.unwrap(),
                    ).await
                } else {
                    storage_client.write(
                        &self.user_info,
                        inodes[i as usize].as_ref().unwrap().inode,
                        args.file_off,
                        args.io_len,
                        bufs[i as usize].as_ref().unwrap().ptr().unwrap(),
                        memh.unwrap(),
                    ).await
                };

                if let Err(e) = io_result {
                    res[i as usize] = -e.code() as i64;
                }
            }

            // 记录监控数据
            let now = Instant::now();
            prepare_latency.add_sample(now - start, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);

            io_depth_dist.add_sample(iod, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
            total_bytes_dist.add_sample(total_bytes, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
            distinct_files_dist.add_sample(distinct_files.len() as u64, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
            distinct_bufs_dist.add_sample(distinct_bufs.len() as u64, vec![
                ("io".to_string(), io_type.to_string()),
                ("uid".to_string(), uids.clone()),
            ]);
        }

        // 更新完成状态
        let new_spt = (spt + to_proc) % self.entries;
        let mut sqes = Vec::with_capacity(to_proc as usize);
        for i in 0..to_proc {
            sqes.push(self.sqe_section[(spt + i) % self.entries as usize].clone());
        }

        {
            let _lock = self.cqe_mtx.lock().unwrap();
            
            if self.sqe_proc_tails.is_empty() {
                panic!("bug?! sqe_proc_tails is empty");
            }

            if self.sqe_proc_tails[0] != new_spt {
                self.sqe_done_tails.insert(new_spt);
            } else {
                self.sqe_tail.store(new_spt, Ordering::SeqCst);
                self.sqe_proc_tails.pop_front();
                
                while !self.sqe_done_tails.is_empty() {
                    if self.sqe_proc_tails.is_empty() {
                        panic!("bug?! sqe_proc_tails is empty");
                    }
                    
                    let first = self.sqe_proc_tails[0];
                    if let Some(_) = self.sqe_done_tails.take(&first) {
                        self.sqe_tail.store(first, Ordering::SeqCst);
                        self.sqe_proc_tails.pop_front();
                    } else {
                        break;
                    }
                }
            }

            // 添加完成事件
            for i in 0..to_proc {
                let sqe = &sqes[i as usize];
                let r = res[i as usize];
                if !self.add_cqe(sqe.index, r, sqe.userdata) {
                    panic!("failed to add cqe");
                }
            }

            self.processing -= to_proc;
        }

        // 计算带宽
        let mut done_bytes = 0;
        for r in res {
            if r > 0 {
                done_bytes += r as u64;
            }
        }
        bw_count.add_sample(done_bytes, vec![
            ("io".to_string(), io_type.to_string()),
            ("uid".to_string(), uids.clone()),
        ]);

        // 记录延迟
        let now = Instant::now();
        complete_latency.add_sample(now - start, vec![
            ("io".to_string(), io_type.to_string()),
            ("uid".to_string(), uids.clone()),
        ]);
        overall_latency.add_sample(now - overall_start, vec![
            ("io".to_string(), io_type.to_string()),
            ("uid".to_string(), uids.clone()),
        ]);
    }
}

// IoRingTable 实现
pub struct IoRingTable {
    sems: Vec<Arc<tokio::sync::Semaphore>>,
    io_rings: Arc<Mutex<Vec<Option<Arc<IoRing>>>>>,
}

impl IoRingTable {
    pub fn new(cap: usize) -> Self {
        let mut sems = Vec::new();
        for prio in 0..=2 {
            let sem = Arc::new(tokio::sync::Semaphore::new(1));
            sems.push(sem);
        }

        Self {
            sems,
            io_rings: Arc::new(Mutex::new(vec![None; cap])),
        }
    }

    pub fn add_io_ring(
        &self,
        mount_name: Path,
        shm: Arc<ShmBuf>,
        name: String,
        user_info: MetaUserInfo,
        for_read: bool,
        buf: &mut [u8],
        size: usize,
        io_depth: i32,
        attrs: IorAttrs,
    ) -> Result<i32> {
        let mut io_rings = self.io_rings.lock().unwrap();
        
        // 查找空闲槽位
        let idx = io_rings.iter().position(|x| x.is_none())
            .ok_or_else(|| "too many io rings".to_string())?;

        let ior = Arc::new(IoRing::new(
            shm,
            name,
            user_info,
            for_read,
            buf,
            size,
            io_depth,
            attrs.priority,
            attrs.timeout,
            attrs.flags,
            true,
        ));
        ior.mount_name = mount_name.to_string();
        
        io_rings[idx] = Some(ior);
        Ok(idx as i32)
    }

    pub fn remove_io_ring(&self, idx: i32) {
        let mut io_rings = self.io_rings.lock().unwrap();
        if idx >= 0 && (idx as usize) < io_rings.len() {
            io_rings[idx as usize] = None;
        }
    }
}