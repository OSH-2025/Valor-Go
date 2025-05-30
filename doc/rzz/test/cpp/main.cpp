#include "rust_fuse_ffi.h"
#include <iostream>

int main() {
    char* result = (char*)complete_app_info(nullptr);
    std::cout << "Result: " << result << std::endl;
    free_string(result);

    // 测试有更新的情况
    const char* app_info_json = R"({"cluster_id": "test_cluster", "allow_other": true})";
    result = (char*)complete_app_info((char*)app_info_json);
    std::cout << "Result (with updates): " << result << std::endl;
    free_string(result);
    return 0;
}