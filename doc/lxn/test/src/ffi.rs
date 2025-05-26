
use std::os::raw::{c_char, c_void};
use std::ffi::{CStr, CString};

// 正确导入 FuseAppConfig 和 KeyValue
use crate::fuse_app_config::{FuseAppConfig, KeyValue};

/// opaque 结构体（用于 C++ 拿到指针）
#[repr(C)]
pub struct FuseAppConfigWrapper {
    _private: c_void,
}

/// 创建对象
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
    updates: *const KeyValue,
    update_len: usize,
) {
    if config.is_null() || file_path.is_null() || (updates.is_null() && update_len > 0) {
        return;
    }

    let file_path = unsafe { CStr::from_ptr(file_path).to_str().unwrap() };
    let updates = unsafe { std::slice::from_raw_parts(updates, update_len) };

    unsafe {
        (*config).init(file_path, dump, updates.to_vec());
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
}/*
use std::os::raw::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::collections::HashMap;

// 导入 FuseAppConfig 及其 KeyValue
use crate::fuse_app_config::{FuseAppConfig, KeyValue as FuseKeyValue};

// 导入 FuseApplication 模块
use crate::fuse_application::{ApplicationBase, FuseApplication, AppInfo};

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
    updates: *const FuseKeyValue,
    update_len: usize,
) {
    if config.is_null() || file_path.is_null() || (updates.is_null() && update_len > 0) {
        return;
    }

    let file_path = unsafe { CStr::from_ptr(file_path).to_str().unwrap() };
    let updates_slice = unsafe { std::slice::from_raw_parts(updates, update_len) };

    let rust_updates = updates_slice
        .iter()
        .map(|kv| KeyValue {
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

/// FuseApplication opaque 结构体
#[repr(C)]
pub struct FuseApplication {
    _private: c_void,
}

/// 创建 FuseApplication 对象
#[no_mangle]
pub extern "C" fn fuse_application_new() -> *mut FuseApplication {
    let app = super::fuse_application::FuseApplication::new();
    Box::into_raw(Box::new(app)) as *mut _
}

/// 解析命令行参数
#[no_mangle]
pub extern "C" fn fuse_application_parse_flags(
    app: &mut FuseApplication,
    argc: c_int,
    argv: *const *const c_char,
) -> Result<(), String> {
    if argc <= 0 || argv.is_null() {
        return Ok(());
    }

    let args: Vec<String> = unsafe {
        std::slice::from_raw_parts(argv, argc as usize)
            .iter()
            .map(|&ptr| CStr::from_ptr(ptr).to_string_lossy().into_owned())
            .collect()
    };

    unsafe {
        ApplicationBase::parse_flags(app as *mut _ as *mut dyn ApplicationBase, argc, args)
            .map_err(|e| e.to_string())
    }
}

/// 初始化应用
#[no_mangle]
pub extern "C" fn fuse_application_init(app: &mut FuseApplication) -> Result<(), String> {
    ApplicationBase::init_application(app as *mut _ as *mut dyn ApplicationBase)
        .map_err(|e| e.to_string())
}

/// 启动主循环
#[no_mangle]
pub extern "C" fn fuse_application_main_loop(app: &FuseApplication) -> c_int {
    ApplicationBase::main_loop(app as *const _ as *const dyn ApplicationBase)
}

/// 停止应用
#[no_mangle]
pub extern "C" fn fuse_application_stop(app: &mut FuseApplication) {
    ApplicationBase::stop(app as *mut _ as *mut dyn ApplicationBase)
}

/// 获取 Node ID
#[no_mangle]
pub extern "C" fn fuse_application_get_node_id(app: &FuseApplication) -> u64 {
    match ApplicationBase::info(app as *const _ as *const dyn ApplicationBase) {
        Some(info) => info.node_id,
        None => 0,
    }
}

/// 配置更新回调
#[no_mangle]
pub extern "C" fn fuse_application_on_config_updated(app: &FuseApplication) {
    ApplicationBase::on_config_updated(app as *const _ as *const dyn ApplicationBase)
}

/// 释放资源
#[no_mangle]
pub extern "C" fn fuse_application_free(app: *mut FuseApplication) {
    if !app.is_null() {
        unsafe {
            Box::from_raw(app);
        }
    }
}*/