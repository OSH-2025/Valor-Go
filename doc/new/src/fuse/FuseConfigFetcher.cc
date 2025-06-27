#include "FuseConfigFetcher.h"
#include <stdexcept>
#include "common/net/Client.h"
#include "client/mgmtd/MgmtdClient.h"

extern "C" {
    void* hf3fs_fuse_config_fetcher_new(const char* mgmtd_service_url);
    void hf3fs_fuse_config_fetcher_drop(void* fetcher);
    int hf3fs_fuse_config_fetcher_complete_app_info(void* fetcher, void* app_info);
}

namespace hf3fs::fuse {

FuseConfigFetcher::FuseConfigFetcher()
    : core::launcher::MgmtdClientFetcher("", net::Client::Config(), client::MgmtdClient::Config())
{
    // 这里可以传递mgmtd_service_url，暂用空字符串
    rust_ptr_ = hf3fs_fuse_config_fetcher_new("");
    if (!rust_ptr_) throw std::runtime_error("Failed to create Rust FuseConfigFetcher");
}

FuseConfigFetcher::~FuseConfigFetcher() {
    if (rust_ptr_) {
        hf3fs_fuse_config_fetcher_drop(rust_ptr_);
        rust_ptr_ = nullptr;
    }
}

Result<Void> FuseConfigFetcher::completeAppInfo(flat::AppInfo &appInfo) {
    if (!rust_ptr_) return makeError(StatusCode::kInvalidArg, "Invalid rust_ptr_");
    // TODO: 这里需要将appInfo适配为Rust端可用的结构体指针
    int ret = hf3fs_fuse_config_fetcher_complete_app_info(rust_ptr_, &appInfo);
    if (ret != 0) return makeError(StatusCode::kUnknown, "Rust complete_app_info failed");
    return Void{};
}

}  // namespace hf3fs::fuse
