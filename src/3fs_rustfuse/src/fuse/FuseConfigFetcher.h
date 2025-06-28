#pragma once

#include "core/app/MgmtdClientFetcher.h"

namespace hf3fs::fuse {
struct FuseConfigFetcher : public core::launcher::MgmtdClientFetcher {
  FuseConfigFetcher();
  ~FuseConfigFetcher();
  void* rust_ptr_ = nullptr;
  using core::launcher::MgmtdClientFetcher::MgmtdClientFetcher;
  Result<Void> completeAppInfo(flat::AppInfo &appInfo) final;
};
}  // namespace hf3fs::fuse
