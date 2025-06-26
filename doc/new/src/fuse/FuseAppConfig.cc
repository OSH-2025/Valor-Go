#include "FuseAppConfig.h"
#include <vector>
#include <string>

extern "C" {
struct FfiKeyValue {
    const char* key;
    const char* value;
};
int hf3fs_fuse_app_config_init(
    void* config,
    const char* file_path,
    bool dump,
    const FfiKeyValue* updates,
    size_t updates_len);
}

namespace hf3fs::fuse {

void FuseAppConfig::init(const String &filePath, bool dump, const std::vector<config::KeyValue> &updates) {
    std::vector<FfiKeyValue> ffi_updates;
    std::vector<std::string> key_buf, value_buf;
    for (const auto& kv : updates) {
        key_buf.push_back(kv.first);
        value_buf.push_back(kv.second);
        ffi_updates.push_back(FfiKeyValue{key_buf.back().c_str(), value_buf.back().c_str()});
    }
    int ret = hf3fs_fuse_app_config_init(
        this,
        filePath.c_str(),
        dump,
        ffi_updates.data(),
        ffi_updates.size()
    );
    if (ret != 0) {
        throw std::runtime_error("Init app config failed (Rust FFI)");
    }
}

}  // namespace hf3fs::fuse
