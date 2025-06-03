use std::os::raw::{c_char};
use std::ffi::{CStr, CString};
use std::ptr;
use anyhow::Context;

mod user_config;
use user_config::{UserConfig, meta};

// opaque 指针封装 Rust 对象
pub struct UserConfigHandle {
    inner: UserConfig,
}

// C 接口定义
#[no_mangle]
pub extern "C" fn user_config_new() -> *mut UserConfigHandle {
    let config = UserConfig::new();
    Box::into_raw(Box::new(UserConfigHandle { inner: config })) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn user_config_free(ptr: *mut UserConfigHandle) {
    if ptr.is_null() {
        return;
    }
    drop(Box::from_raw(ptr));
}

#[no_mangle]
pub unsafe extern "C" fn user_config_set_config(
    handle: *mut UserConfigHandle,
    key: *const c_char,
    val: *const c_char,
    uid: u32,
    gid: u32,
) -> *mut c_char {
    if handle.is_null() || key.is_null() || val.is_null() {
        return ptr::null_mut();
    }

    let key_str = CStr::from_ptr(key).to_string_lossy();
    let val_str = CStr::from_ptr(val).to_string_lossy();

    let ui = meta::UserInfo { uid, gid };

    match (*handle).inner.set_config(&key_str, &val_str, &ui) {
        Ok(inode) => CString::new(inode.data.symlink).unwrap().into_raw(),
        Err(e) => CString::new(format!("error: {}", e)).unwrap().into_raw(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn user_config_lookup_config(
    handle: *mut UserConfigHandle,
    key: *const c_char,
    uid: u32,
    gid: u32,
) -> *mut c_char {
    if handle.is_null() || key.is_null() {
        return ptr::null_mut();
    }

    let key_str = CStr::from_ptr(key).to_string_lossy();
    let ui = meta::UserInfo { uid, gid };

    match (*handle).inner.lookup_config(&key_str, &ui) {
        Ok(inode) => CString::new(inode.data.symlink).unwrap().into_raw(),
        Err(e) => CString::new(format!("error: {}", e)).unwrap().into_raw(),
    }
}