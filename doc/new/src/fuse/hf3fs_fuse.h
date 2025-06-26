#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

int hf3fs_fuse_init(const char *config_path, const char *mountpoint, const char *token_file);

int hf3fs_fuse_run(int allow_other, uintptr_t maxbufsize, const char *cluster_id);

void hf3fs_fuse_cleanup(void);

void *hf3fs_fuse_get_config(void);

int hf3fs_fuse_set_user_config(uint64_t uid, const char *key, const char *value);
