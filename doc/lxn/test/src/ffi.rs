// src/fuse_clients/ffi.rs
use std::os::raw::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::collections::HashMap;

use crate::fuse_clients::{FuseClients as RustFuseClients, FuseConfig};
use crate::fuse_app_config::{FuseAppConfig, KeyValue as FuseKeyValue};
use crate::fuse_application::{ApplicationBase, FuseApplication as OtherFuseApplication, AppInfo};

#[repr(C)]
pub struct FuseClients {
    _private: c_void,
}

#[no_mangle]
pub extern "C" fn fuse_clients_new(mount_point: *const c_char, token: *const c_char) -> *mut FuseClients {
    let mount_point = unsafe { CStr::from_ptr(mount_point).to_string_lossy().into_owned() };
    let token = unsafe { CStr::from_ptr(token).to_string_lossy().into_owned() };

    let config = FuseConfig::default();
    Box::into_raw(Box::new(RustFuseClients::new(config, mount_point, token))) as *mut _
}

#[no_mangle]
pub extern "C" fn fuse_clients_start(clients: *mut FuseClients) {
    if !clients.is_null() {
        unsafe {
            let clients_ref = &mut *(clients as *mut RustFuseClients);
            // 使用 Tokio runtime 来运行 async 函数
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(clients_ref.start());
        }
    }
}

#[no_mangle]
pub extern "C" fn fuse_clients_stop(clients: *mut FuseClients) {
    if !clients.is_null() {
        unsafe {
            let clients_ref = &*(clients as *mut RustFuseClients);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(clients_ref.stop());
        }
    }
}

#[no_mangle]
pub extern "C" fn fuse_clients_periodic_sync_scan(clients: *mut FuseClients) {
    if !clients.is_null() {
        unsafe {
            let clients_ref = &*(clients as *mut RustFuseClients);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(clients_ref.periodic_sync_scan());
        }
    }
}

#[no_mangle]
pub extern "C" fn fuse_clients_free(clients: *mut FuseClients) {
    if !clients.is_null() {
        unsafe {
            Box::from_raw(clients);
        }
    }
}

/// opaque 结构体（用于 C++ 拿到指针）
#[repr(C)]
pub struct FuseAppConfigWrapper {
    _private: c_void,
}

/// 创建 FuseAppConfig 对象
#[no_mangle]
pub extern "C" fn fuse_app_config_new() -> *mut FuseAppConfig {
    Box::into_raw(Box::new(FuseAppConfig::new()))
}

/// 初始化配置
#[no_mangle]
pub extern "C" fn fuse_app_config_init(
    config: *mut FuseAppConfig,
    file_path: *const c_char,
    dump: bool,
    updates: *const FFIKeyValue,
    update_len: usize,
) {
    if config.is_null() || file_path.is_null() || (updates.is_null() && update_len > 0) {
        return;
    }

    let file_path = unsafe { CStr::from_ptr(file_path).to_str().unwrap() };
    let updates_slice = unsafe { std::slice::from_raw_parts(updates, update_len) };

    let rust_updates = updates_slice
        .iter()
        .map(|kv| FuseKeyValue {
            key: unsafe { CStr::from_ptr(kv.key).to_string_lossy().into_owned() },
            value: unsafe { CStr::from_ptr(kv.value).to_string_lossy().into_owned() },
        })
        .collect();

    unsafe {
        (*config).init(file_path, dump, rust_updates);
    }
}

/// 销毁对象
#[no_mangle]
pub extern "C" fn fuse_app_config_free(config: *mut FuseAppConfig) {
    if !config.is_null() {
        unsafe {
            Box::from_raw(config);
        }
    }
}

/// AppInfo 结构体，供 C++ 获取 node_id
#[repr(C)]
pub struct FFIFuseAppInfo {
    pub node_id: u64,
    pub hostname: *const c_char,
}

/// 将重复定义的FuseApplication重命名为FuseApplicationFFI
pub struct FuseApplicationFFI {
    _private: c_void,
}

/// 创建 FuseApplication 对象
#[no_mangle]
pub extern "C" fn fuse_application_new() -> *mut FuseApplicationFFI {
    let app = super::fuse_application::FuseApplication::new();
    Box::into_raw(Box::new(app)) as *mut _
}

/// 解析命令行参数
#[no_mangle]
pub extern "C" fn fuse_application_parse_flags(
    app: *mut FuseApplicationFFI,
    argc: c_int,
    argv: *const *const c_char,
) -> Result<(), String> {
    if app.is_null() || argc <= 0 || argv.is_null() {
        return Ok(());
    }

    let args: Vec<String> = unsafe {
        std::slice::from_raw_parts(argv, argc as usize)
            .iter()
            .map(|&ptr| CStr::from_ptr(ptr).to_string_lossy().into_owned())
            .collect()
    };

    let app_ref = unsafe { &mut *(app as *mut OtherFuseApplication) };
    ApplicationBase::parse_flags(app_ref, argc, args)
        .map_err(|e| e.to_string())
}

/// 初始化应用
#[no_mangle]
pub extern "C" fn fuse_application_init(app: *mut FuseApplicationFFI) -> Result<(), String> {
    if app.is_null() {
        return Err("Null pointer".to_string());
    }
    let app_ref = unsafe { &mut *(app as *mut OtherFuseApplication) };
    ApplicationBase::init_application(app_ref)
        .map_err(|e| e.to_string())
}

/// 启动主循环
#[no_mangle]
pub extern "C" fn fuse_application_main_loop(app: *const FuseApplicationFFI) -> c_int {
    if app.is_null() {
        return -1;
    }
    let app_ref = unsafe { &*(app as *const OtherFuseApplication) };
    ApplicationBase::main_loop(app_ref)
}

/// 停止应用
#[no_mangle]
pub extern "C" fn fuse_application_stop(app: *mut FuseApplicationFFI) {
    if app.is_null() {
        return;
    }
    let app_ref = unsafe { &mut *(app as *mut OtherFuseApplication) };
    ApplicationBase::stop(app_ref);
}

/// 获取 Node ID
#[no_mangle]
pub extern "C" fn fuse_application_get_node_id(app: *const FuseApplicationFFI) -> u64 {
    if app.is_null() {
        return 0;
    }
    let app_ref = unsafe { &*(app as *const OtherFuseApplication) };
    match ApplicationBase::info(app_ref) {
        Some(info) => info.node_id,
        None => 0,
    }
}

/// 配置更新回调
#[no_mangle]
pub extern "C" fn fuse_application_on_config_updated(app: *const FuseApplicationFFI) {
    if app.is_null() {
        return;
    }
    let app_ref = unsafe { &*(app as *const OtherFuseApplication) };
    ApplicationBase::on_config_updated(app_ref);
}

/// 释放资源
#[no_mangle]
pub extern "C" fn fuse_application_free(app: *mut FuseApplicationFFI) {
    if !app.is_null() {
        unsafe {
            Box::from_raw(app);
        }
    }
}

#[repr(C)]
pub struct FFIKeyValue {
    pub key: *const c_char,
    pub value: *const c_char,
}