# CMakeLists.txt 示例：集成 Rust FUSE 库
cmake_minimum_required(VERSION 3.16)
project(hf3fs_fuse_mixed)

# 设置 Rust 工具链
find_program(CARGO cargo REQUIRED)

# 编译 Rust 库
add_custom_target(rust_fuse_lib ALL
    COMMAND ${CMAKE_COMMAND} -E env CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR}/rust_target
    ${CARGO} build --release --manifest-path ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust FUSE library"
)

# 设置 Rust 库路径
set(RUST_LIB_DIR ${CMAKE_CURRENT_BINARY_DIR}/rust_target/release)
set(RUST_LIB_NAME hf3fs_fuse)

# 创建 C++ 可执行文件
add_executable(hf3fs_fuse_cpp main.cpp)

# 链接 Rust 库
target_link_libraries(hf3fs_fuse_cpp
    ${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.a
    # 其他系统库
    pthread
    dl
    fuse3
)

# 包含头文件目录
target_include_directories(hf3fs_fuse_cpp PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

# 依赖关系
add_dependencies(hf3fs_fuse_cpp rust_fuse_lib)

if(CMAKE_SYSTEM_PROCESSOR STREQUAL "x86_64")
    link_directories(/usr/local/lib/x86_64-linux-gnu/ /usr/lib64 /usr/local/lib64)
elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    link_directories(/usr/local/lib/aarch64-linux-gnu/ /usr/lib64 /usr/local/lib64)
endif()

target_add_lib(hf3fs_fuse common core-app meta-client storage-client fuse3 client-lib-common)
target_add_bin(hf3fs_fuse_main hf3fs_fuse.cpp hf3fs_fuse)

if (ENABLE_FUSE_APPLICATION)
target_compile_definitions(hf3fs_fuse PUBLIC -DENABLE_FUSE_APPLICATION)
endif()
