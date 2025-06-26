#include "hf3fs_fuse.h"
#include <string>

int main(int argc, char *argv[]) {
    if (argc < 4) {
        printf("Usage: %s <config_path> <mountpoint> <token_file>\n", argv[0]);
        return 1;
    }
    int ret = hf3fs_fuse_init(argv[1], argv[2], argv[3]);
    if (ret != 0) {
        printf("hf3fs_fuse_init failed: %d\n", ret);
        return ret;
    }
    // 这里参数可根据需要调整
    ret = hf3fs_fuse_run(1, 1024 * 1024, "test_cluster");
    hf3fs_fuse_cleanup();
    return ret;
}
