use std::{
    collections::{VecDeque, HashMap},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Semaphore;
use tokio_uring::fs::File;
use tokio_uring::buf::IoBufMut;
use anyhow::{Result, bail};

#[derive(Clone, Debug)]
pub struct IoArgs {
    pub file_id: u64,
    pub file_off: u64,
    pub io_len: usize,
    pub buf: Vec<u8>,
    pub userdata: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct IoSqe {
    pub index: usize,
    pub userdata: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct IoCqe {
    pub index: usize,
    pub result: isize,
    pub userdata: Option<u64>,
}

#[derive(Clone)]
pub struct IoRingJob {
    pub ioring: Arc<IoRing>,
    pub sqe_proc_tail: usize,
    pub to_proc: usize,
}

pub struct IoRing {
    pub name: String,
    pub entries: usize,
    pub io_depth: usize,
    pub priority: usize,
    pub timeout: Duration,
    pub for_read: bool,
    pub args_queue: Mutex<VecDeque<IoArgs>>,
    pub sqe_queue: Mutex<VecDeque<IoSqe>>,
    pub cqe_queue: Mutex<VecDeque<IoCqe>>,
    pub semaphore: Arc<Semaphore>,
}

impl IoRing {
    pub fn new(name: &str, entries: usize, io_depth: usize, priority: usize, timeout: Duration, for_read: bool) -> Arc<Self> {
        Arc::new(Self {
            name: name.to_string(),
            entries,
            io_depth,
            priority,
            timeout,
            for_read,
            args_queue: Mutex::new(VecDeque::with_capacity(entries)),
            sqe_queue: Mutex::new(VecDeque::with_capacity(entries)),
            cqe_queue: Mutex::new(VecDeque::with_capacity(entries)),
            semaphore: Arc::new(Semaphore::new(io_depth)),
        })
    }

    /// 添加一个 IO 提交项
    pub fn add_sqe(&self, args: IoArgs) -> bool {
        let mut args_queue = self.args_queue.lock().unwrap();
        let mut sqe_queue = self.sqe_queue.lock().unwrap();
        if args_queue.len() >= self.entries {
            return false;
        }
        let idx = args_queue.len();
        args_queue.push_back(args);
        sqe_queue.push_back(IoSqe { index: idx, userdata: None });
        true
    }

    /// 获取可处理的 jobs
    pub fn jobs_to_proc(self: &Arc<Self>, max_jobs: usize) -> Vec<IoRingJob> {
        let args_queue = self.args_queue.lock().unwrap();
        let available = args_queue.len().min(max_jobs);
        if available == 0 {
            return vec![];
        }
        vec![IoRingJob {
            ioring: self.clone(),
            sqe_proc_tail: 0,
            to_proc: available,
        }]
    }

    /// 处理一批 IO（异步）
    pub async fn process(&self, file_map: &HashMap<u64, Arc<File>>) -> Result<Vec<IoCqe>> {
        let mut cqe_vec = Vec::new();
        let mut args_vec = Vec::new();
        {
            let mut args_queue = self.args_queue.lock().unwrap();
            for _ in 0..self.io_depth.min(args_queue.len()) {
                if let Some(args) = args_queue.pop_front() {
                    args_vec.push(args);
                }
            }
        }
        for (i, args) in args_vec.into_iter().enumerate() {
            let file = file_map.get(&args.file_id).ok_or_else(|| anyhow::anyhow!("file_id not found"))?.clone();
            let for_read = self.for_read;
            let semaphore = self.semaphore.clone();
            let permit = semaphore.acquire_owned().await.unwrap();
            let res = if for_read {
                // 读
                let mut buf = vec![0u8; args.io_len];
                let (res, _buf) = file.read_at(buf, args.file_off).await;
                IoCqe {
                    index: i,
                    result: res.map(|n| n as isize).unwrap_or(-1),
                    userdata: args.userdata,
                }
            } else {
                // 写
                let (res, _buf) = file.write_at(args.buf, args.file_off).await;
                IoCqe {
                    index: i,
                    result: res.map(|n| n as isize).unwrap_or(-1),
                    userdata: args.userdata,
                }
            };
            drop(permit);
            cqe_vec.push(res);
        }
        // 填充完成队列
        let mut cqe_queue = self.cqe_queue.lock().unwrap();
        for cqe in &cqe_vec {
            cqe_queue.push_back(cqe.clone());
        }
        Ok(cqe_vec)
    }
}

// IoRingTable 管理多个 IoRing
pub struct IoRingTable {
    pub rings: Mutex<HashMap<String, Arc<IoRing>>>,
}

impl IoRingTable {
    pub fn new() -> Self {
        Self {
            rings: Mutex::new(HashMap::new()),
        }
    }
    pub fn add_ring(&self, ring: Arc<IoRing>) {
        self.rings.lock().unwrap().insert(ring.name.clone(), ring);
    }
    pub fn get_ring(&self, name: &str) -> Option<Arc<IoRing>> {
        self.rings.lock().unwrap().get(name).cloned()
    }
    pub fn remove_ring(&self, name: &str) {
        self.rings.lock().unwrap().remove(name);
    }
}
