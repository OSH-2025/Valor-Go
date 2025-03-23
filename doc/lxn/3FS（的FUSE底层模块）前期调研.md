**0.大模型Deepseek的3FS文件系统是以FUSE（用户空间文件系统）作为底层架构的，故需要对FUSE有所了解**
**1.FUSE文件系统原理解释**
5分钟搞懂用户空间文件系统FUSE工作原理：用户可通过fuse在用户空间来定制实现自己的文件系统。
https://zhuanlan.zhihu.com/p/106719192
FUSE文件系统介绍
https://blog.csdn.net/weixin_45525272/article/details/121298434
**2.代码实战（学习使用FUSE创建个人文件系统）**
吴锦华 / 明鑫 : 用户态文件系统 ( FUSE ) 框架分析和实战
https://cloud.tencent.com/developer/article/1006138

FUSE 用户空间文件系统 （Filesystem in Userspace）:代码实现了一个简单的 FUSE (Filesystem in Userspace) 文件系统，名为 "hello" 文件系统。它会展示一个虚拟文件系统，其中包含一个名为 hello 的文件，文件内容是 "Hello World!"。
https://cloud.tencent.com/developer/article/1766726

分布式文件存储基座Fuse的深度解析：给出了部分fuse调用代码并详细分析，同时还有优化方向及当前进展，如：FUSE max_pages，FUSE_CAP_WRITEBACK_CACHE，FUSE_CAP_PARALLEL_DIROPS，FUSE_PASSTHROUGH
单文件支持并发写，多线程，splice
https://zhuanlan.zhihu.com/p/675783819

**3.对FUSE的已有优化（可以作为优化3FS的思路参考）**
XFUSE: An Infrastructure for Running Filesystem Services in User Space
https://zhuanlan.zhihu.com/p/671680806