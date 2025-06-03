# 3FS部署指南
## PB23111729 吕祖灿

目前主要参考：https://zhuanlan.zhihu.com/p/27571661405
更多参考文档见项目文档

### 硬件检查
1. 查询电脑是否支持NVme-SSD
2. RDMA是否需要支持待定

### Docker安装
+ 参考文档：https://blog.csdn.net/u011278722/article/details/137673353
+ 注意事项：
  + 使用daocloud，不要使用官方路径（国内无法正常访问，开梯子也没成功）

### 3fs Dockerfile构建
+ Dockerfile学习参考文档：https://www.runoob.com/docker/docker-dockerfile.html
+ 具体操作参考文档：https://zhuanlan.zhihu.com/p/27571661405
