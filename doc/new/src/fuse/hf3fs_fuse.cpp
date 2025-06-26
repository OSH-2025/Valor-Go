#include "hf3fs_fuse.h"
#include <cstring>
#include <iostream>

extern "C" int hf3fs_fuse_run(int allow_other, size_t maxbufsize, const char* cluster_id);

int main(int argc, char *argv[]) {
    // 示例参数，实际可根据你的需求从命令行解析
    int allow_other = 1;
    size_t maxbufsize = 1048576;
    const char* cluster_id = "default_cluster";

    // 你可以根据argc/argv解析参数并传递给Rust
    // 这里只是简单示例
    int ret = hf3fs_fuse_run(allow_other, maxbufsize, cluster_id);
    std::cout << "hf3fs_fuse_run returned: " << ret << std::endl;
    return ret;
} 