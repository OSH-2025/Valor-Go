use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use anyhow::{Result, bail};
use async_trait::async_trait;

// ====== 你需要根据实际项目实现这些类型 ======
pub type Void = ();
pub type StatusCode = i32;
pub type MetaCode = i32;
pub type ClientAgentCode = i32;
pub type StorageClientCode = i32;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct InodeId(pub u64);

pub struct Inode {
    pub id: InodeId,
    // 你需要实现 is_file, as_file, 等方法
}
impl Inode {
    pub fn is_file(&self) -> bool { true }
    pub fn as_file(&self) -> &File { &File { layout: FileLayout { chunk_size: 4096 } } }
}
pub struct File {
    pub layout: FileLayout,
}
pub struct FileLayout {
    pub chunk_size: usize,
}
impl File {
    pub fn get_chain_id(&self, _inode: &Inode, _off: isize, _routing: &RoutingInfo, _track: u16) -> Result<ChainId> { Ok(ChainId) }
    pub fn get_chunk_id(&self, _inode_id: InodeId, _off: isize) -> Result<ChunkId> { Ok(ChunkId) }
}
#[derive(Clone, Copy)]
pub struct ChainId;
#[derive(Clone, Copy)]
pub struct ChunkId;
pub struct IOBuffer;
pub struct UserInfo;
pub struct ReadOptions;
pub struct WriteOptions;
pub struct RoutingInfo;
pub struct ReadIO { pub user_ctx: usize, pub length: usize, pub result: IOResult, pub data: Vec<u8> }
pub struct WriteIO { pub user_ctx: usize, pub length: usize, pub result: IOResult, pub data: Vec<u8> }
pub struct TruncateChunkOp { pub user_ctx: usize, pub result: IOResult }
pub struct IOResult { pub length_info: Option<usize>, pub error_code: Option<i32> }
pub struct StorageClient;
impl StorageClient {
    pub fn get_mgmtd_client(&self) -> &Self { self }
    pub fn get_routing_info(&self) -> Option<RoutingInfo> { Some(RoutingInfo) }
    pub fn create_read_io(&self, _chain: ChainId, _chunk: ChunkId, _chunk_off: usize, chunk_len: usize, _buf: &[u8], _memh: &mut IOBuffer, user_ctx: usize) -> ReadIO {
        ReadIO { user_ctx, length: chunk_len, result: IOResult { length_info: Some(chunk_len), error_code: None }, data: vec![0; chunk_len] }
    }
    pub fn create_write_io(&self, _chain: ChainId, _chunk: ChunkId, _chunk_off: usize, chunk_len: usize, _chunk_size: usize, _buf: &[u8], _memh: &mut IOBuffer, user_ctx: usize) -> WriteIO {
        WriteIO { user_ctx, length: chunk_len, result: IOResult { length_info: Some(chunk_len), error_code: None }, data: vec![1; chunk_len] }
    }
    pub async fn batch_read(&self, _rios: &mut [ReadIO], _user_info: &UserInfo, _options: &ReadOptions) -> Result<Void> { Ok(()) }
    pub async fn batch_write(&self, _wios: &mut [WriteIO], _user_info: &UserInfo, _options: &WriteOptions) -> Result<Void> { Ok(()) }
    pub async fn truncate_chunks(&self, _trops: &mut [TruncateChunkOp], _user_info: &UserInfo, _options: &WriteOptions, _failed: &mut Vec<TruncateChunkOp>) -> Result<Void> { Ok(()) }
}
// ===========================================

pub struct PioV<'a> {
    storage_client: &'a StorageClient,
    chunk_size_lim: usize,
    res: &'a mut [isize],
    rios: Vec<ReadIO>,
    wios: Vec<WriteIO>,
    trops: Vec<TruncateChunkOp>,
    potential_lens: HashMap<InodeId, usize>,
    routing_info: RoutingInfo,
}

impl<'a> PioV<'a> {
    pub fn new(storage_client: &'a StorageClient, chunk_size_lim: usize, res: &'a mut [isize]) -> Self {
        let routing_info = storage_client.get_mgmtd_client().get_routing_info().expect("RoutingInfo not found");
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

    pub fn add_read(
        &mut self,
        idx: usize,
        inode: &Inode,
        track: u16,
        off: isize,
        len: usize,
        buf: &mut [u8],
        memh: &mut IOBuffer,
    ) -> Result<Void> {
        if !self.wios.is_empty() {
            bail!("adding read to write operations");
        } else if !inode.is_file() {
            self.res[idx] = -1;
            return Ok(());
        }
        if self.rios.is_empty() {
            self.rios.reserve(self.res.len());
        }
        let mut buf_off = 0;
        let mut rios = std::mem::take(&mut self.rios);
        let storage_client = self.storage_client;
        self.chunk_io(inode, track, off, len, |chain, chunk, _chunk_size, chunk_off, chunk_len| {
            let slice = &buf[buf_off..buf_off + chunk_len];
            let read_io = storage_client.create_read_io(chain, chunk, chunk_off, chunk_len, slice, memh, idx);
            rios.push(read_io);
            buf_off += chunk_len;
        })?;
        self.rios = rios;
        Ok(())
    }

    pub fn add_write(
        &mut self,
        idx: usize,
        inode: &Inode,
        track: u16,
        off: isize,
        len: usize,
        buf: &[u8],
        memh: &mut IOBuffer,
    ) -> Result<Void> {
        if !self.rios.is_empty() {
            bail!("adding write to read operations");
        } else if !inode.is_file() {
            self.res[idx] = -1;
            return Ok(());
        }
        if self.wios.is_empty() {
            self.wios.reserve(self.res.len());
        }
        let mut buf_off = 0;
        let mut wios = std::mem::take(&mut self.wios);
        let mut potential_lens = std::mem::take(&mut self.potential_lens);
        let storage_client = self.storage_client;
        let inode_id = inode.id;
        self.chunk_io(inode, track, off, len, |chain, chunk, chunk_size, chunk_off, chunk_len| {
            let slice = &buf[buf_off..buf_off + chunk_len];
            let write_io = storage_client.create_write_io(chain, chunk, chunk_off, chunk_len, chunk_size, slice, memh, idx);
            wios.push(write_io);
            buf_off += chunk_len;
            let potential_len = (off as usize) + buf_off + chunk_len;
            potential_lens.entry(inode_id).and_modify(|e| *e = (*e).max(potential_len)).or_insert(potential_len);
        })?;
        self.wios = wios;
        self.potential_lens = potential_lens;
        Ok(())
    }

    fn chunk_io<F>(&self, inode: &Inode, track: u16, off: isize, len: usize, mut consume_chunk: F) -> Result<Void>
    where
        F: FnMut(ChainId, ChunkId, usize, usize, usize),
    {
        let f = inode.as_file();
        let chunk_size = f.layout.chunk_size;
        let mut chunk_off = (off as usize) % chunk_size;
        let rcs = if self.chunk_size_lim > 0 { self.chunk_size_lim.min(chunk_size) } else { chunk_size };
        let mut last_l = 0;
        let mut l = (chunk_size - chunk_off).min(len);
        while l < len + chunk_size {
            l = l.min(len);
            let op_off = off + last_l as isize;
            let chain = f.get_chain_id(inode, op_off, &self.routing_info, track)?;
            let fchunk = f.get_chunk_id(inode.id, op_off)?;
            let chunk = fchunk;
            let chunk_len = l - last_l;
            for co in (0..chunk_len).step_by(rcs) {
                consume_chunk(chain, chunk, chunk_size, chunk_off + co, rcs.min(chunk_len - co));
            }
            last_l = l;
            l += chunk_size;
            chunk_off = 0;
        }
        Ok(())
    }

    pub async fn execute_read(&mut self, user_info: &UserInfo, options: &ReadOptions) -> Result<Void> {
        assert!(self.wios.is_empty() && self.trops.is_empty());
        if self.rios.is_empty() {
            return Ok(());
        }
        self.storage_client.batch_read(&mut self.rios, user_info, options).await
    }

    pub async fn execute_write(&mut self, user_info: &UserInfo, options: &WriteOptions) -> Result<Void> {
        assert!(self.rios.is_empty());
        if self.wios.is_empty() {
            return Ok(());
        }
        if !self.trops.is_empty() {
            let mut failed = Vec::new();
            let mut bad_wios = HashSet::new();
            self.storage_client.truncate_chunks(&mut self.trops, user_info, options, &mut failed).await?;
            if !failed.is_empty() {
                for op in &failed {
                    self.res[op.user_ctx] = -1;
                    for (i, _wio) in self.wios.iter().enumerate() {
                        if _wio.user_ctx == op.user_ctx {
                            bad_wios.insert(i);
                        }
                    }
                }
                let mut wios2 = Vec::with_capacity(self.wios.len() - bad_wios.len());
                for (i, wio) in self.wios.iter().enumerate() {
                    if !bad_wios.contains(&i) {
                        wios2.push(self.storage_client.create_write_io(
                            ChainId, ChunkId, 0, 0, 0, &[], &mut IOBuffer, 0
                        ));
                    }
                }
                self.wios = wios2;
            }
        }
        self.storage_client.batch_write(&mut self.wios, user_info, options).await
    }

    pub fn finish_io(&mut self, allow_holes: bool) {
        if self.wios.is_empty() {
            concat_io_res::<ReadIO>(true, self.res, &mut self.rios, allow_holes);
        } else {
            concat_io_res::<WriteIO>(false, self.res, &mut self.wios, false);
        }
    }
}

// 处理IO结果
fn concat_io_res<I: IO + Sized>(
    read: bool,
    res: &mut [isize],
    ios: &mut [I],
    allow_holes: bool,
) {
    let mut last_iov_idx = -1isize;
    let mut in_hole = false;
    let mut hole_io = None;
    let mut hole_off = 0;
    let mut hole_size = 0;
    let mut iov_idx = 0isize;

    let len = ios.len();
    let mut i = 0;
    let mut fill_zero_ranges: Vec<(usize, usize, usize)> = Vec::new();
    while i < len {
        let io_user_ctx = ios[i].user_ctx();
        iov_idx = io_user_ctx as isize;
        let mut iolen = 0;
        if let Some(len_info) = ios[i].result().length_info {
            iolen = len_info as usize;
            if iolen > 0 && in_hole && last_iov_idx == iov_idx {
                if read && allow_holes {
                    let hio_idx: usize = hole_io.unwrap();
                    let hio_len = ios[hio_idx].length();
                    fill_zero_ranges.push((hio_idx, hole_off, hio_len));
                    for j in (hio_idx + 1)..i {
                        let jlen = ios[j].length();
                        fill_zero_ranges.push((j, 0, jlen));
                    }
                    res[iov_idx as usize] += hole_size as isize;
                    in_hole = false;
                    hole_io = None;
                } else {
                    res[iov_idx as usize] = -1;
                }
            } else if last_iov_idx != iov_idx {
                in_hole = false;
                hole_io = None;
            }
        } else if read && ios[i].result().error_code == Some(1) {
            // ignore
        } else if res[iov_idx as usize] >= 0 {
            res[iov_idx as usize] = -1;
        }
        if res[iov_idx as usize] < 0 {
            i += 1;
            continue;
        }
        if iolen < ios[i].length() {
            in_hole = true;
            if hole_io.is_none() {
                hole_io = Some(i);
                hole_off = iolen;
                hole_size = 0;
            }
            hole_size += ios[i].length() - iolen;
        }
        res[iov_idx as usize] += iolen as isize;
        last_iov_idx = iov_idx;
        i += 1;
    }
    // 循环结束后统一填充
    for (idx, start, end) in fill_zero_ranges {
        ios[idx].data_mut()[start..end].fill(0);
    }
}
// IO trait
pub trait IO {
    fn user_ctx(&self) -> usize;
    fn length(&self) -> usize;
    fn result(&self) -> &IOResult;
    fn data_mut(&mut self) -> &mut [u8];
}

impl IO for ReadIO {
    fn user_ctx(&self) -> usize { self.user_ctx }
    fn length(&self) -> usize { self.length }
    fn result(&self) -> &IOResult { &self.result }
    fn data_mut(&mut self) -> &mut [u8] { &mut self.data }
}

impl IO for WriteIO {
    fn user_ctx(&self) -> usize { self.user_ctx }
    fn length(&self) -> usize { self.length }
    fn result(&self) -> &IOResult { &self.result }
    fn data_mut(&mut self) -> &mut [u8] { &mut self.data }
} 