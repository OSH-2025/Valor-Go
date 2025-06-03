// src/lib/agent/pio_v.rs

use std::{
    collections::HashMap,
    sync::Arc,
    vec::Vec,
};

use crate::{
    storage::client::{StorageClient, IOBuffer, ReadOptions, WriteOptions, WriteIO, ReadIO, TruncateChunkOp},
    meta::{Inode, InodeId, ChunkId, ChainId},
    common::{Result, Void, StatusCode, MetaCode, ClientAgentCode, StorageClientCode},
    utils::UserInfo,
};

/// PioV 结构体用于管理并行 IO 操作
pub struct PioV<'a> {
    storage_client: &'a mut StorageClient,
    chunk_size_lim: i32,
    res: &'a mut Vec<i64>,
    rios: Vec<ReadIO>,
    wios: Vec<WriteIO>,
    trops: Vec<TruncateChunkOp>,
    potential_lens: HashMap<InodeId, u64>,
    routing_info: Arc<RoutingInfo>,
}

impl<'a> PioV<'a> {
    /// 创建一个新的 PioV 实例
    pub fn new(
        storage_client: &'a mut StorageClient,
        chunk_size_lim: i32,
        res: &'a mut Vec<i64>,
    ) -> Self {
        let mgmtd_client = storage_client.get_mgmtd_client();
        let routing_info = mgmtd_client.get_routing_info()
            .expect("RoutingInfo not found")
            .raw();

        Self {
            storage_client,
            chunk_size_lim,
            res,
            rios: Vec::new(),
            wios: Vec::new(),
            trops: Vec::new(),
            potential_lens: HashMap::new(),
            routing_info,
        }
    }

    /// 添加读取操作
    pub async fn add_read(
        &mut self,
        idx: usize,
        inode: &Inode,
        track: u16,
        off: i64,
        len: usize,
        buf: *mut u8,
        memh: &mut IOBuffer,
    ) -> Result<Void> {
        if !self.wios.is_empty() {
            return Err(StatusCode::kInvalidArg.into());
        }
        
        if !inode.is_file() {
            self.res[idx] = -(MetaCode::kNotFile as i64);
            return Ok(Void);
        }

        if self.rios.is_empty() {
            self.rios.reserve(self.res.len());
        }

        let mut buf_off = 0;
        self.chunk_io(inode, track, off, len, |chain, chunk, _, chunk_off, chunk_len| {
            self.rios.push(self.storage_client.create_read_io(
                chain,
                chunk,
                chunk_off,
                chunk_len,
                unsafe { buf.add(buf_off) },
                memh,
                idx as *mut _,
            ));
            buf_off += chunk_len;
        })?;

        Ok(Void)
    }

    /// 检查写入偏移量
    pub async fn check_write_off(
        &mut self,
        idx: usize,
        meta_client: Option<&MetaClient>,
        user_info: Option<&UserInfo>,
        inode: &Inode,
        off: usize,
    ) -> Result<bool> {
        if let (Some(meta_client), Some(user_info)) = (meta_client, user_info) {
            // 如果已知长度小于偏移量，则从元数据服务器获取最新文件长度
            if let Some(len) = self.potential_lens.get(&inode.id) {
                if *len < off as u64 {
                    let latest_len = meta_client.get_file_length(inode.id, user_info).await?;
                    return Ok(latest_len >= off as u64);
                }
            }
        }
        Ok(true)
    }

    /// 添加写入操作
    pub async fn add_write(
        &mut self,
        idx: usize,
        inode: &Inode,
        track: u16,
        off: i64,
        len: usize,
        buf: *const u8,
        memh: &mut IOBuffer,
    ) -> Result<Void> {
        if !self.rios.is_empty() {
            return Err(StatusCode::kInvalidArg.into());
        }
        
        if !inode.is_file() {
            self.res[idx] = -(MetaCode::kNotFile as i64);
            return Ok(Void);
        }

        if self.wios.is_empty() {
            self.wios.reserve(self.res.len());
        }

        let mut buf_off = 0;
        self.chunk_io(inode, track, off, len, |chain, chunk, chunk_size, chunk_off, chunk_len| {
            self.wios.push(self.storage_client.create_write_io(
                chain,
                chunk,
                chunk_off,
                chunk_len,
                chunk_size,
                unsafe { buf.add(buf_off) },
                memh,
                idx as *mut _,
            ));
            buf_off += chunk_len;
            let potential_len = off + buf_off as i64 + chunk_len as i64;
            self.potential_lens.entry(inode.id)
                .and_modify(|e| *e = (*e).max(potential_len as u64))
                .or_insert(potential_len as u64);
        })?;

        Ok(Void)
    }

    /// 执行分块 IO 操作
    fn chunk_io<F>(
        &self,
        inode: &Inode,
        track: u16,
        off: i64,
        len: usize,
        mut consume_chunk: F,
    ) -> Result<Void>
    where
        F: FnMut(ChainId, ChunkId, u32, u32, u32),
    {
        let f = inode.as_file();
        let chunk_size = f.layout.chunk_size;
        let chunk_off = (off % chunk_size as i64) as u32;

        let rcs = if self.chunk_size_lim > 0 {
            self.chunk_size_lim.min(chunk_size) as u32
        } else {
            chunk_size
        };

        let mut last_l = 0;
        let mut l = (chunk_size - chunk_off).min(len);
        
        while l < len + chunk_size as usize {
            l = l.min(len);
            let op_off = off + last_l as i64;

            let chain = f.get_chain_id(inode, op_off, &self.routing_info, track)?;
            let fchunk = f.get_chunk_id(inode.id, op_off)?;
            let chunk = ChunkId::from(*fchunk);
            let chunk_len = l - last_l;

            for co in (0..chunk_len).step_by(rcs as usize) {
                consume_chunk(
                    *chain,
                    chunk,
                    chunk_size,
                    chunk_off + co as u32,
                    rcs.min((chunk_len - co) as u32),
                );
            }

            last_l = l;
            l += chunk_size as usize;
        }

        Ok(Void)
    }

    /// 执行读取操作
    pub async fn execute_read(
        &mut self,
        user_info: &UserInfo,
        options: &ReadOptions,
    ) -> Result<Void> {
        assert!(self.wios.is_empty() && self.trops.is_empty());

        if self.rios.is_empty() {
            return Ok(Void);
        }

        self.storage_client.batch_read(&mut self.rios, user_info, options).await
    }

    /// 执行写入操作
    pub async fn execute_write(
        &mut self,
        user_info: &UserInfo,
        options: &WriteOptions,
    ) -> Result<Void> {
        assert!(self.rios.is_empty());

        if self.wios.is_empty() {
            return Ok(Void);
        }

        if !self.trops.is_empty() {
            let mut failed = Vec::new();
            let mut bad_wios = std::collections::HashSet::new();
            
            let r = self.storage_client.truncate_chunks(
                &mut self.trops,
                user_info,
                options,
                &mut failed,
            ).await?;

            if !failed.is_empty() {
                for op in &failed {
                    self.res[op.user_ctx as usize] = -(op.result.length_info.error().code() as i64);
                    for (i, wio) in self.wios.iter().enumerate() {
                        if wio.user_ctx == op.user_ctx {
                            bad_wios.insert(i);
                        }
                    }
                }

                let mut wios2 = Vec::with_capacity(self.wios.len() - bad_wios.len());
                for (i, wio) in self.wios.iter().enumerate() {
                    if !bad_wios.contains(&i) {
                        wios2.push(self.storage_client.create_write_io(
                            wio.routing_target.chain_id,
                            wio.chunk_id,
                            wio.offset,
                            wio.length,
                            wio.chunk_size,
                            wio.data,
                            wio.buffer,
                            wio.user_ctx,
                        ));
                    }
                }
                self.wios = wios2;
            }
        }

        self.storage_client.batch_write(&mut self.wios, user_info, options).await
    }

    /// 完成 IO 操作并处理结果
    pub fn finish_io(&mut self, allow_holes: bool) {
        if self.wios.is_empty() {
            concat_io_res(true, self.res, &self.rios, allow_holes);
        } else {
            concat_io_res(false, self.res, &self.wios, false);
        }
    }
}

/// 处理 IO 结果
fn concat_io_res<I: IO>(
    read: bool,
    res: &mut [i64],
    ios: &[I],
    allow_holes: bool,
) {
    let mut last_iov_idx = -1;
    let mut in_hole = false;
    let mut hole_io = None;
    let mut hole_off = 0;
    let mut hole_size = 0;
    let mut iov_idx = 0;

    for (i, io) in ios.iter().enumerate() {
        iov_idx = io.user_ctx() as isize;
        let mut iolen = 0;

        if let Some(len_info) = io.result().length_info {
            iolen = len_info;
            if iolen > 0 && in_hole && last_iov_idx == iov_idx {
                if read && allow_holes {
                    let hio = &ios[hole_io.unwrap()];
                    unsafe {
                        std::ptr::write_bytes(
                            hio.data().add(hole_off),
                            0,
                            hio.length() - hole_off,
                        );
                    }
                    for j in hole_io.unwrap() + 1..i {
                        unsafe {
                            std::ptr::write_bytes(ios[j].data(), 0, ios[j].length());
                        }
                    }

                    res[iov_idx as usize] += hole_size as i64;
                    in_hole = false;
                    hole_io = None;
                } else {
                    res[iov_idx as usize] = -(ClientAgentCode::kHoleInIoOutcome as i64);
                }
            } else if last_iov_idx != iov_idx {
                in_hole = false;
                hole_io = None;
            }
        } else if read && io.result().length_info.error().code() == StorageClientCode::kChunkNotFound {
            // ignore
        } else if res[iov_idx as usize] >= 0 {
            res[iov_idx as usize] = -(io.result().length_info.error().code() as i64);
        }

        if res[iov_idx as usize] < 0 {
            continue;
        }

        if iolen < io.length() {
            in_hole = true;
            if hole_io.is_none() {
                hole_io = Some(i);
                hole_off = iolen;
                hole_size = 0;
            }
            hole_size += io.length() - iolen;
        }
        res[iov_idx as usize] += iolen as i64;
        last_iov_idx = iov_idx;
    }
}

/// IO 操作特征
pub trait IO {
    fn user_ctx(&self) -> *mut u8;
    fn length(&self) -> u32;
    fn result(&self) -> &IOResult;
    fn data(&self) -> *mut u8;
}

impl IO for ReadIO {
    fn user_ctx(&self) -> *mut u8 {
        self.user_ctx
    }
    fn length(&self) -> u32 {
        self.length
    }
    fn result(&self) -> &IOResult {
        &self.result
    }
    fn data(&self) -> *mut u8 {
        self.data
    }
}

impl IO for WriteIO {
    fn user_ctx(&self) -> *mut u8 {
        self.user_ctx
    }
    fn length(&self) -> u32 {
        self.length
    }
    fn result(&self) -> &IOResult {
        &self.result
    }
    fn data(&self) -> *mut u8 {
        self.data
    }
}

/// 错误码定义
#[derive(Debug)]
pub enum StatusCode {
    kInvalidArg = 1,
}

#[derive(Debug)]
pub enum MetaCode {
    kNotFile = 1,
}

#[derive(Debug)]
pub enum ClientAgentCode {
    kHoleInIoOutcome = 1,
}

#[derive(Debug)]
pub enum StorageClientCode {
    kChunkNotFound = 1,
}

/// IO 结果结构
#[derive(Debug)]
pub struct IOResult {
    pub length_info: Option<u32>,
    pub error: Option<Error>,
}

#[derive(Debug)]
pub struct Error {
    pub code: i32,
}

impl Error {
    pub fn code(&self) -> i32 {
        self.code
    }
}

/// 路由信息结构
#[derive(Debug)]
pub struct RoutingInfo {
    // 实现路由信息的具体字段
}

/// 空类型
#[derive(Debug)]
pub struct Void;