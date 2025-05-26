#ifndef RUST_FUSE_FFI_H
#define RUST_FUSE_FFI_H

#include <stdbool.h>

typedef struct KeyValueC {
    const char* key;
    const char* value;
} KeyValueC;

typedef struct FuseAppConfig FuseAppConfig;

#ifdef __cplusplus
extern "C" {
#endif

// 创建对象
FuseAppConfig* fuse_app_config_new();

// 初始化配置
void fuse_app_config_init(FuseAppConfig* obj,
                          const char* file_path,
                          bool dump,
                          const KeyValueC* updates,
                          unsigned int update_len);

// 销毁对象
void fuse_app_config_free(FuseAppConfig* obj);

#ifdef __cplusplus
}
#endif

#endif // RUST_FUSE_FFI_H
/*
#ifndef RUST_FUSE_FFI_H
#define RUST_FUSE_FFI_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// 原有 FuseAppConfig 接口
typedef struct KeyValueC {
    const char* key;
    const char* value;
} KeyValueC;

typedef struct FuseAppConfig FuseAppConfig;

// 创建 FuseAppConfig 对象
FuseAppConfig* fuse_app_config_new();

// 初始化配置
void fuse_app_config_init(FuseAppConfig* obj,
                          const char* file_path,
                          bool dump,
                          const KeyValueC* updates,
                          unsigned int update_len);

// 销毁对象
void fuse_app_config_free(FuseAppConfig* obj);

// 新增：FuseApplication 接口
typedef struct FuseApplication FuseApplication;

// 创建 FuseApplication 对象
FuseApplication* fuse_application_new();

// 解析命令行参数
int fuse_application_parse_flags(FuseApplication*, int argc, const char** argv);

// 初始化应用
int fuse_application_init(FuseApplication*);

// 启动主循环
int fuse_application_main_loop(const FuseApplication*);

// 停止应用
void fuse_application_stop(FuseApplication*);

// 获取 Node ID
uint64_t fuse_application_get_node_id(const FuseApplication*);

// 持久化配置更新
void fuse_application_on_config_updated(const FuseApplication*);

// 释放资源
void fuse_application_free(FuseApplication*);

#ifdef __cplusplus
}
#endif

#endif // RUST_FUSE_FFI_H
*/