reference1：[基于eRDMA实测DeepSeek开源的3FS_3fs编译-CSDN博客](https://blog.csdn.net/weixin_43778179/article/details/145995349)
reference2：[DeepSeek 3FS部署最佳实践_3fs 部署-CSDN博客](https://blog.csdn.net/Franklin7B/article/details/146308170)

第二篇基本上是第一篇的翻版，本人根据此文章和3fs github网站上的setup guide，大致总结了在阿里云ecs上搭建一个demo的流程和一些注意事项
# 一、 创建实例并且生成基础镜像

首先需要创建编译实例用来构建3fs基础环境，此后的具体节点配置搭建在此编译环境所构成的基础上。

具体来说，使用阿里云ecs服务，申请一个ecs.g8i.4xlarge 的实例，在上面安装编译环境（3fs文档指定，使用apt 无法获取 apt-get 可以获取），rust工具，以及FoundationDB （下载太慢了，感觉下了两个小时）。

然后再将打包好的3fs文件（modified）clone下来，进行编译，之后就可以得到一个包含3fs的基础镜像（需要使用阿里云的镜像生成服务）。

##  构建细节
1. 构建基础环境，具体的shell命令参考3fs github官网

2. 下载fuse，注意这一步可能会比较慢，稍加等待即可

3. 下载rust支持。这一步与我们在自己电脑环境上下载rust基本相同，参考流程如下
```bash
	export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static 
	export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
	#然后使用官网上的下载curl指令
	source $HOME/.cargo/env
	#source之后重启shell使设置成立
```

4. 下载foundationdb，这一步会耗时很长，原因是github/apple的连接传输过慢，经常下载速度在10KIB/S以下，即使调高带宽，我的解决方法是在本地先下载foundaiondb的稳定版本（此时为ver 7.3.63），再通过阿里云的上传文件功能将deb包传输上去

5. 编译3fs，这一步是耗时最长的一步
``` bash
git clone https://github.com/deepseek-ai/3fs
cd 3fs
git submodule update --init --recursive
./patches/apply.sh
cmake -S . -B build -DCMAKE_CXX_COMPILER=clang++-14 -DCMAKE_C_COMPILER=clang-14 -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
cmake --build build -j
```
	这里可能有两个问题，第一个是cargo下载库较慢，另一个是编译过程中内存占用过高导致LINUX OOM kill，表现结果是编译进度卡死。第一个的解决方法是配置国内rust镜像源，第二个的解决方法是要么提高配置，要么降低编译时的并行数（-j参数），以我的例子来说，使用了ecs.g8i.16xlarge,并行数为4,这样是过于保守的做法，代价是编译速度缓慢。

6. 生成3fs编译镜像，这一步比较简单，直接使用阿里云平台上的创建自定义镜像即可（镜像是个好功能，可以多用用） 