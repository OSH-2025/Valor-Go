#include <iostream>
#include <cstdio>

extern "C" {
    struct UserConfigHandle;

    UserConfigHandle* user_config_new();
    void user_config_free(UserConfigHandle*);

    char* user_config_set_config(UserConfigHandle*, const char* key, const char* val, unsigned int uid, unsigned int gid);
    char* user_config_lookup_config(UserConfigHandle*, const char* key, unsigned int uid, unsigned int gid);
}

int main() {
    auto* handle = user_config_new();

    const char* sys_key = "sys.storage.net_client.rdma_control.max_concurrent_transmission";
    const char* sys_val = "256";
    unsigned int uid = 1000;
    unsigned int gid = 1000;

    char* result = user_config_set_config(handle, sys_key, sys_val, uid, gid);
    if (result) {
        std::cout << "Set system config result: " << result << std::endl;
        std::free(result);
    }

    result = user_config_lookup_config(handle, sys_key, uid, gid);
    if (result) {
        std::cout << "Lookup system config result: " << result << std::endl;
        std::free(result);
    }

    const char* user_key = "usr.readonly";
    const char* user_val = "true";

    result = user_config_set_config(handle, user_key, user_val, uid, gid);
    if (result) {
        std::cout << "Set user config result: " << result << std::endl;
        std::free(result);
    }

    result = user_config_lookup_config(handle, user_key, uid, gid);
    if (result) {
        std::cout << "Lookup user config result: " << result << std::endl;
        std::free(result);
    }

    user_config_free(handle);

    return 0;
}