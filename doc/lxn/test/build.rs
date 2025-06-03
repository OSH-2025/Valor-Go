// build.rs

extern crate cc;

fn main() {
    // 告诉 Cargo 需要重新编译的情况
    println!("cargo:rerun-if-changed=src/fuse_app_config.rs");
    println!("cargo:rerun-if-changed=src/ffi.rs");

    // 如果你有 C++ 文件需要编译，可以取消注释下面这行
    /*
    cc::Build::new()
        .cpp(true)
        .file("src_cpp/example.cpp")
        .compile("libexample.a");
    
    println!("cargo:rustc-link-lib=static=example");
    println!("cargo:rustc-link-search=native={}", std::env::var("OUT_DIR").unwrap());
    */
}