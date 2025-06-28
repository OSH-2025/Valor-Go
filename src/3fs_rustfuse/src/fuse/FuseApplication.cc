#ifdef ENABLE_FUSE_APPLICATION

#include "FuseApplication.h"
#include <vector>
#include <string>
#include <stdexcept>

extern "C" {
    void* hf3fs_fuse_application_new();
    void hf3fs_fuse_application_drop(void* app);
    int hf3fs_fuse_application_parse_flags(void* app, int argc, const char** argv);
    int hf3fs_fuse_application_init_application(void* app);
    void hf3fs_fuse_application_stop(void* app);
    int hf3fs_fuse_application_main_loop(void* app);
    const void* hf3fs_fuse_application_get_config(void* app);
    const void* hf3fs_fuse_application_info(void* app);
    bool hf3fs_fuse_application_config_pushable(void* app);
    void hf3fs_fuse_application_on_config_updated(void* app);
}

namespace hf3fs::fuse {

FuseApplication::FuseApplication() {
    rust_ptr_ = hf3fs_fuse_application_new();
    if (!rust_ptr_) throw std::runtime_error("Failed to create Rust FuseApplication");
}

FuseApplication::~FuseApplication() {
    if (rust_ptr_) {
        hf3fs_fuse_application_drop(rust_ptr_);
        rust_ptr_ = nullptr;
    }
}

Result<Void> FuseApplication::parseFlags(int *argc, char ***argv) {
    if (!rust_ptr_ || !argc || !argv || !*argv) return makeError(StatusCode::kInvalidArg, "Invalid args");
    int argc_val = *argc;
    std::vector<const char*> argv_vec(argc_val);
    for (int i = 0; i < argc_val; ++i) {
        argv_vec[i] = (*argv)[i];
    }
    int ret = hf3fs_fuse_application_parse_flags(rust_ptr_, argc_val, argv_vec.data());
    if (ret != 0) return makeError(StatusCode::kUnknown, "Rust parse_flags failed");
    return Void{};
}

Result<Void> FuseApplication::initApplication() {
    if (!rust_ptr_) return makeError(StatusCode::kInvalidArg, "Invalid rust_ptr_");
    int ret = hf3fs_fuse_application_init_application(rust_ptr_);
    if (ret != 0) return makeError(StatusCode::kUnknown, "Rust init_application failed");
    return Void{};
}

void FuseApplication::stop() {
    if (rust_ptr_) hf3fs_fuse_application_stop(rust_ptr_);
}

int FuseApplication::mainLoop() {
    if (!rust_ptr_) return -1;
    return hf3fs_fuse_application_main_loop(rust_ptr_);
}

config::IConfig *FuseApplication::getConfig() {
    if (!rust_ptr_) return nullptr;
    return reinterpret_cast<config::IConfig*>(const_cast<void*>(hf3fs_fuse_application_get_config(rust_ptr_)));
}

const flat::AppInfo *FuseApplication::info() const {
    if (!rust_ptr_) return nullptr;
    return reinterpret_cast<const flat::AppInfo*>(hf3fs_fuse_application_info(const_cast<void*>(rust_ptr_)));
}

bool FuseApplication::configPushable() const {
    if (!rust_ptr_) return false;
    return hf3fs_fuse_application_config_pushable(const_cast<void*>(rust_ptr_));
}

void FuseApplication::onConfigUpdated() {
    if (rust_ptr_) hf3fs_fuse_application_on_config_updated(rust_ptr_);
}

}  // namespace hf3fs::fuse

#endif
