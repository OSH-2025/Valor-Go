#include "rust_fuse_ffi.h"
#include <iostream>

int main() {
    char* result = (char*)complete_app_info(nullptr);
    std::cout << "Result: " << result << std::endl;
    free(result);
    return 0;
}