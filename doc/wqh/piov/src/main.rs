mod lib {
    pub mod agent {
        pub mod piov;
    }
}

use lib::agent::piov::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 构造 StorageClient
    let storage_client = StorageClient;

    // 构造 res 数组
    let mut res = vec![0isize; 1];

    // 构造 Inode
    let inode = Inode { id: InodeId(1) };

    // 构造 IOBuffer
    let mut memh = IOBuffer;

    // 构造 UserInfo
    let user_info = UserInfo;

    // 构造 ReadOptions/WriteOptions
    let read_options = ReadOptions;
    let write_options = WriteOptions;

    {
        let mut buf = vec![0u8; 4096];
        let mut piov = PioV::new(&storage_client, 4096, &mut res);
        // 添加读操作
        piov.add_read(0, &inode, 0, 0, 4096, &mut buf, &mut memh)?;

        // 执行读
        piov.execute_read(&user_info, &read_options).await?;

        // 完成IO
        piov.finish_io(true);
    }
    // 这里 res 不再被 piov 可变借用
    println!("res: {:?}", res);

    {
        let mut piov = PioV::new(&storage_client, 4096, &mut res);
        // 添加写操作
        let wbuf = vec![1u8; 4096];
        piov.add_write(0, &inode, 0, 0, 4096, &wbuf, &mut memh)?;

        // 执行写
        piov.execute_write(&user_info, &write_options).await?;

        // 完成IO
        piov.finish_io(false);
    }
    println!("res after write: {:?}", res);

    Ok(())
}