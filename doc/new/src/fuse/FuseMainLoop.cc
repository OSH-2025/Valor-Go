#include "FuseMainLoop.h"
#include <string>

extern "C" {
int hf3fs_fuse_main_loop(
    const char* program_name,
    bool allow_other,
    const char* mountpoint,
    size_t maxbufsize,
    const char* cluster_id);
}

namespace hf3fs::fuse {

int fuseMainLoop(const String &programName,
                 bool allowOther,
                 const String &mountpoint,
                 size_t maxbufsize,
                 const String &clusterId) {
    return hf3fs_fuse_main_loop(
        programName.c_str(),
        allowOther,
        mountpoint.c_str(),
        maxbufsize,
        clusterId.c_str()
    );
}

}  // namespace hf3fs::fuse
