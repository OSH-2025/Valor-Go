cmake_minimum_required(VERSION 3.10)
project(rust_cpp_mixed)

add_executable(rust_cpp_mixed main.cpp)
target_include_directories(rust_cpp_mixed PRIVATE ${CMAKE_SOURCE_DIR}/include)
target_link_libraries(rust_cpp_mixed PRIVATE rust_cpp_mixed)