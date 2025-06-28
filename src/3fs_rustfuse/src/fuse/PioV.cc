#include "PioV.h"
#include <stdexcept>
#include <string>

extern "C" {
    void* hf3fs_piov_new(void* storage_client, int chunk_size_lim, void* res_vec, size_t res_len);
    void hf3fs_piov_drop(void* piov);
    int hf3fs_piov_add_read(void* piov, size_t idx, const void* inode, uint16_t track, off_t off, size_t len, void* buf, void* memh);
    int hf3fs_piov_add_write(void* piov, size_t idx, const void* inode, uint16_t track, off_t off, size_t len, const void* buf, void* memh);
    int hf3fs_piov_execute_read(void* piov, const void* user_info, const void* options);
    int hf3fs_piov_execute_write(void* piov, const void* user_info, const void* options);
    void hf3fs_piov_finish_io(void* piov, bool allow_holes);
}

namespace hf3fs::lib::agent {

PioV::PioV(storage::client::StorageClient &storageClient, int chunkSizeLim, std::vector<ssize_t> &res)
    : storageClient_(storageClient), chunkSizeLim_(chunkSizeLim), res_(res) {
    rust_ptr_ = hf3fs_piov_new(&storageClient, chunkSizeLim, res.data(), res.size());
    if (!rust_ptr_) throw std::runtime_error("Failed to create Rust PioV");
}

PioV::~PioV() {
    if (rust_ptr_) {
        hf3fs_piov_drop(rust_ptr_);
        rust_ptr_ = nullptr;
    }
}

hf3fs::Result<Void> PioV::addRead(size_t idx,
                                  const meta::Inode &inode,
                                  uint16_t track,
                                  off_t off,
                                  size_t len,
                                  void *buf,
                                  storage::client::IOBuffer &memh) {
    if (!rust_ptr_) return makeError(StatusCode::kInvalidArg, "PioV not initialized");
    int ret = hf3fs_piov_add_read(rust_ptr_, idx, &inode, track, off, len, buf, &memh);
    if (ret != 0) return makeError(StatusCode::kNotImplemented, "addRead failed");
    return Void{};
}

hf3fs::Result<Void> PioV::addWrite(size_t idx,
                                   const meta::Inode &inode,
                                   uint16_t track,
                                   off_t off,
                                   size_t len,
                                   const void *buf,
                                   storage::client::IOBuffer &memh) {
    if (!rust_ptr_) return makeError(StatusCode::kInvalidArg, "PioV not initialized");
    int ret = hf3fs_piov_add_write(rust_ptr_, idx, &inode, track, off, len, buf, &memh);
    if (ret != 0) return makeError(StatusCode::kNotImplemented, "addWrite failed");
    return Void{};
}

CoTryTask<void> PioV::executeRead(const UserInfo &userInfo, const storage::client::ReadOptions &options) {
    if (!rust_ptr_) co_return makeError(StatusCode::kInvalidArg, "PioV not initialized");
    int ret = hf3fs_piov_execute_read(rust_ptr_, &userInfo, &options);
    if (ret != 0) co_return makeError(StatusCode::kNotImplemented, "executeRead failed");
    co_return Void{};
}

CoTryTask<void> PioV::executeWrite(const UserInfo &userInfo, const storage::client::WriteOptions &options) {
    if (!rust_ptr_) co_return makeError(StatusCode::kInvalidArg, "PioV not initialized");
    int ret = hf3fs_piov_execute_write(rust_ptr_, &userInfo, &options);
    if (ret != 0) co_return makeError(StatusCode::kNotImplemented, "executeWrite failed");
    co_return Void{};
}

void PioV::finishIo(bool allowHoles) {
    if (!rust_ptr_) return;
    hf3fs_piov_finish_io(rust_ptr_, allowHoles);
}

}  // namespace hf3fs::lib::agent
