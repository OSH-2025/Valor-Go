// FFI 接口层，提供 C 接口供 C++ 代码调用
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::default::Default;

use crate::FuseConfig::FuseConfig;
use crate::FuseClients::FuseClients;
use crate::fuse_main_loop_rs;
use crate::FuseAppConfig::ConfigBase;
use crate::FuseAppConfig::FuseAppConfig;
use crate::FuseApplication::FuseApplication;

#[no_mangle]
pub extern "C" fn hf3fs_fuse_init(
    config_path: *const c_char,
    mountpoint: *const c_char,
    token_file: *const c_char,
) -> c_int {
    if config_path.is_null() || mountpoint.is_null() || token_file.is_null() {
        return -1;
    }

    let config_path_str = unsafe { CStr::from_ptr(config_path).to_string_lossy() };
    let mountpoint_str = unsafe { CStr::from_ptr(mountpoint).to_string_lossy() };
    let token_file_str = unsafe { CStr::from_ptr(token_file).to_string_lossy() };

    // 加载配置
    let mut config: FuseConfig = Default::default();
    if let Err(_) = config.init(&*config_path_str, false, vec![]) {
        return -2;
    }

    // 初始化 FUSE 客户端
    let mut clients: FuseClients = Default::default();
    let app_config = FuseAppConfig::new();
    let app = FuseApplication::new();
    if !clients.init(&app_config, &app) {
        return -3;
    }

    0
}

#[no_mangle]
pub extern "C" fn hf3fs_fuse_run(
    allow_other: c_int,
    maxbufsize: usize,
    cluster_id: *const c_char,
) -> c_int {
    if cluster_id.is_null() {
        return -1;
    }

    let cluster_id_str = unsafe { CStr::from_ptr(cluster_id).to_string_lossy() };
    
    // 运行 FUSE 主循环
    match fuse_main_loop_rs(
        "hf3fs_fuse".to_string(),
        allow_other != 0,
        "/mnt".to_string(), // 这里可以从配置中获取
        maxbufsize,
        cluster_id_str.to_string(),
    ) {
        Ok(_) => 0,
        Err(e) => e,
    }
}

#[no_mangle]
pub extern "C" fn hf3fs_fuse_cleanup() {
    let mut clients: FuseClients = Default::default();
    clients.stop();
}

// 配置相关 FFI 接口
#[no_mangle]
pub extern "C" fn hf3fs_fuse_get_config() -> *mut c_void {
    // 返回配置指针，C++ 代码可以通过其他接口访问
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn hf3fs_fuse_set_user_config(
    uid: u64,
    key: *const c_char,
    value: *const c_char,
) -> c_int {
    if key.is_null() || value.is_null() {
        return -1;
    }

    let key_str = unsafe { CStr::from_ptr(key).to_string_lossy() };
    let value_str = unsafe { CStr::from_ptr(value).to_string_lossy() };

    // 这里可以调用 UserConfig 的 set_config 方法
    println!("Setting config for uid {}: {} = {}", uid, key_str, value_str);
    
    0
} 