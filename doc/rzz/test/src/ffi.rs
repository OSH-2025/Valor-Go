use crate::fuse_config_fetcher::FuseConfigFetcher;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

#[no_mangle]
pub extern "C" fn complete_app_info(app_info: *mut c_char) -> *mut c_char {
    let fetcher = FuseConfigFetcher;
    let mut app_info_builder = flatbuffers::FlatBufferBuilder::new();

    match tokio::runtime::Runtime::new().unwrap().block_on(fetcher.complete_app_info(&mut app_info_builder)) {
        Ok(_) => {
            let c_str = CString::new("Success").unwrap();
            c_str.into_raw()
        }
        Err(e) => {
            let c_str = CString::new(e).unwrap();
            c_str.into_raw()
        }
    }
}