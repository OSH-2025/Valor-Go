根据末尾链接文章和3fs github网站上的setup guide，大致总结了在阿里云ecs上搭建一个demo的流程和一些注意事项
# 1.创建实例并且生成基础镜像

首先需要创建编译实例用来构建3fs基础环境，此后的具体节点配置搭建在此编译环境所构成的基础上。

具体来说，使用阿里云ecs服务，申请一个ecs.g8i.4xlarge 的实例，在上面安装编译环境（3fs文档指定，使用apt 无法获取 apt-get 可以获取），rust工具，以及FoundationDB （下载太慢了，感觉下了两个小时）。

然后再将打包好的3fs文件（modified）clone下来，进行编译，之后就可以得到一个包含3fs的基础镜像（需要使用阿里云的镜像生成服务）。

##  1.1构建细节
1. 构建基础环境，具体的shell命令参考3fs github官网
```bash
apt install cmake libuv1-dev liblz4-dev liblzma-dev libdouble-conversion-dev libprocps-dev libdwarf-dev libunwind-dev \
  libaio-dev libgflags-dev libgoogle-glog-dev libgtest-dev libgmock-dev clang-format-14 clang-14 clang-tidy-14 lld-14 \
  libgoogle-perftools-dev google-perftools libssl-dev ccache gcc-12 g++-12 libboost-all-dev
```

2. 下载fuse，注意这一步可能会比较慢，稍加等待即可
```bash
wget https://github.com/libfuse/libfuse/releases/download/fuse-3.16.1/fuse-3.16.1.tar.gz
tar vzxf fuse-3.16.1.tar.gz
cd fuse-3.16.1/
mkdir build; cd build
apt install meson
meson setup ..
ninja ; ninja install
```

3. 下载rust支持。这一步与我们在自己电脑环境上下载rust基本相同，参考流程如下
```bash
	export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static 
	export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
	#然后使用官网上的下载curl指令
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
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

# 2.使用生成镜像进行各节点配置
在本测试中，使用1个meta节点，3个storage节点进行配置，其中很多功能共同运行在meta节点上，注意这里因为只有mgmtd节点

| nodeid | name     | monitor | admin client | mgmtd | meta | storage | fuse client |
| ------ | -------- | ------- | ------------ | ----- | ---- | ------- | ----------- |
| 1，100  | meta     | y       | y            | y     | y    | n       | y*          |
| 10001  | storage0 | n       | y            | n     | n    | y       | n           |
| 10002  | storage1 | n       | y            | n     | n    | y       | n           |
| 10003  | storage2 | n       | y            | n     | n    | y       | n           |
>Tips: 对于fuse client，可以搭载在meta node 上，用以进行简易测试，也可以在外部独立节点搭建，用以模拟高并发环境下的情况

## 2.1 搭建meta节点
1. 将eRDMA模式改为1 
```Bash
rmmod erdma
odprobe erdma compat_mode=1
```
2. 修改mxsge
```Bash
cd ~/3fs/configs
sed -i 's/max_sge = 16/max_sge = 1/g' `grep -rl max_sge`
```

3. 创建脚本，支持mellanox网卡

```Bash
#!/bin/bash
echo "erdma_0 port 1 ==> eth0 (Up)"
chmod +x /usr/sbin/ibdev2netdev
```

4. 将meta对应的ip填入每个节点的/etc/hosts，这一步用在以后的rsync操作中，用来简化meta节点的ip地址的填写，但在我们实际配置中，使用这个的操作可以用root@（实际IP地址），并且有关于ip的操作中，更加麻烦的是更改各种config中的ip地址以及端口

5. 安装clickhouse
```bash
sudo apt-get install -y apt-transport-https ca-certificates curl gnupg
curl -fsSL 'https://packages.clickhouse.com/rpm/lts/repodata/repomd.xml.key' | sudo gpg --dearmor -o /usr/share/keyrings/clickhouse-keyring.gpg
 
ARCH=$(dpkg --print-architecture)
echo "deb [signed-by=/usr/share/keyrings/clickhouse-keyring.gpg arch=${ARCH}] https://packages.clickhouse.com/deb stable main" | sudo tee /etc/apt/sources.list.d/clickhouse.list
sudo apt-get update
 
sudo apt-get install -y clickhouse-server clickhouse-client
#在安装的时候会要求输入密码, 任意指定即可
sudo clickhouse start 
#开启服务

clickhouse-client --password 'your password'
#client启动
clickhouse-client --password 'your password' -n  < ~/3fs/deploy/sql/3fs-monitor.sql
#创建metric table
```
4. 配置monitor服务。在我们的配置过程中，有时会出现，启动失败的情况，报错core - dumped，但在我们执行了这段代码后，启动正常，*不过不排除启动需要时间，而我们在没有启动的时候就查看状态了*
```bash
rmmod erdma
odprobe erdma compat_mode=1
```
```bash
mkdir -p /opt/3fs/{bin,etc}
mkdir -p /var/log/3fs
cp ~/3fs/build/bin/monitor_collector_main /opt/3fs/bin
cp ~/3fs/configs/monitor_collector_main.toml /opt/3fs/etc
#创建一个运行位置

vim /opt/3fs/etc/monitor_collector_main.toml
#下面是修改内容
[server.monitor_collector.reporter.clickhouse]
db = '3fs'
host = '127.0.0.1'
passwd = 'eRDMA123!!'
port = '9000'
user = 'default'

cp ~/3fs/deploy/systemd/monitor_collector_main.service /usr/lib/systemd/system
#cpy进system空间
systemctl start monitor_collector_main
systemctl status monitor_collector_main 
#start and check
```
7. 配置Admin Client 服务
    这里需要添加一下关于rsync的说明，通常需要使用ssh进行连接，也就是说需要**第一22端口的开放，第二密钥中的公钥传递到~/.ssh/authorizedkeys中**。但是阿里云申请的ecs实例在ssh权限方面有问题，导致ssh相关操作会失败,经过比对，authorized_keys 中只有阿里云上自建的密钥对信息。这是源自sshd.config的问题，阿里云会将config中的大部分注释掉，需要手动进行改动设置，比如如下的比较重要的几个设置，其中最下面的图片中改为yes，可以使用ssh-copy-id 自动传递公钥，如果不改，需要手动将公钥写入authorized_keys 中。[参考](https://blog.csdn.net/yxyc666/article/details/142331896)

```Bash
# on server A
#如果希望使用meta代替ip地址，可能需要清除旧的痕迹
ssh-keygen -f "/root/.ssh/known_hosts" -R "meta"

#first
nano /etc/ssh/sshd_config

PubkeyAuthentication yes
AuthorizedKeysFile .ssh/authorized_keys .ssh/authorized_keys2
PasswordAuthentication yes

# 去控制台设置实例的密码

#then
systemctl restart sshd

# on client B
ssh-keygen -t rsa
ssh-copy-id root@$(ip_address)#meta节点的ip地址

#rsync .......
```
tips：之后使用rsync meta：也可以写成 rsync root@meta： 或者是 rsync root@（实际ip地址）
参考起见，这是我使用参考文章里面命令的例子
```bash
mkdir -p /opt/3fs/{bin,etc}
rsync -avz meta:~/3fs/build/bin/admin_cli /opt/3fs/bin
rsync -avz meta:~/3fs/configs/admin_cli.toml /opt/3fs/etc
rsync -avz meta:/etc/foundationdb/fdb.cluster /opt/3fs/etc
#这里用到rsync，可以在多台主机之间互通，方便通讯

vim /opt/3fs/etc/admin_cli.toml

#修改内容如下
cluster_id = "stage"
 
[fdb]
clusterFile = '/opt/3fs/etc/fdb.cluster'
```
8. 配置mgmtd服务，我们会出现配置失败的情况，有以下两种我们尝试过的方法，~~后者好像更有效一点~~
```bash
cp ~/3fs/deploy/systemd/mgmtd_main.service /usr/lib/systemd/system
#cp 命令 做了对于system空间的更改，需要以下命令进行重载
systemctl daemon-reload
#如果未有warning提醒，并且没有使用上面的命令可能会出现fail
#反之，如果出现warning并且按照这样更改，那么就会成功
```
事实上这只是一方面，另一方面是，阿里云的erdma需要每次都重新配置，需要
```bash
# 卸载
rmmod erdma
#重装 
modprobe erdma compat_mode=1
```
参考流程如下：
```bash
cp ~/3fs/build/bin/mgmtd_main /opt/3fs/bin
cp ~/3fs/configs/{mgmtd_main.toml,mgmtd_main_launcher.toml,mgmtd_main_app.toml} /opt/3fs/etc

vim /opt/3fs/etc/mgmtd_main_app.toml
##修改
node_id = 1

vim /opt/3fs/etc/mgmtd_main_launcher.toml
##修改
cluster_id = "stage"
 
[fdb]
clusterFile = '/opt/3fs/etc/fdb.cluster'

vim /opt/3fs/etc/mgmtd_main.toml
##修改
[common.monitor.reporters.monitor_collector]
remote_ip = "10.99.0.1:10000"#这里填入meta节点机器的ip地址


/opt/3fs/bin/admin_cli -cfg /opt/3fs/etc/admin_cli.toml "init-cluster --mgmtd /opt/3fs/etc/mgmtd_main.toml 1 1048576 16"
 #初始化集群

cp ~/3fs/deploy/systemd/mgmtd_main.service /usr/lib/systemd/system
systemctl start mgmtd_main
systemctl status mgmtd_main
#start and check
```
9. 配置meta service
```bash
cp ~/3fs/build/bin/meta_main /opt/3fs/bin
cp ~/3fs/configs/{meta_main_launcher.toml,meta_main.toml,meta_main_app.toml} /opt/3fs/etc

vim /opt/3fs/etc/meta_main_app.toml
##修改
node_id = 100

vim /opt/3fs/etc/meta_main_launcher.toml
##修改
cluster_id = "stage"
 
[mgmtd_client]
mgmtd_server_addresses = ["RDMA://10.99.0.1:8000"]#这里填入meta节点机器的ip地址

vim /opt/3fs/etc/mgmtd_main.toml
##修改
[server.mgmtd_client]
mgmtd_server_addresses = ["RDMA://10.99.0.1:8000"]

[common.monitor.reporters.monitor_collector]
remote_ip = "10.99.0.1:10000"#这里填入meta节点机器的ip地址

[server.fdb]
clusterFile = '/opt/3fs/etc/fdb.cluster'

/opt/3fs/bin/admin_cli -cfg /opt/3fs/etc/admin_cli.toml --config.mgmtd_client.mgmtd_server_addresses '["RDMA://10.99.0.1:8000"]' "set-config --type META --file /opt/3fs/etc/meta_main.toml"

cp ~/3fs/deploy/systemd/meta_main.service /usr/lib/systemd/system
systemctl start meta_main
systemctl status meta_main
#start and check
```

## 2.2 配置storage节点
1. rmmod等代码（同从2.1开始的若干步一直到clickhouse）
2. 配置admin client，这里可能会出现/var/log/3fs 未找到的问题，直接mkdir -p 即可
3. 根据自己的节点数，盘数调整，来配置storage service
```bash
mkdir -p /storage/data{0..7}
mkdir -p /var/log/3fs
for i in {0..7};do mkfs.xfs -L data${i} /dev/nvme${i}n1;mount -o noatime,nodiratime -L data${i} /storage/data${i};done
mkdir -p /storage/data{0..7}/3fs
#根据SSD数量调整
sysctl -w fs.aio-max-nr=67108864
```
紧接着参考文章的步骤是改动meta节点中的原始配置文件，这一步我认为是为了简化配置过程，因为每个storage节点的配置需求相同，因此先将配置文件写到meta中，再通过rsync复制，那么配置多个节点的时候只需要rsync即可。具体见下
```bash
#在meta中进行修改，后续只需要进行rsync即可
vim ~/3fs/configs/storage_main_launcher.toml
#修改内容
cluster_id = "stage"
 
[mgmtd_client]
mgmtd_server_addresses = ["RDMA://10.99.0.1:8000"]#adjust

vim ~/3fs/configs/storage_main.toml
#修改内容
[server.mgmtd]
mgmtd_server_address = ["RDMA://10.99.0.1:8000"]
 
[common.monitor.reporters.monitor_collector]
remote_ip = "10.99.0.1:10000"
 
[server.targets]
target_paths = ["/storage/data0/3fs","/storage/data1/3fs","/storage/data2/3fs","/storage/data3/3fs","/storage/data4/3fs","/storage/data5/3fs","/storage/data6/3fs","/storage/data7/3fs"]#根据盘数调整

#之后每个storage节点进行
rsync -avz meta:~/3fs/build/bin/storage_main /opt/3fs/bin
rsync -avz meta:~/3fs/configs/{storage_main_launcher.toml,storage_main.toml,storage_main_app.toml} /opt/3fs/etc

vim /opt/3fs/etc/storage_main_app.toml
#修改对应的node-id

/opt/3fs/bin/admin_cli -cfg /opt/3fs/etc/admin_cli.toml --config.mgmtd_client.mgmtd_server_addresses '["RDMA://10.99.0.1:8000"]' "set-config --type STORAGE --file /opt/3fs/etc/storage_main.toml"

rsync -avz meta:~/3fs/deploy/systemd/storage_main.service /usr/lib/systemd/system
systemctl start storage_main
systemctl status storage_main
#start and check
```
## 2.3 配置meta节点之配置3FS
 参考流程
 ```bash
 /opt/3fs/bin/admin_cli -cfg /opt/3fs/etc/admin_cli.toml --config.mgmtd_client.mgmtd_server_addresses '["RDMA://10.99.0.1:8000"]' "user-add --root --admin 0 root"

#将输出信息中的token（不包含括号内的内容）加入/opt/3fs/etc/token.txt保存
pip3 install -r ~/3fs/deploy/data_placement/requirements.txt
#安装依赖

 ```
具体见参考链接。这里需要根据自己的ecs配置进行设置，以我们的存储是申请了3个节点，每个节点一个SSD盘（超级丐版），这里的num-nodes需要按照这样更改。以下的两段代码是在分配物理存储的映射关系，与3fs使用的CR和CRAQ有关，target是最小的单元。
```bash
python3 ~/3fs/deploy/data_placement/src/model/data_placement.py \
  -ql -relax -type CR --num_nodes 3 --replication_factor 3 --min_targets_per_disk 1
```
```bash
 python3 ~/3fs/deploy/data_placement/src/setup/gen_chain_table.py \
   --chain_table_type CR --node_id_begin 10001 --node_id_end 10003 \ 
   --num_disks_per_node 1 --num_targets_per_disk 1 \
   --target_id_prefix 1 --chain_id_prefix 1 \
   --incidence_matrix_path output/DataPlacementModel-v_3-b_1-r_1-k_3-λ_1-lb_1-ub_0/incidence_matrix.pickle
```

以上均运行在/opt/3fs/下，或者说output会产生在当前目录下，只要能在后面的命令中找到就可以

出现未知错误，可能由于admin client功能莫名奇妙失去挂载（可能由于服务器停机）。也可能是因为storage 节点断开链接或者未成功链接导致target（到物理SSD的映射）（必须要保证已经建立的节点数和node id begin 到 node id end 相匹配） 

另一个问题是如何选择命令的参数，这里我们使用的是每个存储节点1 $\cdot$ 3GIB 的配置,(教程中是每个节点8$\cdot$ 3GIB)，这里会有一个target_per_disk 的参数，我们选择除了1以外的数都会报错：parameter infeasible，*因此**可能** 需要将这个参数设置为小于等于每个节点的盘数*

之后的流程：
```bash
/opt/3fs/bin/admin_cli --cfg /opt/3fs/etc/admin_cli.toml --config.mgmtd_client.mgmtd_server_addresses '["RDMA://10.99.0.1:8000"]' --config.user_info.token $(<"/opt/3fs/etc/token.txt") < output/create_target_cmd.txt
#这一步出错可能需要检查storage节点是否失联，因为这一步是将逻辑映射到物理的步骤，需要storage
/opt/3fs/bin/admin_cli --cfg /opt/3fs/etc/admin_cli.toml --config.mgmtd_client.mgmtd_server_addresses '["RDMA://10.99.0.1:8000"]' --config.user_info.token $(<"/opt/3fs/etc/token.txt") "upload-chains output/generated_chains.csv"
 
/opt/3fs/bin/admin_cli --cfg /opt/3fs/etc/admin_cli.toml --config.mgmtd_client.mgmtd_server_addresses '["RDMA://10.99.0.1:8000"]' --config.user_info.token $(<"/opt/3fs/etc/token.txt") "upload-chain-table --desc stage 1 output/generated_chain_table.csv"
 
```
在这一步之后，主体分布式结构就已经搭建好了

## 2.4 配置FUSE Client
按照参考配置即可。我们在配置过程中阴差阳错将fuse没有配置到独立节点而是配置到了meta节点上，但最后也运行成功了。经过查资料，这样多个功能放在同一节点的方式可以用于小规模简易测试，而全部分散到多个独立节点可以更加贴近生产环境。

最后：十分感谢末尾的两篇资料还有更多网络上的资料，在配置过程中，我在其中受益良多，因此，推荐配合参考文章进行具体的实践。
reference1：[基于eRDMA实测DeepSeek开源的3FS_3fs编译-CSDN博客](https://blog.csdn.net/weixin_43778179/article/details/145995349)
reference2：[DeepSeek 3FS部署最佳实践_3fs 部署-CSDN博客](https://blog.csdn.net/Franklin7B/article/details/146308170)