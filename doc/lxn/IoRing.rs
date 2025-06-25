// Rust 版本的 IoRing，补全 IO 队列管理和异步处理
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

#[derive(Debug, Clone, Default)]
pub struct IoArgs {
    pub buf_id: [u8; 16],
    pub buf_off: usize,
    pub file_iid: u64,
    pub file_off: usize,
    pub io_len: u64,
    pub userdata: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct IoSqe {
    pub index: i32,
    pub userdata: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct IoCqe {
    pub index: i32,
    pub reserved: i32,
    pub result: i64,
    pub userdata: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct IoRingJob {
    pub ior: Arc<IoRing>,
    pub sqe_proc_tail: i32,
    pub to_proc: i32,
}

impl Default for IoRingJob {
    fn default() -> Self {
        Self {
            ior: Arc::new(IoRing::default()),
            sqe_proc_tail: 0,
            to_proc: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct IoRing {
    pub name: String,
    pub entries: i32,
    pub io_depth: i32,
    pub priority: i32,
    pub sqe_head: i32,
    pub sqe_tail: i32,
    pub cqe_head: i32,
    pub cqe_tail: i32,
    pub jobs: Mutex<Vec<IoRingJob>>,
    pub notify: Arc<Notify>,
}

impl IoRing {
    pub fn new(name: &str, entries: i32, io_depth: i32, priority: i32) -> Self {
        Self {
            name: name.to_string(),
            entries,
            io_depth,
            priority,
            sqe_head: 0,
            sqe_tail: 0,
            cqe_head: 0,
            cqe_tail: 0,
            jobs: Mutex::new(Vec::new()),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn add_sqe(&self, sqe: IoSqe) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.push(IoRingJob {
            ior: Arc::new(self.clone()),
            sqe_proc_tail: sqe.index,
            to_proc: 1,
        });
        self.notify.notify_one();
    }

    pub fn jobs_to_proc(&self, max_jobs: i32) -> Vec<IoRingJob> {
        let mut jobs = self.jobs.lock().unwrap();
        let take = std::cmp::min(jobs.len(), max_jobs as usize);
        jobs.drain(0..take).collect()
    }

    pub async fn process(&self) {
        loop {
            self.notify.notified().await;
            // 实际应处理IO任务，这里只做模拟
            println!("[IoRing] process: 有新IO任务");
        }
    }
}

// 单元测试示例
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ioring_jobs_to_proc() {
        let ioring = IoRing::new("test", 128, 8, 1);
        let jobs = ioring.jobs_to_proc(4);
        assert_eq!(jobs.len(), 4);
    }
    #[test]
    fn test_ioring_process() {
        let ioring = IoRing::new("test", 128, 8, 1);
        ioring.process(0, 2);
    }
} 