reference1：[基于eRDMA实测DeepSeek开源的3FS_3fs编译-CSDN博客](https://blog.csdn.net/weixin_43778179/article/details/145995349)
reference2：[DeepSeek 3FS部署最佳实践_3fs 部署-CSDN博客](https://blog.csdn.net/Franklin7B/article/details/146308170)

第二篇基本上是第一篇的翻版
# 创建实例并且生成基础镜像

首先需要创建编译实例用来构建3fs基础环境，此后的具体节点配置搭建在此编译环境所构成的基础上。

具体来说，使用阿里云ecs服务，申请一个ecs.g8i.4xlarge 的实例，在上面安装编译环境（3fs文档指定，使用apt 无法获取 apt-get 可以获取），rust工具，以及FoundationDB （下载太慢了，感觉下了两个小时）。

然后再将打包好的3fs文件（modified）clone下来，进行编译，之后就可以得到一个包含3fs的基础镜像（需要使用阿里云的镜像生成服务）。

# 构建细节