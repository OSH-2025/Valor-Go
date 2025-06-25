use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::collections::HashMap;
use crate::fuse_launcher_config::{FuseLauncherConfig, KeyValue};
use crate::mgmtd_client::MgmtdClient;
use crate::fuse_config_fetcher::FuseConfigFetcher;
use crate::fuse_ops::Hf3fsFuseOps;

// ========== fuse_launcher_config FFI ==========
#[no_mangle]
pub extern "C" fn complete_app_info(app_info: *const c_char) -> *mut c_char {
    let mut config = FuseLauncherConfig::new();

    if !app_info.is_null() {
        let app_info_str = unsafe { CStr::from_ptr(app_info).to_string_lossy().into_owned() };
        let updates_map: HashMap<String, String> = serde_json::from_str(&app_info_str).unwrap_or_default();
        let updates_vec: Vec<KeyValue> = updates_map
            .into_iter()
            .map(|(k, v)| KeyValue { key: k, value: v })
            .collect();
        config.init("", false, &updates_vec);
    }

    let result = serde_json::to_string(&config).unwrap();
    let c_result = CString::new(result).unwrap();
    c_result.into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn get_default_launcher_config() -> *mut c_char {
    let config = FuseLauncherConfig::default();
    let result = serde_json::to_string(&config).unwrap();
    CString::new(result).unwrap().into_raw()
}

// ========== mgmtd_client FFI ==========
static mut GLOBAL_MGMT: Option<MgmtdClient> = None;

#[no_mangle]
pub extern "C" fn mgmt_init(url: *const c_char) {
    let url = unsafe { CStr::from_ptr(url).to_string_lossy().into_owned() };
    unsafe { GLOBAL_MGMT = Some(MgmtdClient::new(&url)); }
}

#[no_mangle]
pub extern "C" fn mgmt_get_url(buffer: *mut c_char, max_len: usize) -> usize {
    unsafe {
        if let Some(mgmt) = &GLOBAL_MGMT {
            let url = &mgmt.mgmtd_service_url;
            let bytes = url.as_bytes();
            let len = bytes.len().min(max_len - 1);
            if !buffer.is_null() {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, len);
                *buffer.add(len) = 0;
            }
            return len;
        }
    }
    0
}

// ========== fuse_config_fetcher FFI ==========
static mut GLOBAL_FETCHER: Option<FuseConfigFetcher> = None;

#[no_mangle]
pub extern "C" fn fetcher_init(url: *const c_char) {
    let url = unsafe { CStr::from_ptr(url).to_string_lossy().into_owned() };
    unsafe { GLOBAL_FETCHER = Some(FuseConfigFetcher::new(&url)); }
}

#[no_mangle]
pub extern "C" fn fetcher_get_mgmt_url(buffer: *mut c_char, max_len: usize) -> usize {
    unsafe {
        if let Some(fetcher) = &GLOBAL_FETCHER {
            let url = &fetcher.mgmtd_client.mgmtd_service_url;
            let bytes = url.as_bytes();
            let len = bytes.len().min(max_len - 1);
            if !buffer.is_null() {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, len);
                *buffer.add(len) = 0;
            }
            return len;
        }
    }
    0
}

// ========== fuse_ops FFI ==========
static mut GLOBAL_FS: Option<Hf3fsFuseOps> = None;

#[no_mangle]
pub extern "C" fn fs_init() {
    unsafe { GLOBAL_FS = Some(Hf3fsFuseOps::new()); }
}

#[no_mangle]
pub extern "C" fn fs_create_file(name: *const c_char) -> u64 {
    unsafe {
        if let Some(fs) = &mut GLOBAL_FS {
            let _name = CStr::from_ptr(name).to_string_lossy().into_owned();
            // 这里只是演示，实际应调用 fuse_ops 的创建逻辑
            let ino = fs.alloc_ino();
            // 省略插入到根目录的逻辑
            return ino as u64;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn fs_create_file_in_root(name: *const c_char) -> u64 {
    unsafe {
        if let Some(fs) = &mut GLOBAL_FS {
            let name = CStr::from_ptr(name).to_string_lossy().into_owned();
            let ino = fs.alloc_ino();
            if let Some(root) = fs.inodes.get_mut(&1) {
                if let crate::fuse_ops::InodeData::Directory { entries } = &mut root.data {
                    entries.insert(name.into(), ino);
                }
            }
            // 实际还应插入 inode
            return ino as u64;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn fs_list_root_files(buffer: *mut u64, max: usize) -> usize {
    unsafe {
        if let Some(fs) = &GLOBAL_FS {
            if let Some(root) = fs.inodes.get(&1) {
                if let crate::fuse_ops::InodeData::Directory { entries } = &root.data {
                    let mut count = 0;
                    for (_name, &ino) in entries.iter().take(max) {
                        if !buffer.is_null() {
                            *buffer.add(count) = ino;
                        }
                        count += 1;
                    }
                    return count;
                }
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn fs_get_file_size(ino: u64) -> u64 {
    unsafe {
        if let Some(fs) = &GLOBAL_FS {
            if let Some(inode) = fs.inodes.get(&ino) {
                return inode.attr.size;
            }
        }
    }
    0
}

// ========== fuse_main_loop FFI ==========
// 这里只做演示，实际挂载 FUSE 需更复杂的参数和生命周期管理
#[no_mangle]
pub extern "C" fn fuse_main_loop_run_stub() -> i32 {
    // 这里只是演示，实际应调用 fuse_main_loop 的挂载逻辑
    0 // 返回0表示成功
}