#include "IoRing.h"

#include <optional>
#include <type_traits>
#include <utility>
#include <stdexcept>
#include <cstring>

#include "PioV.h"
#include "common/utils/UtcTime.h"
#include "fbs/meta/Schema.h"
#include "fuse/FuseClients.h"
#include "fuse/FuseOps.h"
#include "lib/api/hf3fs_usrbio.h"

extern "C" {
    void* hf3fs_ioring_new(const char* name, size_t entries, size_t io_depth, size_t priority, uint64_t timeout_ms, bool for_read);
    bool hf3fs_ioring_add_sqe(void* ring, FfiIoArgs args);
    struct FfiIoArgs {
        uint64_t file_id;
        uint64_t file_off;
        size_t io_len;
        uint8_t* buf;
        size_t buf_len;
        uint64_t userdata;
    };
}

namespace hf3fs::fuse {

IoRing::IoRing(/* 你可以根据需要添加参数 */) {
    // 这里只是示例，实际参数请根据你的需求传递
    rust_ptr_ = hf3fs_ioring_new("ioring", 128, 32, 0, 1000, true);
    if (!rust_ptr_) throw std::runtime_error("Failed to create Rust IoRing");
}

IoRing::~IoRing() {
    // 如果有drop函数可以调用，记得释放
    // 例如: hf3fs_ioring_drop(rust_ptr_);
    rust_ptr_ = nullptr;
}

bool IoRing::addSqe(int idx, const void *userdata) {
    if (!rust_ptr_) return false;
    FfiIoArgs args{};
    args.file_id = 0; // 你需要根据实际参数填充
    args.file_off = 0;
    args.io_len = 0;
    args.buf = nullptr;
    args.buf_len = 0;
    args.userdata = reinterpret_cast<uint64_t>(userdata);
    return hf3fs_ioring_add_sqe(rust_ptr_, args);
}

std::vector<IoRingJob> IoRing::jobsToProc(int maxJobs) {
    // TODO: 调用Rust FFI（如hf3fs_ioring_jobs_to_proc），并将结果转换为C++ IoRingJob
    std::vector<IoRingJob> jobs;
    // 示例：int out_len = 0; hf3fs_ioring_jobs_to_proc(rust_ptr_, maxJobs, jobs.data(), &out_len); jobs.resize(out_len);
    return jobs;
}

bool IoRing::process(const std::unordered_map<uint64_t, std::shared_ptr<void>>& fileMap, std::vector<IoCqe>& cqes) {
    // TODO: 调用Rust FFI（如hf3fs_ioring_process），并将结果转换为C++ IoCqe
    // 示例：int out_len = 0; hf3fs_ioring_process(rust_ptr_, fileMapPtr, cqes.data(), &out_len); cqes.resize(out_len);
    return true;
}
}  // namespace hf3fs::fuse
