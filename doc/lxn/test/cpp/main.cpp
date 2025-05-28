#include "rust_fuse_ffi.h"
#include <iostream>

int main(int argc, const char** argv) {
    auto config = fuse_app_config_new();
    if (!config) {
        std::cerr << "Failed to create FuseAppConfig\n";
        return -1;
    }

    // 初始化配置
    fuse_app_config_init(config, "config.json", true, nullptr, 0);

    // 获取 Node ID
    uint64_t node_id = fuse_app_config_get_node_id(config);
    std::cout << "Node ID (from FuseAppConfig): " << node_id << "\n";

    // 创建 Application 并解析参数
    auto app = fuse_application_new();
    if (!app) {
        std::cerr << "Failed to create FuseApplication\n";
        goto cleanup_config;
    }

    int res = fuse_application_parse_flags(app, argc, argv);
    if (res != 0) {
        std::cerr << "Parse flags failed\n";
        goto cleanup_app;
    }

    res = fuse_application_init(app);
    if (res != 0) {
        std::cerr << "Init application failed\n";
        goto cleanup_app;
    }

    // 启动 FuseClients
    auto clients = fuse_clients_new("/mnt/hf3fs", "dummy_token");
    if (!clients) {
        std::cerr << "Failed to create FuseClients\n";
        goto cleanup_app;
    }

    fuse_clients_start(clients);
    fuse_clients_periodic_sync_scan(clients);
    fuse_clients_stop(clients);
    fuse_clients_free(clients);

    // 停止应用
    fuse_application_stop(app);
    fuse_application_free(app);

cleanup_app:
    fuse_app_config_free(config);

cleanup_config:
    return res;
}
