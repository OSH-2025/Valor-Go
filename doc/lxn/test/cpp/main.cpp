/*
#include "rust_fuse_ffi.h"
#include <vector>
#include <iostream>

int main() {
    auto cfg = fuse_app_config_new();

    std::vector<KeyValueC> updates = {
        { "key1", "value1" },
        { "key2", "value2" }
    };

    fuse_app_config_init(cfg, "config.json", true, updates.data(), updates.size());

    fuse_app_config_free(cfg);
    std::cout << "Rust FuseAppConfig initialized successfully!" << std::endl;
    return 0;
}
*/

#include "rust_fuse_ffi.h"
#include <iostream>

int main() {
    auto config = fuse_app_config_new();
    if (config == nullptr) {
        std::cerr << "Failed to create FuseAppConfig\n";
        return -1;
    }

    // 调用 init 方法
    fuse_app_config_init(config, "config.json", true, nullptr, 0);

    // 获取 NodeId
    uint64_t node_id = fuse_application_get_node_id(config);
    std::cout << "Node ID: " << node_id << std::endl;

    // 释放资源
    fuse_app_config_free(config);

    return 0;
}
