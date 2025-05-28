fn main() {
    // 用于构建 C++ 部分
    println!("cargo:rustc-link-search=native=cpp/build");
    println!("cargo:rustc-link-lib=static=cpp_mixed");
}