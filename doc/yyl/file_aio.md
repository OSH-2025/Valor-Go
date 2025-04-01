# 关联库与函数
- folly
	- random.h
	- executors/CPUThreadPoolExecutor.h
- common/utils
	- BoundedQueue.h
	- ConfigBase.h
	- Result.h

# file list
- AioReadWorker
- AioStatus
- BatchReadJob

# AioReadWorker
## h file
主要声明一个class：AioReadWorker, 包含
### enumclass： IoEngine
可能是表示IO读写方式依赖于何种方式
- libaio（AIO）异步读
- io_uring：linux下的异步IO API
- random，（随机?）

### class: Config --基于ConfigBase<T>的派生类
```c
class Config : public ConfigBase<Config> {}
```
TIPS：使用[CRTP](https://blog.csdn.net/qq_39354847/article/details/127576222)编程技术

---

```c
	CONFIG_ITEM(num_threads, 32ul);
    CONFIG_ITEM(queue_size, 4096u);
    CONFIG_ITEM(max_events, 512u);
    CONFIG_ITEM(enable_io_uring, true);
    CONFIG_HOT_UPDATED_ITEM(min_complete, 128u);
    CONFIG_HOT_UPDATED_ITEM(wait_all_inflight, false);      // deprecated.
    CONFIG_HOT_UPDATED_ITEM(inflight_control_offset, 128);  // deprecated.
    CONFIG_HOT_UPDATED_ITEM(ioengine, IoEngine::libaio);
```
涉及到CONFIG_ITEM and CONFIG_HOT_UPDATED_ITEM functions,(定义在哪里？)
>这里猜测是进行配置文件的范围定义

```c
	inline bool useIoUring() const {
      if (!enable_io_uring()) {
        return false;
      }
      switch (ioengine()) {
        case IoEngine::io_uring:
          return true;
        case IoEngine::libaio:
          return false;
        case IoEngine::random:
          return folly::Random::rand32() & 1;
      }
	}
```
函数useIouring()表示是否使用io_uring API 实现，要么!enable,要么通过switch判定ioengin()
> 这里可以猜测folly：：Random可能实现了某种随机生成函数，通过&1生成0,1二者之中的随机值


>[!question]
>**enable_io_uring()** 定义
>**ioengine()**定义





