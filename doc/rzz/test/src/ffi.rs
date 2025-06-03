use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::collections::HashMap;
use crate::fuse_launcher_config::{FuseLauncherConfig, KeyValue};

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