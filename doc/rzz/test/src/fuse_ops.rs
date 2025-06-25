use fuser::{Filesystem, FileAttr, FileType, Request, ReplyAttr, ReplyEntry, ReplyData, ReplyDirectory, ReplyEmpty, ReplyWrite, ReplyOpen, ReplyCreate, ReplyXattr, ReplyIoctl, MountOption, KernelConfig, TimeOrNow, ReplyDirectoryPlus};
use libc::{ENOENT, EROFS, ENOSYS, EINVAL, ENOTSUP, ENODATA, ENOTDIR, EPERM};
use std::ffi::{OsStr, OsString};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::path::PathBuf;

// 你可以根据实际需要引入自己的后端类型
// use crate::{MetaClient, StorageClient, Inode, ...};

// ========== 核心数据结构 ==========

#[derive(Debug, Clone)]
pub enum InodeData {
    Directory { entries: HashMap<OsString, u64> },
    File { content: Vec<u8> },
    Symlink { target: PathBuf },
}

#[derive(Debug, Clone)]
pub struct Inode {
    pub ino: u64,
    pub kind: FileType,
    pub attr: FileAttr,
    pub data: InodeData,
}

impl Inode {
    pub fn new_dir(ino: u64, uid: u32, gid: u32) -> Self {
        let now = SystemTime::now();
        Inode {
            ino,
            kind: FileType::Directory,
            attr: FileAttr {
                ino,
                size: 0,
                blocks: 0,
                atime: now,
                mtime: now,
                ctime: now,
                crtime: now,
                kind: FileType::Directory,
                perm: 0o755,
                nlink: 2,
                uid,
                gid,
                rdev: 0,
                flags: 0,
                blksize: 512,
            },
            data: InodeData::Directory { entries: HashMap::new() },
        }
    }
    pub fn new_file(ino: u64, uid: u32, gid: u32) -> Self {
        let now = SystemTime::now();
        Inode {
            ino,
            kind: FileType::RegularFile,
            attr: FileAttr {
                ino,
                size: 0,
                blocks: 0,
                atime: now,
                mtime: now,
                ctime: now,
                crtime: now,
                kind: FileType::RegularFile,
                perm: 0o644,
                nlink: 1,
                uid,
                gid,
                rdev: 0,
                flags: 0,
                blksize: 512,
            },
            data: InodeData::File { content: Vec::new() },
        }
    }
}

// ========== 全局状态 ==========

pub struct Hf3fsFuseOps {
    pub inodes: HashMap<u64, Inode>,
    pub next_ino: u64,
    // 这里存放你的后端状态、配置、客户端等
    // pub meta_client: MetaClient,
    // pub storage_client: StorageClient,
    // pub config: ...,
}

impl Hf3fsFuseOps {
    pub fn new() -> Self {
        let mut inodes = HashMap::new();
        // 创建根目录 inode
        let root_inode = Inode::new_dir(1, 0, 0);
        inodes.insert(1, root_inode);
        Self {
            inodes,
            next_ino: 2,
        }
    }
    // 分配新 inode 号
    pub fn alloc_ino(&mut self) -> u64 {
        let ino = self.next_ino;
        self.next_ino += 1;
        ino
    }
}

// ========== FUSE 操作实现 ==========

impl Filesystem for Hf3fsFuseOps {
    fn init(&mut self, _req: &Request, _config: &mut KernelConfig) -> Result<(), libc::c_int> {
        // 初始化文件系统（如挂载点、缓存等）
        Ok(())
    }

    fn destroy(&mut self) {
        // 卸载时的清理操作
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // 查找 parent 目录下的 name
        if let Some(parent_inode) = self.inodes.get(&parent) {
            if let InodeData::Directory { entries } = &parent_inode.data {
                if let Some(&child_ino) = entries.get(name) {
                    if let Some(child_inode) = self.inodes.get(&child_ino) {
                        let ttl = Duration::new(1, 0);
                        reply.entry(&ttl, &child_inode.attr, 0);
                        return;
                    }
                }
            }
        }
        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        if let Some(inode) = self.inodes.get(&ino) {
            let ttl = Duration::new(1, 0);
            reply.attr(&ttl, &inode.attr);
        } else {
            reply.error(ENOENT);
        }
    }

    fn setattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        // 设置 inode 属性
        // todo: 实现 setattr 逻辑
        reply.error(ENOSYS);
    }

    fn readlink(&mut self, _req: &Request, _ino: u64, reply: fuser::ReplyData) {
        // 读取符号链接
        // todo: 实现 readlink 逻辑
        reply.error(ENOSYS);
    }

    fn mknod(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _rdev: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        if (mode & libc::S_IFMT) != libc::S_IFREG {
            reply.error(ENOSYS);
            return;
        }

        if let Some(parent_inode) = self.inodes.get(&parent) {
            if let InodeData::Directory { entries } = &parent_inode.data {
                if entries.contains_key(name) {
                    reply.error(libc::EEXIST);
                    return;
                }
            } else {
                reply.error(libc::ENOTDIR);
                return;
            }
        } else {
            reply.error(ENOENT);
            return;
        }

        let ino = self.alloc_ino();
        let uid = _req.uid();
        let gid = _req.gid();
        let mut new_file = Inode::new_file(ino, uid, gid);
        new_file.attr.perm = mode as u16;

        let parent_inode = self.inodes.get_mut(&parent).unwrap();
        if let InodeData::Directory { entries } = &mut parent_inode.data {
            entries.insert(name.to_os_string(), ino);
        }
        self.inodes.insert(ino, new_file.clone());

        let ttl = Duration::new(1, 0);
        reply.entry(&ttl, &new_file.attr, 0);
    }

    fn mkdir(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        if let Some(parent_inode) = self.inodes.get(&parent) {
            if let InodeData::Directory { entries } = &parent_inode.data {
                if entries.contains_key(name) {
                    reply.error(libc::EEXIST);
                    return;
                }
            } else {
                reply.error(libc::ENOTDIR);
                return;
            }
        } else {
            reply.error(ENOENT);
            return;
        }

        let ino = self.alloc_ino();
        let uid = _req.uid();
        let gid = _req.gid();
        let mut new_dir = Inode::new_dir(ino, uid, gid);
        new_dir.attr.perm = mode as u16;

        let parent_inode = self.inodes.get_mut(&parent).unwrap();
        if let InodeData::Directory { entries } = &mut parent_inode.data {
            entries.insert(name.to_os_string(), ino);
        }
        self.inodes.insert(ino, new_dir.clone());

        let ttl = Duration::new(1, 0);
        reply.entry(&ttl, &new_dir.attr, 0);
    }

    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let child_ino = {
            if let Some(parent_inode) = self.inodes.get(&parent) {
                if let InodeData::Directory { entries } = &parent_inode.data {
                    entries.get(name).copied()
                } else { None }
            } else { None }
        };
        if let Some(ino) = child_ino {
            if let Some(parent_inode) = self.inodes.get_mut(&parent) {
                if let InodeData::Directory { entries } = &mut parent_inode.data {
                    entries.remove(name);
                }
            }
            self.inodes.remove(&ino);
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }

    fn rmdir(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let child_ino = {
            if let Some(parent_inode) = self.inodes.get(&parent) {
                if let InodeData::Directory { entries } = &parent_inode.data {
                    entries.get(name).copied()
                } else { None }
            } else { None }
        };
        if let Some(ino) = child_ino {
            let is_empty = {
                if let Some(child_inode) = self.inodes.get(&ino) {
                    if let InodeData::Directory { entries: child_entries } = &child_inode.data {
                        child_entries.is_empty()
                    } else { false }
                } else { false }
            };
            if is_empty {
                if let Some(parent_inode) = self.inodes.get_mut(&parent) {
                    if let InodeData::Directory { entries } = &mut parent_inode.data {
                        entries.remove(name);
                    }
                }
                self.inodes.remove(&ino);
                reply.ok();
            } else {
                reply.error(libc::ENOTEMPTY);
            }
        } else {
            reply.error(ENOENT);
        }
    }

    fn symlink(&mut self, _req: &Request, parent: u64, name: &OsStr, link: &std::path::Path, reply: ReplyEntry) {
        if let Some(parent_inode) = self.inodes.get(&parent) {
            if let InodeData::Directory { entries } = &parent_inode.data {
                if entries.contains_key(name) {
                    reply.error(libc::EEXIST);
                    return;
                }
            } else {
                reply.error(libc::ENOTDIR);
                return;
            }
        } else {
            reply.error(ENOENT);
            return;
        }

        let ino = self.alloc_ino();
        let now = SystemTime::now();
        let inode = Inode {
            ino,
            kind: FileType::Symlink,
            attr: FileAttr {
                ino,
                size: link.as_os_str().len() as u64,
                blocks: 0,
                atime: now,
                mtime: now,
                ctime: now,
                crtime: now,
                kind: FileType::Symlink,
                perm: 0o777,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
                blksize: 512,
            },
            data: InodeData::Symlink { target: link.to_path_buf() },
        };
        
        let parent_inode = self.inodes.get_mut(&parent).unwrap();
        if let InodeData::Directory { entries } = &mut parent_inode.data {
            entries.insert(name.to_os_string(), ino);
        }
        self.inodes.insert(ino, inode.clone());

        let ttl = Duration::new(1, 0);
        reply.entry(&ttl, &inode.attr, 0);
    }

    fn rename(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        _flags: u32,
        reply: ReplyEmpty,
    ) {
        let moved_ino = {
            if let Some(parent_inode) = self.inodes.get(&parent) {
                if let InodeData::Directory { entries } = &parent_inode.data {
                    entries.get(name).copied()
                } else { None }
            } else { None }
        };
        if let Some(ino) = moved_ino {
            if let Some(parent_inode) = self.inodes.get_mut(&parent) {
                if let InodeData::Directory { entries } = &mut parent_inode.data {
                    entries.remove(name);
                }
            }
            if let Some(newparent_inode) = self.inodes.get_mut(&newparent) {
                if let InodeData::Directory { entries: new_entries } = &mut newparent_inode.data {
                    new_entries.insert(newname.to_os_string(), ino);
                }
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }

    fn link(&mut self, _req: &Request, ino: u64, newparent: u64, newname: &OsStr, reply: ReplyEntry) {
        if let Some(newparent_inode) = self.inodes.get(&newparent) {
            if let InodeData::Directory { entries } = &newparent_inode.data {
                if entries.contains_key(newname) {
                    reply.error(libc::EEXIST);
                    return;
                }
            } else {
                reply.error(libc::ENOTDIR);
                return;
            }
        } else {
            reply.error(ENOENT);
            return;
        }

        if let Some(inode) = self.inodes.get(&ino) {
            let attr_copy = inode.attr;
            if let Some(newparent_inode) = self.inodes.get_mut(&newparent) {
                if let InodeData::Directory { entries } = &mut newparent_inode.data {
                    entries.insert(newname.to_os_string(), ino);
                }
            }
            let ttl = Duration::new(1, 0);
            reply.entry(&ttl, &attr_copy, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn open(&mut self, _req: &Request, _ino: u64, _flags: i32, reply: ReplyOpen) {
        // 打开文件
        // todo: 实现 open 逻辑
        reply.error(ENOSYS);
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        // 读取文件内容
        if let Some(inode) = self.inodes.get(&ino) {
            if let InodeData::File { content } = &inode.data {
                let offset = offset as usize;
                if offset >= content.len() {
                    reply.data(&[]);
                    return;
                }
                let end = std::cmp::min(offset + size as usize, content.len());
                reply.data(&content[offset..end]);
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        // 写入文件内容
        if let Some(inode) = self.inodes.get_mut(&ino) {
            if let InodeData::File { content } = &mut inode.data {
                let offset = offset as usize;
                if offset > content.len() {
                    content.resize(offset, 0);
                }
                if offset + data.len() > content.len() {
                    content.resize(offset + data.len(), 0);
                }
                content[offset..offset + data.len()].copy_from_slice(data);
                inode.attr.size = content.len() as u64;
                inode.attr.mtime = SystemTime::now();
                reply.written(data.len() as u32);
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn flush(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        reply.ok();
    }

    fn release(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        // 关闭文件（内存型无需处理）
        reply.ok();
    }

    fn fsync(&mut self, _req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: ReplyEmpty) {
        reply.ok();
    }

    fn opendir(&mut self, _req: &Request, ino: u64, _flags: i32, reply: ReplyOpen) {
        // 打开目录（内存型无需处理）
        if self.inodes.contains_key(&ino) {
            reply.opened(0, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if let Some(inode) = self.inodes.get(&ino) {
            if let InodeData::Directory { entries } = &inode.data {
                let mut idx = 0;
                if offset == 0 {
                    reply.add(ino, 1, FileType::Directory, ".");
                    reply.add(ino, 2, FileType::Directory, "..");
                    idx = 2;
                } else {
                    idx = offset as usize;
                }
                for (i, (name, &child_ino)) in entries.iter().enumerate().skip(idx - 2) {
                    if let Some(child_inode) = self.inodes.get(&child_ino) {
                        let full_offset = (i + 2) as i64;
                        if reply.add(child_inode.ino, full_offset, child_inode.kind, name) {
                            break;
                        }
                    }
                }
                reply.ok();
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn releasedir(&mut self, _req: &Request, _ino: u64, _fh: u64, _flags: i32, reply: ReplyEmpty) {
        // 关闭目录（内存型无需处理）
        reply.ok();
    }

    fn statfs(&mut self, _req: &Request, _ino: u64, reply: fuser::ReplyStatfs) {
        // 返回虚拟的文件系统信息
        reply.statfs(1024, 1024, 1024, 1000, 1000, 512, 255, 0);
    }

    fn setxattr(&mut self, _req: &Request, _ino: u64, _name: &OsStr, _value: &[u8], _flags: i32, _position: u32, reply: ReplyEmpty) {
        // 内存型暂不支持 xattr
        reply.error(ENOTSUP);
    }

    fn getxattr(&mut self, _req: &Request, _ino: u64, _name: &OsStr, _size: u32, reply: ReplyXattr) {
        reply.error(ENOTSUP);
    }

    fn listxattr(&mut self, _req: &Request, _ino: u64, _size: u32, reply: ReplyXattr) {
        reply.error(ENOTSUP);
    }

    fn removexattr(&mut self, _req: &Request, _ino: u64, _name: &OsStr, reply: ReplyEmpty) {
        reply.error(ENOTSUP);
    }

    fn create(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        if let Some(parent_inode) = self.inodes.get(&parent) {
            if let InodeData::Directory { entries } = &parent_inode.data {
                if entries.contains_key(name) {
                    reply.error(libc::EEXIST);
                    return;
                }
            } else {
                reply.error(libc::ENOTDIR);
                return;
            }
        } else {
            reply.error(ENOENT);
            return;
        }

        let ino = self.alloc_ino();
        let uid = _req.uid();
        let gid = _req.gid();
        let mut new_file = Inode::new_file(ino, uid, gid);
        new_file.attr.perm = mode as u16;

        let parent_inode = self.inodes.get_mut(&parent).unwrap();
        if let InodeData::Directory { entries } = &mut parent_inode.data {
            entries.insert(name.to_os_string(), ino);
        }
        self.inodes.insert(ino, new_file.clone());

        let ttl = Duration::new(1, 0);
        reply.created(&ttl, &new_file.attr, 0, 0, flags as u32);
    }

    fn ioctl(&mut self, _req: &Request, _ino: u64, _fh: u64, _flags: u32, _cmd: u32, _in_data: &[u8], _out_size: u32, reply: ReplyIoctl) {
        reply.error(ENOSYS);
    }

    fn readdirplus(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectoryPlus,
    ) {
        if let Some(inode) = self.inodes.get(&ino) {
            if let InodeData::Directory { entries } = &inode.data {
                let ttl = Duration::new(1, 0);
                let mut idx = 0;
                if offset == 0 {
                    reply.add(ino, 1, ".", &ttl, &inode.attr, 0);
                    reply.add(ino, 2, "..", &ttl, &inode.attr, 0);
                    idx = 2;
                } else {
                    idx = offset as usize;
                }
                for (i, (name, &child_ino)) in entries.iter().enumerate().skip(idx - 2) {
                    if let Some(child_inode) = self.inodes.get(&child_ino) {
                        let full_offset = (i + 2) as i64;
                        if reply.add(child_inode.ino, full_offset, name, &ttl, &child_inode.attr, 0) {
                            break;
                        }
                    }
                }
                reply.ok();
                return;
            }
        }
        reply.error(ENOENT);
    }
} 