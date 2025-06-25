mod ioring;
use ioring::*;
use std::collections::HashMap;
use std::time::Duration;
use tokio_uring::fs::File;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    tokio_uring::start(async {
        // 创建测试文件
        let test_file_path = "testfile.bin";
        let file = Arc::new(File::create(test_file_path).await?);
        let mut file_map = HashMap::new();
        file_map.insert(1u64, file);

        // 创建 IoRing
        let ioring = IoRing::new("test_ring", 16, 4, 0, Duration::from_secs(5), false);

        // 添加写操作
        for i in 0..4 {
            let args = IoArgs {
                file_id: 1,
                file_off: (i * 4096) as u64,
                io_len: 4096,
                buf: vec![i as u8; 4096],
                userdata: Some(i),
            };
            ioring.add_sqe(args);
        }

        // 执行写
        let cqes = ioring.process(&file_map).await?;
        println!("写完成队列: {:?}", cqes);

        // 切换为读
        let ioring = IoRing::new("test_ring", 16, 4, 0, Duration::from_secs(5), true);
        for i in 0..4 {
            let args = IoArgs {
                file_id: 1,
                file_off: (i * 4096) as u64,
                io_len: 4096,
                buf: vec![],
                userdata: Some(i),
            };
            ioring.add_sqe(args);
        }
        let cqes = ioring.process(&file_map).await?;
        println!("读完成队列: {:?}", cqes);

        Ok(())
    })
}
