#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <iterator>
#include <new>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

#ifndef CXXBRIDGE1_PANIC
#define CXXBRIDGE1_PANIC
template <typename Exception>
void panic [[noreturn]] (const char *msg);
#endif // CXXBRIDGE1_PANIC

namespace {
template <typename T>
class impl;
} // namespace

class String;

template <typename T>
::std::size_t size_of();
template <typename T>
::std::size_t align_of();

#ifndef CXXBRIDGE1_RUST_STR
#define CXXBRIDGE1_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const String &) noexcept;
  Str(const std::string &);
  Str(const char *);
  Str(const char *, std::size_t);

  Str &operator=(const Str &) &noexcept = default;

  explicit operator std::string() const;

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

  using iterator = const char *;
  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const Str &) const noexcept;
  bool operator!=(const Str &) const noexcept;
  bool operator<(const Str &) const noexcept;
  bool operator<=(const Str &) const noexcept;
  bool operator>(const Str &) const noexcept;
  bool operator>=(const Str &) const noexcept;

  void swap(Str &) noexcept;

private:
  class uninit;
  Str(uninit) noexcept;
  friend impl<Str>;

  std::array<std::uintptr_t, 2> repr;
};
#endif // CXXBRIDGE1_RUST_STR

#ifndef CXXBRIDGE1_RUST_SLICE
#define CXXBRIDGE1_RUST_SLICE
namespace detail {
template <bool>
struct copy_assignable_if {};

template <>
struct copy_assignable_if<false> {
  copy_assignable_if() noexcept = default;
  copy_assignable_if(const copy_assignable_if &) noexcept = default;
  copy_assignable_if &operator=(const copy_assignable_if &) &noexcept = delete;
  copy_assignable_if &operator=(copy_assignable_if &&) &noexcept = default;
};
} // namespace detail

template <typename T>
class Slice final
    : private detail::copy_assignable_if<std::is_const<T>::value> {
public:
  using value_type = T;

  Slice() noexcept;
  Slice(T *, std::size_t count) noexcept;

  template <typename C>
  explicit Slice(C& c) : Slice(c.data(), c.size()) {}

  Slice &operator=(const Slice<T> &) &noexcept = default;
  Slice &operator=(Slice<T> &&) &noexcept = default;

  T *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  T &operator[](std::size_t n) const noexcept;
  T &at(std::size_t n) const;
  T &front() const noexcept;
  T &back() const noexcept;

  Slice(const Slice<T> &) noexcept = default;
  ~Slice() noexcept = default;

  class iterator;
  iterator begin() const noexcept;
  iterator end() const noexcept;

  void swap(Slice &) noexcept;

private:
  class uninit;
  Slice(uninit) noexcept;
  friend impl<Slice>;
  friend void sliceInit(void *, const void *, std::size_t) noexcept;
  friend void *slicePtr(const void *) noexcept;
  friend std::size_t sliceLen(const void *) noexcept;

  std::array<std::uintptr_t, 2> repr;
};

template <typename T>
class Slice<T>::iterator final {
public:
  using iterator_category = std::random_access_iterator_tag;
  using value_type = T;
  using difference_type = std::ptrdiff_t;
  using pointer = typename std::add_pointer<T>::type;
  using reference = typename std::add_lvalue_reference<T>::type;

  reference operator*() const noexcept;
  pointer operator->() const noexcept;
  reference operator[](difference_type) const noexcept;

  iterator &operator++() noexcept;
  iterator operator++(int) noexcept;
  iterator &operator--() noexcept;
  iterator operator--(int) noexcept;

  iterator &operator+=(difference_type) noexcept;
  iterator &operator-=(difference_type) noexcept;
  iterator operator+(difference_type) const noexcept;
  iterator operator-(difference_type) const noexcept;
  difference_type operator-(const iterator &) const noexcept;

  bool operator==(const iterator &) const noexcept;
  bool operator!=(const iterator &) const noexcept;
  bool operator<(const iterator &) const noexcept;
  bool operator<=(const iterator &) const noexcept;
  bool operator>(const iterator &) const noexcept;
  bool operator>=(const iterator &) const noexcept;

private:
  friend class Slice;
  void *pos;
  std::size_t stride;
};

template <typename T>
Slice<T>::Slice() noexcept {
  sliceInit(this, reinterpret_cast<void *>(align_of<T>()), 0);
}

template <typename T>
Slice<T>::Slice(T *s, std::size_t count) noexcept {
  assert(s != nullptr || count == 0);
  sliceInit(this,
            s == nullptr && count == 0
                ? reinterpret_cast<void *>(align_of<T>())
                : const_cast<typename std::remove_const<T>::type *>(s),
            count);
}

template <typename T>
T *Slice<T>::data() const noexcept {
  return reinterpret_cast<T *>(slicePtr(this));
}

template <typename T>
std::size_t Slice<T>::size() const noexcept {
  return sliceLen(this);
}

template <typename T>
std::size_t Slice<T>::length() const noexcept {
  return this->size();
}

template <typename T>
bool Slice<T>::empty() const noexcept {
  return this->size() == 0;
}

template <typename T>
T &Slice<T>::operator[](std::size_t n) const noexcept {
  assert(n < this->size());
  auto ptr = static_cast<char *>(slicePtr(this)) + size_of<T>() * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
T &Slice<T>::at(std::size_t n) const {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Slice index out of range");
  }
  return (*this)[n];
}

template <typename T>
T &Slice<T>::front() const noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
T &Slice<T>::back() const noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
typename Slice<T>::iterator::reference
Slice<T>::iterator::operator*() const noexcept {
  return *static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::pointer
Slice<T>::iterator::operator->() const noexcept {
  return static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::reference Slice<T>::iterator::operator[](
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ptr = static_cast<char *>(this->pos) + this->stride * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator++() noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator++(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator--() noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator--(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator+=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator-=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator+(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) + this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator-(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) - this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator::difference_type
Slice<T>::iterator::operator-(const iterator &other) const noexcept {
  auto diff = std::distance(static_cast<char *>(other.pos),
                            static_cast<char *>(this->pos));
  return diff / static_cast<typename Slice<T>::iterator::difference_type>(
                    this->stride);
}

template <typename T>
bool Slice<T>::iterator::operator==(const iterator &other) const noexcept {
  return this->pos == other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator!=(const iterator &other) const noexcept {
  return this->pos != other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator<(const iterator &other) const noexcept {
  return this->pos < other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator<=(const iterator &other) const noexcept {
  return this->pos <= other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>(const iterator &other) const noexcept {
  return this->pos > other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>=(const iterator &other) const noexcept {
  return this->pos >= other.pos;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::begin() const noexcept {
  iterator it;
  it.pos = slicePtr(this);
  it.stride = size_of<T>();
  return it;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::end() const noexcept {
  iterator it = this->begin();
  it.pos = static_cast<char *>(it.pos) + it.stride * this->size();
  return it;
}

template <typename T>
void Slice<T>::swap(Slice &rhs) noexcept {
  std::swap(*this, rhs);
}
#endif // CXXBRIDGE1_RUST_SLICE

#ifndef CXXBRIDGE1_RUST_BOX
#define CXXBRIDGE1_RUST_BOX
template <typename T>
class Box final {
public:
  using element_type = T;
  using const_pointer =
      typename std::add_pointer<typename std::add_const<T>::type>::type;
  using pointer = typename std::add_pointer<T>::type;

  Box() = delete;
  Box(Box &&) noexcept;
  ~Box() noexcept;

  explicit Box(const T &);
  explicit Box(T &&);

  Box &operator=(Box &&) &noexcept;

  const T *operator->() const noexcept;
  const T &operator*() const noexcept;
  T *operator->() noexcept;
  T &operator*() noexcept;

  template <typename... Fields>
  static Box in_place(Fields &&...);

  void swap(Box &) noexcept;

  static Box from_raw(T *) noexcept;

  T *into_raw() noexcept;

  /* Deprecated */ using value_type = element_type;

private:
  class uninit;
  class allocation;
  Box(uninit) noexcept;
  void drop() noexcept;

  friend void swap(Box &lhs, Box &rhs) noexcept { lhs.swap(rhs); }

  T *ptr;
};

template <typename T>
class Box<T>::uninit {};

template <typename T>
class Box<T>::allocation {
  static T *alloc() noexcept;
  static void dealloc(T *) noexcept;

public:
  allocation() noexcept : ptr(alloc()) {}
  ~allocation() noexcept {
    if (this->ptr) {
      dealloc(this->ptr);
    }
  }
  T *ptr;
};

template <typename T>
Box<T>::Box(Box &&other) noexcept : ptr(other.ptr) {
  other.ptr = nullptr;
}

template <typename T>
Box<T>::Box(const T &val) {
  allocation alloc;
  ::new (alloc.ptr) T(val);
  this->ptr = alloc.ptr;
  alloc.ptr = nullptr;
}

template <typename T>
Box<T>::Box(T &&val) {
  allocation alloc;
  ::new (alloc.ptr) T(std::move(val));
  this->ptr = alloc.ptr;
  alloc.ptr = nullptr;
}

template <typename T>
Box<T>::~Box() noexcept {
  if (this->ptr) {
    this->drop();
  }
}

template <typename T>
Box<T> &Box<T>::operator=(Box &&other) &noexcept {
  if (this->ptr) {
    this->drop();
  }
  this->ptr = other.ptr;
  other.ptr = nullptr;
  return *this;
}

template <typename T>
const T *Box<T>::operator->() const noexcept {
  return this->ptr;
}

template <typename T>
const T &Box<T>::operator*() const noexcept {
  return *this->ptr;
}

template <typename T>
T *Box<T>::operator->() noexcept {
  return this->ptr;
}

template <typename T>
T &Box<T>::operator*() noexcept {
  return *this->ptr;
}

template <typename T>
template <typename... Fields>
Box<T> Box<T>::in_place(Fields &&...fields) {
  allocation alloc;
  auto ptr = alloc.ptr;
  ::new (ptr) T{std::forward<Fields>(fields)...};
  alloc.ptr = nullptr;
  return from_raw(ptr);
}

template <typename T>
void Box<T>::swap(Box &rhs) noexcept {
  using std::swap;
  swap(this->ptr, rhs.ptr);
}

template <typename T>
Box<T> Box<T>::from_raw(T *raw) noexcept {
  Box box = uninit{};
  box.ptr = raw;
  return box;
}

template <typename T>
T *Box<T>::into_raw() noexcept {
  T *raw = this->ptr;
  this->ptr = nullptr;
  return raw;
}

template <typename T>
Box<T>::Box(uninit) noexcept {}
#endif // CXXBRIDGE1_RUST_BOX

#ifndef CXXBRIDGE1_RUST_OPAQUE
#define CXXBRIDGE1_RUST_OPAQUE
class Opaque {
public:
  Opaque() = delete;
  Opaque(const Opaque &) = delete;
  ~Opaque() = delete;
};
#endif // CXXBRIDGE1_RUST_OPAQUE

#ifndef CXXBRIDGE1_IS_COMPLETE
#define CXXBRIDGE1_IS_COMPLETE
namespace detail {
namespace {
template <typename T, typename = std::size_t>
struct is_complete : std::false_type {};
template <typename T>
struct is_complete<T, decltype(sizeof(T))> : std::true_type {};
} // namespace
} // namespace detail
#endif // CXXBRIDGE1_IS_COMPLETE

#ifndef CXXBRIDGE1_LAYOUT
#define CXXBRIDGE1_LAYOUT
class layout {
  template <typename T>
  friend std::size_t size_of();
  template <typename T>
  friend std::size_t align_of();
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return T::layout::size();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return sizeof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      size_of() {
    return do_size_of<T>();
  }
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return T::layout::align();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return alignof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      align_of() {
    return do_align_of<T>();
  }
};

template <typename T>
std::size_t size_of() {
  return layout::size_of<T>();
}

template <typename T>
std::size_t align_of() {
  return layout::align_of<T>();
}
#endif // CXXBRIDGE1_LAYOUT

template <typename T>
class Slice<T>::uninit {};
template <typename T>
inline Slice<T>::Slice(uninit) noexcept {}

namespace repr {
using Fat = ::std::array<::std::uintptr_t, 2>;
} // namespace repr

namespace {
template <typename T>
class impl<Slice<T>> final {
public:
  static Slice<T> slice(repr::Fat repr) noexcept {
    Slice<T> slice = typename Slice<T>::uninit{};
    slice.repr = repr;
    return slice;
  }
};
} // namespace
} // namespace cxxbridge1
} // namespace rust

namespace hf3fs {
  namespace chunk_engine {
    struct UpdateReq;
    struct GetReq;
    struct RawMeta;
    struct RawUsedSize;
    struct FdAndOffset;
    struct Metrics;
    struct Engine;
    struct LogGuard;
    struct Chunk;
    struct WritingChunk;
    struct RawChunks;
  }
}

namespace hf3fs {
namespace chunk_engine {
#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$UpdateReq
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$UpdateReq
struct UpdateReq final {
  bool without_checksum;
  bool is_truncate;
  bool is_remove;
  bool is_syncing;
  ::std::uint32_t update_ver;
  ::std::uint32_t chain_ver;
  ::std::uint32_t checksum;
  ::std::uint32_t length;
  ::std::uint32_t offset;
  ::std::uint64_t data;
  ::std::uint64_t last_request_id;
  ::std::uint64_t last_client_low;
  ::std::uint64_t last_client_high;
  ::rust::Slice<::std::uint8_t const> expected_tag;
  ::rust::Slice<::std::uint8_t const> desired_tag;
  bool create_new;
  bool out_non_existent;
  ::std::uint16_t out_error_code;
  ::std::uint32_t out_commit_ver;
  ::std::uint32_t out_chain_ver;
  ::std::uint32_t out_checksum;

  bool operator==(UpdateReq const &) const noexcept;
  bool operator!=(UpdateReq const &) const noexcept;
  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$UpdateReq

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$GetReq
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$GetReq
struct GetReq final {
  ::rust::Slice<::std::uint8_t const> chunk_id;
  ::hf3fs::chunk_engine::Chunk const *chunk_ptr;

  bool operator==(GetReq const &) const noexcept;
  bool operator!=(GetReq const &) const noexcept;
  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$GetReq

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$RawMeta
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$RawMeta
struct RawMeta final {
  ::std::uint64_t pos;
  ::std::uint32_t chain_ver;
  ::std::uint32_t chunk_ver;
  ::std::uint32_t len;
  ::std::uint32_t checksum;
  ::std::uint64_t timestamp;
  ::std::uint64_t last_request_id;
  ::std::uint64_t last_client_low;
  ::std::uint64_t last_client_high;

  bool operator==(RawMeta const &) const noexcept;
  bool operator!=(RawMeta const &) const noexcept;
  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$RawMeta

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$RawUsedSize
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$RawUsedSize
struct RawUsedSize final {
  ::std::uint64_t allocated_size;
  ::std::uint64_t reserved_size;
  ::std::uint64_t position_count;
  ::std::uint64_t position_rc;

  bool operator==(RawUsedSize const &) const noexcept;
  bool operator!=(RawUsedSize const &) const noexcept;
  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$RawUsedSize

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$FdAndOffset
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$FdAndOffset
struct FdAndOffset final {
  ::std::int32_t fd;
  ::std::uint64_t offset;

  bool operator==(FdAndOffset const &) const noexcept;
  bool operator!=(FdAndOffset const &) const noexcept;
  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$FdAndOffset

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$Metrics
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$Metrics
struct Metrics final {
  ::std::uint64_t copy_on_write_times;
  ::std::uint64_t copy_on_write_latency;
  ::std::uint64_t copy_on_write_read_bytes;
  ::std::uint64_t copy_on_write_read_times;
  ::std::uint64_t copy_on_write_read_latency;
  ::std::uint64_t checksum_reuse;
  ::std::uint64_t checksum_combine;
  ::std::uint64_t checksum_recalculate;
  ::std::uint64_t safe_write_direct_append;
  ::std::uint64_t safe_write_indirect_append;
  ::std::uint64_t safe_write_truncate_shorten;
  ::std::uint64_t safe_write_truncate_extend;
  ::std::uint64_t safe_write_read_tail_times;
  ::std::uint64_t safe_write_read_tail_bytes;
  ::std::uint64_t allocate_times;
  ::std::uint64_t allocate_latency;
  ::std::uint64_t pwrite_times;
  ::std::uint64_t pwrite_latency;

  bool operator==(Metrics const &) const noexcept;
  bool operator!=(Metrics const &) const noexcept;
  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$Metrics

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$Engine
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$Engine
struct Engine final : public ::rust::Opaque {
  ::hf3fs::chunk_engine::RawUsedSize raw_used_size() const noexcept;
  ::std::size_t allocate_groups(::std::size_t min_remain, ::std::size_t max_remain, ::std::size_t batch_size) const noexcept;
  ::std::size_t allocate_ultra_groups(::std::size_t min_remain, ::std::size_t max_remain, ::std::size_t batch_size) const noexcept;
  ::std::size_t compact_groups(::std::uint64_t max_reserved) const noexcept;
  void set_allow_to_allocate(bool val) const noexcept;
  void speed_up_quit() const noexcept;
  ::hf3fs::chunk_engine::Chunk const *get_raw_chunk(::rust::Slice<::std::uint8_t const> chunk_id, ::std::string &error) const noexcept;
  void get_raw_chunks(::rust::Slice<::hf3fs::chunk_engine::GetReq > reqs, ::std::string &error) const noexcept;
  void release_raw_chunk(::hf3fs::chunk_engine::Chunk const *chunk) const noexcept;
  void release_writing_chunk(::hf3fs::chunk_engine::WritingChunk *chunk) const noexcept;
  ::hf3fs::chunk_engine::WritingChunk *update_raw_chunk(::rust::Slice<::std::uint8_t const> chunk_id, ::hf3fs::chunk_engine::UpdateReq &req, ::std::string &error) const noexcept;
  void commit_raw_chunk(::hf3fs::chunk_engine::WritingChunk *new_chunk, bool sync, ::std::string &error) const noexcept;
  void commit_raw_chunks(::rust::Slice<::hf3fs::chunk_engine::WritingChunk *const> reqs, bool sync, ::std::string &error) const noexcept;
  ::rust::Box<::hf3fs::chunk_engine::RawChunks> query_raw_chunks(::rust::Slice<::std::uint8_t const> begin, ::rust::Slice<::std::uint8_t const> end, ::std::uint64_t max_count, ::std::string &error) const noexcept;
  ::rust::Box<::hf3fs::chunk_engine::RawChunks> query_all_raw_chunks(::rust::Slice<::std::uint8_t const> prefix, ::std::string &error) const noexcept;
  ::rust::Box<::hf3fs::chunk_engine::RawChunks> query_raw_chunks_by_timestamp(::rust::Slice<::std::uint8_t const> prefix, ::std::uint64_t begin, ::std::uint64_t end, ::std::uint64_t max_count, ::std::string &error) const noexcept;
  ::std::uint64_t raw_batch_remove(::rust::Slice<::std::uint8_t const> begin, ::rust::Slice<::std::uint8_t const> end, ::std::uint64_t max_count, ::std::string &error) const noexcept;
  ::std::uint64_t query_raw_used_size(::rust::Slice<::std::uint8_t const> prefix, ::std::string &error) const noexcept;
  ::hf3fs::chunk_engine::Metrics get_metrics() const noexcept;
  ::rust::Box<::hf3fs::chunk_engine::RawChunks> query_uncommitted_raw_chunks(::rust::Slice<::std::uint8_t const> prefix, ::std::string &error) const noexcept;
  ::rust::Box<::hf3fs::chunk_engine::RawChunks> handle_uncommitted_raw_chunks(::rust::Slice<::std::uint8_t const> prefix, ::std::uint32_t chain_ver, ::std::string &error) const noexcept;
  ~Engine() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$Engine

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$LogGuard
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$LogGuard
struct LogGuard final : public ::rust::Opaque {
  ~LogGuard() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$LogGuard

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$Chunk
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$Chunk
struct Chunk final : public ::rust::Opaque {
  ::hf3fs::chunk_engine::RawMeta const &raw_meta() const noexcept;
  ::rust::Slice<::std::uint8_t const> raw_etag() const noexcept;
  bool uncommitted() const noexcept;
  ::hf3fs::chunk_engine::FdAndOffset fd_and_offset() const noexcept;
  ~Chunk() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$Chunk

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$WritingChunk
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$WritingChunk
struct WritingChunk final : public ::rust::Opaque {
  ::hf3fs::chunk_engine::RawMeta const &raw_meta() const noexcept;
  ::rust::Slice<::std::uint8_t const> raw_etag() const noexcept;
  bool uncommitted() const noexcept;
  ::hf3fs::chunk_engine::Chunk const *raw_chunk() const noexcept;
  void set_chain_ver(::std::uint32_t chain_ver) noexcept;
  ~WritingChunk() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$WritingChunk

#ifndef CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$RawChunks
#define CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$RawChunks
struct RawChunks final : public ::rust::Opaque {
  ::std::size_t len() const noexcept;
  ::rust::Slice<::std::uint8_t const> chunk_id(::std::size_t pos) const noexcept;
  ::hf3fs::chunk_engine::RawMeta const &chunk_meta(::std::size_t pos) const noexcept;
  ::rust::Slice<::std::uint8_t const> chunk_etag(::std::size_t pos) const noexcept;
  bool chunk_uncommitted(::std::size_t pos) const noexcept;
  ~RawChunks() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_hf3fs$chunk_engine$RawChunks

extern "C" {
bool hf3fs$chunk_engine$cxxbridge1$UpdateReq$operator$eq(UpdateReq const &, UpdateReq const &) noexcept;
bool hf3fs$chunk_engine$cxxbridge1$GetReq$operator$eq(GetReq const &, GetReq const &) noexcept;
bool hf3fs$chunk_engine$cxxbridge1$RawMeta$operator$eq(RawMeta const &, RawMeta const &) noexcept;
bool hf3fs$chunk_engine$cxxbridge1$RawUsedSize$operator$eq(RawUsedSize const &, RawUsedSize const &) noexcept;
bool hf3fs$chunk_engine$cxxbridge1$FdAndOffset$operator$eq(FdAndOffset const &, FdAndOffset const &) noexcept;
bool hf3fs$chunk_engine$cxxbridge1$Metrics$operator$eq(Metrics const &, Metrics const &) noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$Engine$operator$sizeof() noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$Engine$operator$alignof() noexcept;

::hf3fs::chunk_engine::Engine *hf3fs$chunk_engine$cxxbridge1$create(::rust::Str path, bool create, ::std::size_t prefix_len, ::std::string &error) noexcept;

void hf3fs$chunk_engine$cxxbridge1$release(::hf3fs::chunk_engine::Engine *engine) noexcept;

::hf3fs::chunk_engine::RawUsedSize hf3fs$chunk_engine$cxxbridge1$Engine$raw_used_size(::hf3fs::chunk_engine::Engine const &self) noexcept;

::std::size_t hf3fs$chunk_engine$cxxbridge1$Engine$allocate_groups(::hf3fs::chunk_engine::Engine const &self, ::std::size_t min_remain, ::std::size_t max_remain, ::std::size_t batch_size) noexcept;

::std::size_t hf3fs$chunk_engine$cxxbridge1$Engine$allocate_ultra_groups(::hf3fs::chunk_engine::Engine const &self, ::std::size_t min_remain, ::std::size_t max_remain, ::std::size_t batch_size) noexcept;

::std::size_t hf3fs$chunk_engine$cxxbridge1$Engine$compact_groups(::hf3fs::chunk_engine::Engine const &self, ::std::uint64_t max_reserved) noexcept;

void hf3fs$chunk_engine$cxxbridge1$Engine$set_allow_to_allocate(::hf3fs::chunk_engine::Engine const &self, bool val) noexcept;

void hf3fs$chunk_engine$cxxbridge1$Engine$speed_up_quit(::hf3fs::chunk_engine::Engine const &self) noexcept;

::hf3fs::chunk_engine::Chunk const *hf3fs$chunk_engine$cxxbridge1$Engine$get_raw_chunk(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::std::uint8_t const> chunk_id, ::std::string &error) noexcept;

void hf3fs$chunk_engine$cxxbridge1$Engine$get_raw_chunks(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::hf3fs::chunk_engine::GetReq > reqs, ::std::string &error) noexcept;

void hf3fs$chunk_engine$cxxbridge1$Engine$release_raw_chunk(::hf3fs::chunk_engine::Engine const &self, ::hf3fs::chunk_engine::Chunk const *chunk) noexcept;

void hf3fs$chunk_engine$cxxbridge1$Engine$release_writing_chunk(::hf3fs::chunk_engine::Engine const &self, ::hf3fs::chunk_engine::WritingChunk *chunk) noexcept;

::hf3fs::chunk_engine::WritingChunk *hf3fs$chunk_engine$cxxbridge1$Engine$update_raw_chunk(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::std::uint8_t const> chunk_id, ::hf3fs::chunk_engine::UpdateReq &req, ::std::string &error) noexcept;

void hf3fs$chunk_engine$cxxbridge1$Engine$commit_raw_chunk(::hf3fs::chunk_engine::Engine const &self, ::hf3fs::chunk_engine::WritingChunk *new_chunk, bool sync, ::std::string &error) noexcept;

void hf3fs$chunk_engine$cxxbridge1$Engine$commit_raw_chunks(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::hf3fs::chunk_engine::WritingChunk *const> reqs, bool sync, ::std::string &error) noexcept;

::hf3fs::chunk_engine::RawChunks *hf3fs$chunk_engine$cxxbridge1$Engine$query_raw_chunks(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::std::uint8_t const> begin, ::rust::Slice<::std::uint8_t const> end, ::std::uint64_t max_count, ::std::string &error) noexcept;

::hf3fs::chunk_engine::RawChunks *hf3fs$chunk_engine$cxxbridge1$Engine$query_all_raw_chunks(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::std::uint8_t const> prefix, ::std::string &error) noexcept;

::hf3fs::chunk_engine::RawChunks *hf3fs$chunk_engine$cxxbridge1$Engine$query_raw_chunks_by_timestamp(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::std::uint8_t const> prefix, ::std::uint64_t begin, ::std::uint64_t end, ::std::uint64_t max_count, ::std::string &error) noexcept;

::std::uint64_t hf3fs$chunk_engine$cxxbridge1$Engine$raw_batch_remove(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::std::uint8_t const> begin, ::rust::Slice<::std::uint8_t const> end, ::std::uint64_t max_count, ::std::string &error) noexcept;

::std::uint64_t hf3fs$chunk_engine$cxxbridge1$Engine$query_raw_used_size(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::std::uint8_t const> prefix, ::std::string &error) noexcept;

::hf3fs::chunk_engine::Metrics hf3fs$chunk_engine$cxxbridge1$Engine$get_metrics(::hf3fs::chunk_engine::Engine const &self) noexcept;

::hf3fs::chunk_engine::RawChunks *hf3fs$chunk_engine$cxxbridge1$Engine$query_uncommitted_raw_chunks(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::std::uint8_t const> prefix, ::std::string &error) noexcept;

::hf3fs::chunk_engine::RawChunks *hf3fs$chunk_engine$cxxbridge1$Engine$handle_uncommitted_raw_chunks(::hf3fs::chunk_engine::Engine const &self, ::rust::Slice<::std::uint8_t const> prefix, ::std::uint32_t chain_ver, ::std::string &error) noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$LogGuard$operator$sizeof() noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$LogGuard$operator$alignof() noexcept;

::hf3fs::chunk_engine::LogGuard *hf3fs$chunk_engine$cxxbridge1$init_log(::rust::Str path, ::std::string &error) noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$Chunk$operator$sizeof() noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$Chunk$operator$alignof() noexcept;

::hf3fs::chunk_engine::RawMeta const *hf3fs$chunk_engine$cxxbridge1$Chunk$raw_meta(::hf3fs::chunk_engine::Chunk const &self) noexcept;

::rust::repr::Fat hf3fs$chunk_engine$cxxbridge1$Chunk$raw_etag(::hf3fs::chunk_engine::Chunk const &self) noexcept;

bool hf3fs$chunk_engine$cxxbridge1$Chunk$uncommitted(::hf3fs::chunk_engine::Chunk const &self) noexcept;

::hf3fs::chunk_engine::FdAndOffset hf3fs$chunk_engine$cxxbridge1$Chunk$fd_and_offset(::hf3fs::chunk_engine::Chunk const &self) noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$WritingChunk$operator$sizeof() noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$WritingChunk$operator$alignof() noexcept;

::hf3fs::chunk_engine::RawMeta const *hf3fs$chunk_engine$cxxbridge1$WritingChunk$raw_meta(::hf3fs::chunk_engine::WritingChunk const &self) noexcept;

::rust::repr::Fat hf3fs$chunk_engine$cxxbridge1$WritingChunk$raw_etag(::hf3fs::chunk_engine::WritingChunk const &self) noexcept;

bool hf3fs$chunk_engine$cxxbridge1$WritingChunk$uncommitted(::hf3fs::chunk_engine::WritingChunk const &self) noexcept;

::hf3fs::chunk_engine::Chunk const *hf3fs$chunk_engine$cxxbridge1$WritingChunk$raw_chunk(::hf3fs::chunk_engine::WritingChunk const &self) noexcept;

void hf3fs$chunk_engine$cxxbridge1$WritingChunk$set_chain_ver(::hf3fs::chunk_engine::WritingChunk &self, ::std::uint32_t chain_ver) noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$RawChunks$operator$sizeof() noexcept;
::std::size_t hf3fs$chunk_engine$cxxbridge1$RawChunks$operator$alignof() noexcept;

::std::size_t hf3fs$chunk_engine$cxxbridge1$RawChunks$len(::hf3fs::chunk_engine::RawChunks const &self) noexcept;

::rust::repr::Fat hf3fs$chunk_engine$cxxbridge1$RawChunks$chunk_id(::hf3fs::chunk_engine::RawChunks const &self, ::std::size_t pos) noexcept;

::hf3fs::chunk_engine::RawMeta const *hf3fs$chunk_engine$cxxbridge1$RawChunks$chunk_meta(::hf3fs::chunk_engine::RawChunks const &self, ::std::size_t pos) noexcept;

::rust::repr::Fat hf3fs$chunk_engine$cxxbridge1$RawChunks$chunk_etag(::hf3fs::chunk_engine::RawChunks const &self, ::std::size_t pos) noexcept;

bool hf3fs$chunk_engine$cxxbridge1$RawChunks$chunk_uncommitted(::hf3fs::chunk_engine::RawChunks const &self, ::std::size_t pos) noexcept;
} // extern "C"

bool UpdateReq::operator==(UpdateReq const &rhs) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$UpdateReq$operator$eq(*this, rhs);
}

bool UpdateReq::operator!=(UpdateReq const &rhs) const noexcept {
  return !(*this == rhs);
}

bool GetReq::operator==(GetReq const &rhs) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$GetReq$operator$eq(*this, rhs);
}

bool GetReq::operator!=(GetReq const &rhs) const noexcept {
  return !(*this == rhs);
}

bool RawMeta::operator==(RawMeta const &rhs) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$RawMeta$operator$eq(*this, rhs);
}

bool RawMeta::operator!=(RawMeta const &rhs) const noexcept {
  return !(*this == rhs);
}

bool RawUsedSize::operator==(RawUsedSize const &rhs) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$RawUsedSize$operator$eq(*this, rhs);
}

bool RawUsedSize::operator!=(RawUsedSize const &rhs) const noexcept {
  return !(*this == rhs);
}

bool FdAndOffset::operator==(FdAndOffset const &rhs) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$FdAndOffset$operator$eq(*this, rhs);
}

bool FdAndOffset::operator!=(FdAndOffset const &rhs) const noexcept {
  return !(*this == rhs);
}

bool Metrics::operator==(Metrics const &rhs) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Metrics$operator$eq(*this, rhs);
}

bool Metrics::operator!=(Metrics const &rhs) const noexcept {
  return !(*this == rhs);
}

::std::size_t Engine::layout::size() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$operator$sizeof();
}

::std::size_t Engine::layout::align() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$operator$alignof();
}

::hf3fs::chunk_engine::Engine *create(::rust::Str path, bool create, ::std::size_t prefix_len, ::std::string &error) noexcept {
  return hf3fs$chunk_engine$cxxbridge1$create(path, create, prefix_len, error);
}

void release(::rust::Box<::hf3fs::chunk_engine::Engine> engine) noexcept {
  hf3fs$chunk_engine$cxxbridge1$release(engine.into_raw());
}

::hf3fs::chunk_engine::RawUsedSize Engine::raw_used_size() const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$raw_used_size(*this);
}

::std::size_t Engine::allocate_groups(::std::size_t min_remain, ::std::size_t max_remain, ::std::size_t batch_size) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$allocate_groups(*this, min_remain, max_remain, batch_size);
}

::std::size_t Engine::allocate_ultra_groups(::std::size_t min_remain, ::std::size_t max_remain, ::std::size_t batch_size) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$allocate_ultra_groups(*this, min_remain, max_remain, batch_size);
}

::std::size_t Engine::compact_groups(::std::uint64_t max_reserved) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$compact_groups(*this, max_reserved);
}

void Engine::set_allow_to_allocate(bool val) const noexcept {
  hf3fs$chunk_engine$cxxbridge1$Engine$set_allow_to_allocate(*this, val);
}

void Engine::speed_up_quit() const noexcept {
  hf3fs$chunk_engine$cxxbridge1$Engine$speed_up_quit(*this);
}

::hf3fs::chunk_engine::Chunk const *Engine::get_raw_chunk(::rust::Slice<::std::uint8_t const> chunk_id, ::std::string &error) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$get_raw_chunk(*this, chunk_id, error);
}

void Engine::get_raw_chunks(::rust::Slice<::hf3fs::chunk_engine::GetReq > reqs, ::std::string &error) const noexcept {
  hf3fs$chunk_engine$cxxbridge1$Engine$get_raw_chunks(*this, reqs, error);
}

void Engine::release_raw_chunk(::hf3fs::chunk_engine::Chunk const *chunk) const noexcept {
  hf3fs$chunk_engine$cxxbridge1$Engine$release_raw_chunk(*this, chunk);
}

void Engine::release_writing_chunk(::hf3fs::chunk_engine::WritingChunk *chunk) const noexcept {
  hf3fs$chunk_engine$cxxbridge1$Engine$release_writing_chunk(*this, chunk);
}

::hf3fs::chunk_engine::WritingChunk *Engine::update_raw_chunk(::rust::Slice<::std::uint8_t const> chunk_id, ::hf3fs::chunk_engine::UpdateReq &req, ::std::string &error) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$update_raw_chunk(*this, chunk_id, req, error);
}

void Engine::commit_raw_chunk(::hf3fs::chunk_engine::WritingChunk *new_chunk, bool sync, ::std::string &error) const noexcept {
  hf3fs$chunk_engine$cxxbridge1$Engine$commit_raw_chunk(*this, new_chunk, sync, error);
}

void Engine::commit_raw_chunks(::rust::Slice<::hf3fs::chunk_engine::WritingChunk *const> reqs, bool sync, ::std::string &error) const noexcept {
  hf3fs$chunk_engine$cxxbridge1$Engine$commit_raw_chunks(*this, reqs, sync, error);
}

::rust::Box<::hf3fs::chunk_engine::RawChunks> Engine::query_raw_chunks(::rust::Slice<::std::uint8_t const> begin, ::rust::Slice<::std::uint8_t const> end, ::std::uint64_t max_count, ::std::string &error) const noexcept {
  return ::rust::Box<::hf3fs::chunk_engine::RawChunks>::from_raw(hf3fs$chunk_engine$cxxbridge1$Engine$query_raw_chunks(*this, begin, end, max_count, error));
}

::rust::Box<::hf3fs::chunk_engine::RawChunks> Engine::query_all_raw_chunks(::rust::Slice<::std::uint8_t const> prefix, ::std::string &error) const noexcept {
  return ::rust::Box<::hf3fs::chunk_engine::RawChunks>::from_raw(hf3fs$chunk_engine$cxxbridge1$Engine$query_all_raw_chunks(*this, prefix, error));
}

::rust::Box<::hf3fs::chunk_engine::RawChunks> Engine::query_raw_chunks_by_timestamp(::rust::Slice<::std::uint8_t const> prefix, ::std::uint64_t begin, ::std::uint64_t end, ::std::uint64_t max_count, ::std::string &error) const noexcept {
  return ::rust::Box<::hf3fs::chunk_engine::RawChunks>::from_raw(hf3fs$chunk_engine$cxxbridge1$Engine$query_raw_chunks_by_timestamp(*this, prefix, begin, end, max_count, error));
}

::std::uint64_t Engine::raw_batch_remove(::rust::Slice<::std::uint8_t const> begin, ::rust::Slice<::std::uint8_t const> end, ::std::uint64_t max_count, ::std::string &error) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$raw_batch_remove(*this, begin, end, max_count, error);
}

::std::uint64_t Engine::query_raw_used_size(::rust::Slice<::std::uint8_t const> prefix, ::std::string &error) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$query_raw_used_size(*this, prefix, error);
}

::hf3fs::chunk_engine::Metrics Engine::get_metrics() const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Engine$get_metrics(*this);
}

::rust::Box<::hf3fs::chunk_engine::RawChunks> Engine::query_uncommitted_raw_chunks(::rust::Slice<::std::uint8_t const> prefix, ::std::string &error) const noexcept {
  return ::rust::Box<::hf3fs::chunk_engine::RawChunks>::from_raw(hf3fs$chunk_engine$cxxbridge1$Engine$query_uncommitted_raw_chunks(*this, prefix, error));
}

::rust::Box<::hf3fs::chunk_engine::RawChunks> Engine::handle_uncommitted_raw_chunks(::rust::Slice<::std::uint8_t const> prefix, ::std::uint32_t chain_ver, ::std::string &error) const noexcept {
  return ::rust::Box<::hf3fs::chunk_engine::RawChunks>::from_raw(hf3fs$chunk_engine$cxxbridge1$Engine$handle_uncommitted_raw_chunks(*this, prefix, chain_ver, error));
}

::std::size_t LogGuard::layout::size() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$LogGuard$operator$sizeof();
}

::std::size_t LogGuard::layout::align() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$LogGuard$operator$alignof();
}

::hf3fs::chunk_engine::LogGuard *init_log(::rust::Str path, ::std::string &error) noexcept {
  return hf3fs$chunk_engine$cxxbridge1$init_log(path, error);
}

::std::size_t Chunk::layout::size() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Chunk$operator$sizeof();
}

::std::size_t Chunk::layout::align() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Chunk$operator$alignof();
}

::hf3fs::chunk_engine::RawMeta const &Chunk::raw_meta() const noexcept {
  return *hf3fs$chunk_engine$cxxbridge1$Chunk$raw_meta(*this);
}

::rust::Slice<::std::uint8_t const> Chunk::raw_etag() const noexcept {
  return ::rust::impl<::rust::Slice<::std::uint8_t const>>::slice(hf3fs$chunk_engine$cxxbridge1$Chunk$raw_etag(*this));
}

bool Chunk::uncommitted() const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Chunk$uncommitted(*this);
}

::hf3fs::chunk_engine::FdAndOffset Chunk::fd_and_offset() const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$Chunk$fd_and_offset(*this);
}

::std::size_t WritingChunk::layout::size() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$WritingChunk$operator$sizeof();
}

::std::size_t WritingChunk::layout::align() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$WritingChunk$operator$alignof();
}

::hf3fs::chunk_engine::RawMeta const &WritingChunk::raw_meta() const noexcept {
  return *hf3fs$chunk_engine$cxxbridge1$WritingChunk$raw_meta(*this);
}

::rust::Slice<::std::uint8_t const> WritingChunk::raw_etag() const noexcept {
  return ::rust::impl<::rust::Slice<::std::uint8_t const>>::slice(hf3fs$chunk_engine$cxxbridge1$WritingChunk$raw_etag(*this));
}

bool WritingChunk::uncommitted() const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$WritingChunk$uncommitted(*this);
}

::hf3fs::chunk_engine::Chunk const *WritingChunk::raw_chunk() const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$WritingChunk$raw_chunk(*this);
}

void WritingChunk::set_chain_ver(::std::uint32_t chain_ver) noexcept {
  hf3fs$chunk_engine$cxxbridge1$WritingChunk$set_chain_ver(*this, chain_ver);
}

::std::size_t RawChunks::layout::size() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$RawChunks$operator$sizeof();
}

::std::size_t RawChunks::layout::align() noexcept {
  return hf3fs$chunk_engine$cxxbridge1$RawChunks$operator$alignof();
}

::std::size_t RawChunks::len() const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$RawChunks$len(*this);
}

::rust::Slice<::std::uint8_t const> RawChunks::chunk_id(::std::size_t pos) const noexcept {
  return ::rust::impl<::rust::Slice<::std::uint8_t const>>::slice(hf3fs$chunk_engine$cxxbridge1$RawChunks$chunk_id(*this, pos));
}

::hf3fs::chunk_engine::RawMeta const &RawChunks::chunk_meta(::std::size_t pos) const noexcept {
  return *hf3fs$chunk_engine$cxxbridge1$RawChunks$chunk_meta(*this, pos);
}

::rust::Slice<::std::uint8_t const> RawChunks::chunk_etag(::std::size_t pos) const noexcept {
  return ::rust::impl<::rust::Slice<::std::uint8_t const>>::slice(hf3fs$chunk_engine$cxxbridge1$RawChunks$chunk_etag(*this, pos));
}

bool RawChunks::chunk_uncommitted(::std::size_t pos) const noexcept {
  return hf3fs$chunk_engine$cxxbridge1$RawChunks$chunk_uncommitted(*this, pos);
}
} // namespace chunk_engine
} // namespace hf3fs

extern "C" {
::hf3fs::chunk_engine::Engine *cxxbridge1$box$hf3fs$chunk_engine$Engine$alloc() noexcept;
void cxxbridge1$box$hf3fs$chunk_engine$Engine$dealloc(::hf3fs::chunk_engine::Engine *) noexcept;
void cxxbridge1$box$hf3fs$chunk_engine$Engine$drop(::rust::Box<::hf3fs::chunk_engine::Engine> *ptr) noexcept;

::hf3fs::chunk_engine::RawChunks *cxxbridge1$box$hf3fs$chunk_engine$RawChunks$alloc() noexcept;
void cxxbridge1$box$hf3fs$chunk_engine$RawChunks$dealloc(::hf3fs::chunk_engine::RawChunks *) noexcept;
void cxxbridge1$box$hf3fs$chunk_engine$RawChunks$drop(::rust::Box<::hf3fs::chunk_engine::RawChunks> *ptr) noexcept;
} // extern "C"

namespace rust {
inline namespace cxxbridge1 {
template <>
::hf3fs::chunk_engine::Engine *Box<::hf3fs::chunk_engine::Engine>::allocation::alloc() noexcept {
  return cxxbridge1$box$hf3fs$chunk_engine$Engine$alloc();
}
template <>
void Box<::hf3fs::chunk_engine::Engine>::allocation::dealloc(::hf3fs::chunk_engine::Engine *ptr) noexcept {
  cxxbridge1$box$hf3fs$chunk_engine$Engine$dealloc(ptr);
}
template <>
void Box<::hf3fs::chunk_engine::Engine>::drop() noexcept {
  cxxbridge1$box$hf3fs$chunk_engine$Engine$drop(this);
}
template <>
::hf3fs::chunk_engine::RawChunks *Box<::hf3fs::chunk_engine::RawChunks>::allocation::alloc() noexcept {
  return cxxbridge1$box$hf3fs$chunk_engine$RawChunks$alloc();
}
template <>
void Box<::hf3fs::chunk_engine::RawChunks>::allocation::dealloc(::hf3fs::chunk_engine::RawChunks *ptr) noexcept {
  cxxbridge1$box$hf3fs$chunk_engine$RawChunks$dealloc(ptr);
}
template <>
void Box<::hf3fs::chunk_engine::RawChunks>::drop() noexcept {
  cxxbridge1$box$hf3fs$chunk_engine$RawChunks$drop(this);
}
} // namespace cxxbridge1
} // namespace rust
