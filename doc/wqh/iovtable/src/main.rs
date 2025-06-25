mod iov_table;
use iov_table::*;
use std::path::Path;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建测试文件
    let test_file_path = "test_shm.bin";
    tokio::fs::write(test_file_path, vec![0u8; 4096]).await?;

    let user = UserInfo { uid: 1000, gid: 1000 };
    let key = format!("{}b4096", Uuid::new_v4().simple().to_string());

    let table = IovTable::new("mnt");

    // 添加
    let shm = table.add_iov(&key, Path::new(test_file_path), &user).await?;
    println!("add_iov: {:?}", shm);

    // 查找
    let shm2 = table.lookup_iov(&key, &user).await?;
    println!("lookup_iov: {:?}", shm2);

    // 列出
    let all = table.list_iovs(&user).await;
    println!("list_iovs: {:?}", all);

    // 删除
    let shm3 = table.rm_iov(&key, &user).await?;
    println!("rm_iov: {:?}", shm3);

    Ok(())
}