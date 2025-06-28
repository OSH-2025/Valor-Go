if(CMAKE_SYSTEM_PROCESSOR STREQUAL "x86_64")
    link_directories(/usr/local/lib/x86_64-linux-gnu/ /usr/lib64 /usr/local/lib64)
elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    link_directories(/usr/local/lib/aarch64-linux-gnu/ /usr/lib64 /usr/local/lib64)
endif()

# 添加 Rust FFI 静态库路径
link_directories(${CMAKE_CURRENT_SOURCE_DIR}/target/release)

# 添加 Rust FFI 静态库到主程序链接
set(RUST_FUSE_LIB hf3fs_fuse)

# 依赖库
target_add_lib(hf3fs_fuse common core-app meta-client storage-client fuse3 client-lib-common)
target_add_bin(hf3fs_fuse_main hf3fs_fuse.cpp hf3fs_fuse)

target_link_libraries(hf3fs_fuse_main ${CMAKE_CURRENT_SOURCE_DIR}/target/release/libhf3fs_fuse.a)

if (ENABLE_FUSE_APPLICATION)
target_compile_definitions(hf3fs_fuse PUBLIC -DENABLE_FUSE_APPLICATION)
endif()
