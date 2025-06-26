cmake_minimum_required(VERSION 3.16)
project(hf3fs_fuse_mixed)

# 设置 Rust 工具链
find_program(CARGO cargo REQUIRED)

# 编译 Rust 静态库
add_custom_command(
    OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/rust_target/release/libhf3fs_fuse.a
    COMMAND ${CMAKE_COMMAND} -E env CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR}/rust_target
            ${CARGO} build --release --manifest-path ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust FUSE library"
)

add_custom_target(rust_fuse_lib ALL
    DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/rust_target/release/libhf3fs_fuse.a
)

set(RUST_LIB_DIR ${CMAKE_CURRENT_BINARY_DIR}/rust_target/release)
set(RUST_LIB_NAME hf3fs_fuse)

# 你的C++可执行文件
add_executable(hf3fs_fuse_main hf3fs_fuse.cpp)

# 链接 Rust 静态库
target_link_libraries(hf3fs_fuse_main
    ${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.a
    pthread
    dl
    fuse3
)

target_include_directories(hf3fs_fuse_main PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
add_dependencies(hf3fs_fuse_main rust_fuse_lib)
