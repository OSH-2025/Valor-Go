//添加
cmake_minimum_required(VERSION 3.15)
project(rust_cpp_mixed)

set(CMAKE_CXX_STANDARD 17)

add_subdirectory(rust_fuse)  # 假设你把 Rust 项目作为子模块或外部库

add_executable(main main.cpp)
target_include_directories(main PRIVATE ../include)
target_link_libraries(main PRIVATE rust_fuse)