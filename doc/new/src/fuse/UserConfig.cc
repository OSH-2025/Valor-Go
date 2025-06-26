#include "UserConfig.h"
#include <stdexcept>
#include <string>

extern "C" {
    void* hf3fs_user_config_new();
    void hf3fs_user_config_drop(void* uc);
    void hf3fs_user_config_init(void* uc, void* fuse_config);
    int hf3fs_user_config_set_config(void* uc, const char* key, const char* val, const void* user_info, void* out_inode);
    int hf3fs_user_config_lookup_config(void* uc, const char* key, const void* user_info, void* out_inode);
    int hf3fs_user_config_stat_config(void* uc, uint64_t iid, const void* user_info, void* out_inode);
    void* hf3fs_user_config_list_config(void* uc, const void* user_info, size_t* out_len);
    const void* hf3fs_user_config_get_config(void* uc, const void* user_info);
}

namespace hf3fs::fuse {

UserConfig::UserConfig() {
    rust_ptr_ = hf3fs_user_config_new();
    if (!rust_ptr_) throw std::runtime_error("Failed to create Rust UserConfig");
}

UserConfig::~UserConfig() {
    if (rust_ptr_) {
        hf3fs_user_config_drop(rust_ptr_);
        rust_ptr_ = nullptr;
    }
}

void UserConfig::init(FuseConfig &config) {
    if (!rust_ptr_) throw std::runtime_error("UserConfig not initialized");
    hf3fs_user_config_init(rust_ptr_, &config);
}

Result<meta::Inode> UserConfig::setConfig(const char *key, const char *val, const meta::UserInfo &ui) {
    if (!rust_ptr_) return makeError(StatusCode::kInvalidArg, "UserConfig not initialized");
    meta::Inode out_inode;
    int ret = hf3fs_user_config_set_config(rust_ptr_, key, val, &ui, &out_inode);
    if (ret != 0) return makeError(StatusCode::kNotSupported, "setConfig failed");
    return out_inode;
}

Result<meta::Inode> UserConfig::lookupConfig(const char *key, const meta::UserInfo &ui) {
    if (!rust_ptr_) return makeError(StatusCode::kInvalidArg, "UserConfig not initialized");
    meta::Inode out_inode;
    int ret = hf3fs_user_config_lookup_config(rust_ptr_, key, &ui, &out_inode);
    if (ret != 0) return makeError(StatusCode::kNotSupported, "lookupConfig failed");
    return out_inode;
}

Result<meta::Inode> UserConfig::statConfig(meta::InodeId iid, const meta::UserInfo &ui) {
    if (!rust_ptr_) return makeError(StatusCode::kInvalidArg, "UserConfig not initialized");
    meta::Inode out_inode;
    int ret = hf3fs_user_config_stat_config(rust_ptr_, iid.u64(), &ui, &out_inode);
    if (ret != 0) return makeError(StatusCode::kNotSupported, "statConfig failed");
    return out_inode;
}

std::pair<std::shared_ptr<std::vector<meta::DirEntry>>, std::shared_ptr<std::vector<std::optional<meta::Inode>>>>
UserConfig::listConfig(const meta::UserInfo &ui) {
    if (!rust_ptr_) return {nullptr, nullptr};
    size_t out_len = 0;
    void* result = hf3fs_user_config_list_config(rust_ptr_, &ui, &out_len);
    // TODO: 需要将result转换为C++ std::vector<...>
    return {nullptr, nullptr};
}

const FuseConfig &UserConfig::getConfig(const meta::UserInfo &ui) {
    if (!rust_ptr_) throw std::runtime_error("UserConfig not initialized");
    return *reinterpret_cast<const FuseConfig*>(hf3fs_user_config_get_config(rust_ptr_, &ui));
}

}  // namespace hf3fs::fuse
