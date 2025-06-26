// // FFI 接口层，提供 C 接口供 C++ 代码调用
// use std::ffi::{CStr, CString};
// use std::os::raw::{c_char, c_int, c_void, c_ulong, c_ulonglong};
// use std::ptr;
// use std::default::Default;
// use std::sync::OnceLock;
// use std::sync::Arc;
// use std::path::Path;

// use crate::FuseConfig::FuseConfig;
// use crate::FuseClients::FuseClients;
// use crate::fuse_main_loop_rs;
// use crate::FuseAppConfig::ConfigBase;
// use crate::FuseAppConfig::FuseAppConfig;
// use crate::FuseApplication::FuseApplication;
// use crate::FuseAppConfig::KeyValue;
// use crate::FuseConfigFetcher::FuseConfigFetcher;
// use crate::FuseLauncherConfig::{FuseLauncherConfig, KeyValue};
// use crate::IoRing::{IoRing, IoArgs, IoSqe, IoCqe};
// use crate::IovTable::{IovTable, UserInfo as IovUserInfo};
// use crate::PioV::PioV;
// use crate::UserConfig::UserConfig;

// #[repr(C)]
// pub struct FfiKeyValue {
//     key: *const c_char,
//     value: *const c_char,
// }

// #[repr(C)]
// pub struct FfiIoArgs {
//     file_id: u64,
//     file_off: u64,
//     io_len: usize,
//     buf: *mut u8,
//     buf_len: usize,
//     userdata: u64,
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_init(
//     config_path: *const c_char,
//     mountpoint: *const c_char,
//     token_file: *const c_char,
// ) -> c_int {
//     if config_path.is_null() || mountpoint.is_null() || token_file.is_null() {
//         return -1;
//     }

//     let config_path_str = unsafe { CStr::from_ptr(config_path).to_string_lossy() };
//     let mountpoint_str = unsafe { CStr::from_ptr(mountpoint).to_string_lossy() };
//     let token_file_str = unsafe { CStr::from_ptr(token_file).to_string_lossy() };

//     // 加载配置
//     let mut config: FuseConfig = Default::default();
//     if let Err(_) = config.init(&*config_path_str, false, vec![]) {
//         return -2;
//     }

//     // 初始化 FUSE 客户端
//     let mut clients: FuseClients = Default::default();
//     let app_config = FuseAppConfig::new();
//     let app = FuseApplication::new();
//     if !clients.init(&app_config, &app) {
//         return -3;
//     }

//     0
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_run(
//     allow_other: c_int,
//     maxbufsize: usize,
//     cluster_id: *const c_char,
// ) -> c_int {
//     if cluster_id.is_null() {
//         return -1;
//     }

//     let cluster_id_str = unsafe { CStr::from_ptr(cluster_id).to_string_lossy() };
    
//     // 运行 FUSE 主循环
//     match fuse_main_loop_rs(
//         "hf3fs_fuse".to_string(),
//         allow_other != 0,
//         "/mnt".to_string(), // 这里可以从配置中获取
//         maxbufsize,
//         cluster_id_str.to_string(),
//     ) {
//         Ok(_) => 0,
//         Err(e) => e,
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_cleanup() {
//     let mut clients: FuseClients = Default::default();
//     clients.stop();
// }

// // 配置相关 FFI 接口
// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_get_config() -> *mut c_void {
//     // 返回配置指针，C++ 代码可以通过其他接口访问
//     ptr::null_mut()
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_set_user_config(
//     uid: u64,
//     key: *const c_char,
//     value: *const c_char,
// ) -> c_int {
//     if key.is_null() || value.is_null() {
//         return -1;
//     }

//     let key_str = unsafe { CStr::from_ptr(key).to_string_lossy() };
//     let value_str = unsafe { CStr::from_ptr(value).to_string_lossy() };

//     // 这里可以调用 UserConfig 的 set_config 方法
//     println!("Setting config for uid {}: {} = {}", uid, key_str, value_str);
    
//     0
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_app_config_init(
//     config: *mut FuseAppConfig,
//     file_path: *const c_char,
//     dump: bool,
//     updates: *const FfiKeyValue,
//     updates_len: usize,
// ) -> c_int {
//     if config.is_null() || file_path.is_null() {
//         return -1;
//     }
//     let file_path = unsafe { CStr::from_ptr(file_path).to_string_lossy().to_string() };
//     let mut updates_vec = Vec::new();
//     if !updates.is_null() && updates_len > 0 {
//         let slice = unsafe { std::slice::from_raw_parts(updates, updates_len) };
//         for ffi_kv in slice {
//             if ffi_kv.key.is_null() || ffi_kv.value.is_null() {
//                 continue;
//             }
//             let key = unsafe { CStr::from_ptr(ffi_kv.key).to_string_lossy().to_string() };
//             let value = unsafe { CStr::from_ptr(ffi_kv.value).to_string_lossy().to_string() };
//             updates_vec.push(KeyValue::new(key, value));
//         }
//     }
//     let config = unsafe { &mut *config };
//     config.init(&file_path, dump, updates_vec);
//     0
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_new() -> *mut FuseApplication {
//     Box::into_raw(Box::new(FuseApplication::new()))
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_drop(ptr: *mut FuseApplication) {
//     if !ptr.is_null() {
//         unsafe { Box::from_raw(ptr); }
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_parse_flags(
//     app: *mut FuseApplication,
//     argc: c_int,
//     argv: *const *const c_char,
// ) -> c_int {
//     if app.is_null() || argv.is_null() { return -1; }
//     let args: Vec<String> = (0..argc)
//         .map(|i| unsafe { CStr::from_ptr(*argv.offset(i as isize)).to_string_lossy().to_string() })
//         .collect();
//     let app = unsafe { &mut *app };
//     match app.parse_flags(argc, args) {
//         Ok(_) => 0,
//         Err(_) => -2,
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_init_application(app: *mut FuseApplication) -> c_int {
//     if app.is_null() { return -1; }
//     let app = unsafe { &mut *app };
//     match app.init_application() {
//         Ok(_) => 0,
//         Err(_) => -2,
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_stop(app: *mut FuseApplication) {
//     if app.is_null() { return; }
//     let app = unsafe { &mut *app };
//     app.stop();
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_main_loop(app: *mut FuseApplication) -> c_int {
//     if app.is_null() { return -1; }
//     let app = unsafe { &*app };
//     app.main_loop()
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_get_config(app: *mut FuseApplication) -> *const c_void {
//     if app.is_null() { return std::ptr::null(); }
//     let app = unsafe { &*app };
//     app.get_config() as *const _ as *const c_void
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_info(app: *mut FuseApplication) -> *const c_void {
//     if app.is_null() { return std::ptr::null(); }
//     let app = unsafe { &*app };
//     match app.info() {
//         Some(info) => info as *const _ as *const c_void,
//         None => std::ptr::null(),
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_config_pushable(app: *mut FuseApplication) -> bool {
//     if app.is_null() { return false; }
//     let app = unsafe { &*app };
//     app.config_pushable()
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_application_on_config_updated(app: *mut FuseApplication) {
//     if app.is_null() { return; }
//     let app = unsafe { &*app };
//     app.on_config_updated();
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_clients_new() -> *mut FuseClients {
//     Box::into_raw(Box::new(FuseClients::new()))
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_clients_drop(ptr: *mut FuseClients) {
//     if !ptr.is_null() {
//         unsafe { Box::from_raw(ptr); }
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_clients_init(
//     clients: *mut FuseClients,
//     config: *const FuseAppConfig,
//     app: *const FuseApplication,
// ) -> bool {
//     if clients.is_null() || config.is_null() || app.is_null() { return false; }
//     let clients = unsafe { &mut *clients };
//     let config = unsafe { &*config };
//     let app = unsafe { &*app };
//     clients.init(config, app)
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_clients_stop(clients: *mut FuseClients) {
//     if clients.is_null() { return; }
//     let clients = unsafe { &mut *clients };
//     clients.stop();
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_config_fetcher_new(mgmtd_service_url: *const c_char) -> *mut FuseConfigFetcher {
//     if mgmtd_service_url.is_null() { return std::ptr::null_mut(); }
//     let url = unsafe { CStr::from_ptr(mgmtd_service_url).to_string_lossy().to_string() };
//     Box::into_raw(Box::new(FuseConfigFetcher::new(&url)))
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_config_fetcher_drop(ptr: *mut FuseConfigFetcher) {
//     if !ptr.is_null() {
//         unsafe { Box::from_raw(ptr); }
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_config_fetcher_complete_app_info(
//     fetcher: *mut FuseConfigFetcher,
//     app_info: *mut c_void,
// ) -> c_int {
//     if fetcher.is_null() || app_info.is_null() { return -1; }
//     let _fetcher = unsafe { &mut *fetcher };
//     // TODO: 这里需要你适配app_info的类型，并实现异步调用
//     0
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_launcher_config_init(
//     config: *mut FuseLauncherConfig,
//     file_path: *const c_char,
//     dump: bool,
//     updates: *const FfiKeyValue,
//     updates_len: usize,
// ) -> c_int {
//     if config.is_null() || file_path.is_null() {
//         return -1;
//     }
//     let mut updates_vec = Vec::new();
//     if !updates.is_null() && updates_len > 0 {
//         let slice = unsafe { std::slice::from_raw_parts(updates, updates_len) };
//         for ffi_kv in slice {
//             if ffi_kv.key.is_null() || ffi_kv.value.is_null() {
//                 continue;
//             }
//             let key = unsafe { CStr::from_ptr(ffi_kv.key).to_string_lossy().to_string() };
//             let value = unsafe { CStr::from_ptr(ffi_kv.value).to_string_lossy().to_string() };
//             updates_vec.push(KeyValue { key, value });
//         }
//     }
//     let file_path = unsafe { CStr::from_ptr(file_path).to_string_lossy().to_string() };
//     let config = unsafe { &mut *config };
//     config.init(&file_path, dump, &updates_vec);
//     0
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_main_loop(
//     program_name: *const c_char,
//     allow_other: bool,
//     mountpoint: *const c_char,
//     maxbufsize: c_ulong,
//     cluster_id: *const c_char,
// ) -> c_int {
//     if program_name.is_null() || mountpoint.is_null() || cluster_id.is_null() {
//         return -1;
//     }
//     let program_name = unsafe { CStr::from_ptr(program_name).to_string_lossy().to_string() };
//     let mountpoint = unsafe { CStr::from_ptr(mountpoint).to_string_lossy().to_string() };
//     let cluster_id = unsafe { CStr::from_ptr(cluster_id).to_string_lossy().to_string() };
//     match crate::FuseMainLoop::fuse_main_loop_rs(
//         program_name,
//         allow_other,
//         mountpoint,
//         maxbufsize as usize,
//         cluster_id,
//     ) {
//         Ok(_) => 0,
//         Err(e) => e,
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_fuse_clients_instance() -> *mut FuseClients {
//     static INSTANCE: OnceLock<Box<FuseClients>> = OnceLock::new();
//     INSTANCE.get_or_init(|| Box::new(FuseClients::new())).as_ref() as *const _ as *mut FuseClients
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_ioring_new(
//     name: *const c_char,
//     entries: usize,
//     io_depth: usize,
//     priority: usize,
//     timeout_ms: u64,
//     for_read: bool,
// ) -> *mut IoRing {
//     if name.is_null() { return std::ptr::null_mut(); }
//     let name = unsafe { CStr::from_ptr(name).to_string_lossy().to_string() };
//     let ring = IoRing::new(&name, entries, io_depth, priority, std::time::Duration::from_millis(timeout_ms), for_read);
//     Arc::into_raw(ring) as *mut IoRing
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_ioring_add_sqe(ring: *mut IoRing, args: FfiIoArgs) -> bool {
//     if ring.is_null() { return false; }
//     let ring = unsafe { Arc::from_raw(ring) };
//     let buf = unsafe { std::slice::from_raw_parts_mut(args.buf, args.buf_len) };
//     let io_args = IoArgs {
//         file_id: args.file_id,
//         file_off: args.file_off,
//         io_len: args.io_len,
//         buf: buf.to_vec(),
//         userdata: Some(args.userdata),
//     };
//     let res = ring.add_sqe(io_args);
//     std::mem::forget(ring);
//     res
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_iovtable_new(mount_name: *const c_char) -> *mut IovTable {
//     if mount_name.is_null() { return std::ptr::null_mut(); }
//     let mount_name = unsafe { CStr::from_ptr(mount_name).to_string_lossy().to_string() };
//     Box::into_raw(Box::new(IovTable::new(&mount_name)))
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_iovtable_drop(ptr: *mut IovTable) {
//     if !ptr.is_null() {
//         unsafe { Box::from_raw(ptr); }
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_iovtable_add_iov(
//     table: *mut IovTable,
//     key: *const c_char,
//     shm_path: *const c_char,
//     uid: u32,
//     gid: u32,
// ) -> *mut c_void {
//     if table.is_null() || key.is_null() || shm_path.is_null() { return std::ptr::null_mut(); }
//     let table = unsafe { &mut *table };
//     let key = unsafe { CStr::from_ptr(key).to_string_lossy().to_string() };
//     let shm_path = unsafe { CStr::from_ptr(shm_path).to_string_lossy().to_string() };
//     let user = IovUserInfo { uid, gid };
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     match rt.block_on(table.add_iov(&key, Path::new(&shm_path), &user)) {
//         Ok(shmbuf) => Arc::into_raw(shmbuf) as *mut c_void,
//         Err(_) => std::ptr::null_mut(),
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_iovtable_rm_iov(
//     table: *mut IovTable,
//     key: *const c_char,
//     uid: u32,
//     gid: u32,
// ) -> *mut c_void {
//     if table.is_null() || key.is_null() { return std::ptr::null_mut(); }
//     let table = unsafe { &mut *table };
//     let key = unsafe { CStr::from_ptr(key).to_string_lossy().to_string() };
//     let user = IovUserInfo { uid, gid };
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     match rt.block_on(table.rm_iov(&key, &user)) {
//         Ok(shmbuf) => Arc::into_raw(shmbuf) as *mut c_void,
//         Err(_) => std::ptr::null_mut(),
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_iovtable_lookup_iov(
//     table: *mut IovTable,
//     key: *const c_char,
//     uid: u32,
//     gid: u32,
// ) -> *mut c_void {
//     if table.is_null() || key.is_null() { return std::ptr::null_mut(); }
//     let table = unsafe { &mut *table };
//     let key = unsafe { CStr::from_ptr(key).to_string_lossy().to_string() };
//     let user = IovUserInfo { uid, gid };
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     match rt.block_on(table.lookup_iov(&key, &user)) {
//         Ok(shmbuf) => Arc::into_raw(shmbuf) as *mut c_void,
//         Err(_) => std::ptr::null_mut(),
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_iovtable_list_iovs(
//     table: *mut IovTable,
//     uid: u32,
//     gid: u32,
//     out_len: *mut usize,
// ) -> *mut *mut c_void {
//     if table.is_null() || out_len.is_null() { return std::ptr::null_mut(); }
//     let table = unsafe { &mut *table };
//     let user = IovUserInfo { uid, gid };
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     let list = rt.block_on(table.list_iovs(&user));
//     unsafe { *out_len = list.len(); }
//     let mut ptrs: Vec<*mut c_void> = list.into_iter().map(|shmbuf| Arc::into_raw(shmbuf) as *mut c_void).collect();
//     let ptr = ptrs.as_mut_ptr();
//     std::mem::forget(ptrs);
//     ptr
// }

// // ========== PioV FFI ==========
// #[no_mangle]
// pub extern "C" fn hf3fs_piov_new(_storage_client: *mut c_void, _chunk_size_lim: i32, _res_vec: *mut c_void, _res_len: usize) -> *mut PioV<'static> {
//     // TODO: 需要适配storage_client和res_vec
//     std::ptr::null_mut()
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_piov_drop(_ptr: *mut PioV) {
//     // TODO: 实现析构
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_piov_add_read(
//     _piov: *mut PioV,
//     _idx: usize,
//     _inode: *const c_void,
//     _track: u16,
//     _off: isize,
//     _len: usize,
//     _buf: *mut c_void,
//     _memh: *mut c_void,
// ) -> i32 {
//     // TODO: 参数和结构体 glue
//     -1
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_piov_add_write(
//     _piov: *mut PioV,
//     _idx: usize,
//     _inode: *const c_void,
//     _track: u16,
//     _off: isize,
//     _len: usize,
//     _buf: *const c_void,
//     _memh: *mut c_void,
// ) -> i32 {
//     // TODO: 参数和结构体 glue
//     -1
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_piov_execute_read(
//     _piov: *mut PioV,
//     _user_info: *const c_void,
//     _options: *const c_void,
// ) -> i32 {
//     // TODO: 参数和结构体 glue
//     -1
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_piov_execute_write(
//     _piov: *mut PioV,
//     _user_info: *const c_void,
//     _options: *const c_void,
// ) -> i32 {
//     // TODO: 参数和结构体 glue
//     -1
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_piov_finish_io(_piov: *mut PioV, _allow_holes: bool) {
//     // TODO: 参数和结构体 glue
// }

// // ========== UserConfig FFI ==========
// #[no_mangle]
// pub extern "C" fn hf3fs_user_config_new() -> *mut UserConfig {
//     Box::into_raw(Box::new(UserConfig::new()))
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_user_config_drop(ptr: *mut UserConfig) {
//     if !ptr.is_null() {
//         unsafe { Box::from_raw(ptr); }
//     }
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_user_config_init(_uc: *mut UserConfig, _fuse_config: *mut c_void) {
//     // TODO: 参数和结构体 glue
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_user_config_set_config(
//     _uc: *mut UserConfig,
//     _key: *const c_char,
//     _val: *const c_char,
//     _user_info: *const c_void,
//     _out_inode: *mut c_void,
// ) -> i32 {
//     // TODO: 参数和结构体 glue
//     -1
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_user_config_lookup_config(
//     _uc: *mut UserConfig,
//     _key: *const c_char,
//     _user_info: *const c_void,
//     _out_inode: *mut c_void,
// ) -> i32 {
//     // TODO: 参数和结构体 glue
//     -1
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_user_config_stat_config(
//     _uc: *mut UserConfig,
//     _iid: u64,
//     _user_info: *const c_void,
//     _out_inode: *mut c_void,
// ) -> i32 {
//     // TODO: 参数和结构体 glue
//     -1
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_user_config_list_config(
//     _uc: *mut UserConfig,
//     _user_info: *const c_void,
//     _out_len: *mut usize,
// ) -> *mut c_void {
//     // TODO: 参数和结构体 glue
//     std::ptr::null_mut()
// }

// #[no_mangle]
// pub extern "C" fn hf3fs_user_config_get_config(
//     _uc: *mut UserConfig,
//     _user_info: *const c_void,
// ) -> *const c_void {
//     // TODO: 参数和结构体 glue
//     std::ptr::null()
// } 