cmake_minimum_required(VERSION 3.10)
project(rust_cpp_mixed)

# 设置输出目录
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/../target/release/deps)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/../target/release/deps)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/../target/release/deps)

# 添加静态库
add_library(cpp_mixed STATIC main.cpp)
target_include_directories(cpp_mixed PRIVATE ${CMAKE_SOURCE_DIR}/../include)