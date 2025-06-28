# 改写和编译过程中遇到的错误与解决

## 1. FFI 相关错误

### 1.1 C/C++ 与 Rust 类型不匹配

- Rust 和 C/C++ 的类型系统不同，常见如 `usize`/`isize` 与 `size_t`/`ssize_t`，`u8`/`i32` 与 `uint8_t`/`int32_t` 等不一致。

- 结构体对齐、填充、大小不一致，导致内存访问错误。

### 1.2 ABI（应用二进制接口）不一致
- Rust 端声明 `extern "C"`，但 C 端未用 `extern "C"` 导出，或者C端声明 `extern "C"` ，但Rust端未用 `extern "C"` 导入(最开始写 hf3fs_fuse_init 模块时就出现过)。

- 函数参数传递的顺序、返回值类型不一致（比如MultiPrioQueue方法传递的Key-Value）。
```
#[no_mangle]
pub extern "C" fn hf3fs_fuse_set_user_config(
    uid: u64,
    key: *const c_char,
    value: *const c_char,
) -> c_int
```

### 1.3 链接错误

- Rust 代码调用 C/C++ 库，找不到对应的 `.so`/`.a` 文件，报 `undefined reference`。

- CMake/Cargo.toml 配置不正确，导致链接参数丢失。

### 1.4 FFI 安全性问题

- Rust 端传递了悬垂指针、未初始化内存、越界访问等，导致 C 端崩溃。

- C 端释放了 Rust 端分配的内存，导致 double free。

---

## 2. Rust 文件改写 C 文件时遇到的错误

### 2.1 trait/impl 不匹配

- 实现的 trait 方法签名与 trait 定义不一致。

- impl 块中类型参数、生命周期参数不匹配。

### 2.2 所有权/借用错误

- 借用检查器报错，如"cannot borrow as mutable more than once at a time"。

- 生命周期不明确，导致"does not live long enough"错误。

### 2.3 类型推断失败

- 类型不明确，编译器无法推断类型。

### 2.4 feature/gate 错误

- 用了 nightly feature，但没有加 `#![feature(...)]`。

- 依赖 crate 版本不兼容。

---

## 3. 编译整个 3FS 项目时遇到的错误

### 3.1 依赖缺失

- 缺少 C/C++ 依赖库（如 leveldb、rocksdb、liburing 等）。

- Rust crate 依赖没有正确声明或版本冲突。

### 3.2 CMake/Cargo 配置错误

- CMakeLists.txt/Cargo.toml 配置不正确，导致找不到源文件的位置或者链接失败。

- build.rs 脚本出错，环境变量未设置。

### 3.5 安全性错误
- unsafe 代码块未正确处理，导致运行时崩溃。

- 多线程/并发相关的竞态条件。

---

## 4. 错误示例

- `error: linking with 'cc' failed: exit code: 1`

- `undefined reference to ''`

- `error[E0308]: mismatched types`

- `error[E0597]: borrowed value does not live long enough`

- `error: could not find native static library`

- `error: expected identifier, found keyword 'type'`

- `Segmentation fault (core dumped)`（运行时）

---

## 5. 我们的对策

1. **先单独编译 FFI 相关的 C/C++ 代码**，确保无误。

2. **用 `cargo build -vv` 查看详细编译日志**，定位 Rust 端错误。

3. **检查 CMake/Cargo.toml 配置**，确认依赖和链接参数。

4. **用 `nm`/`objdump` 检查符号导出**，确认 ABI 一致。

5. **用 `RUST_BACKTRACE=1` 跑出错的二进制，获取详细堆栈信息**。

---

如有具体报错信息或日志，建议贴出详细内容以便进一步分析和定位问题。 