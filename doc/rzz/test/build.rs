extern crate cmake;
use cmake::Config;

fn main() {
    // Build the C++ part using CMake
    let _dst = Config::new("cpp")
        .build_target("cpp_mixed") // Assuming the CMake target is named cpp_mixed
        .build();

    // The CMAKE_ARCHIVE_OUTPUT_DIRECTORY is set in CMakeLists.txt
    // to ${CMAKE_SOURCE_DIR}/../target/release/deps
    // which is target/release/deps from the project root.
    println!("cargo:rustc-link-search=native=target/release/deps");
    println!("cargo:rustc-link-lib=static=cpp_mixed");

    // If your C++ library depends on other system libraries, you might need to link them too.
    // For example, if it uses the standard C++ library:
    // println!("cargo:rustc-link-lib=dylib=stdc++");
}