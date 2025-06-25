#!/bin/bash

# meta 服务器快速配置脚本

apt update
apt install ibverbs-utils
ibv_devinfo
sudo sh -c "echo 'options erdma compat_mode=Y' >> /etc/modprobe.d/erdma.conf"
sudo rmmod erdma
sudo modprobe erdma compat_mode=Y

apt update
apt install docker.io -y

docker run -d \
    --network=host \
    --name clickhouse-server \
    --ulimit nofile=262144:262144 \
    ac2-registry.cn-hangzhou.cr.aliyuncs.com/ac2/clickhouse:25.3.1.2703-ubuntu22.04

docker run -d \
    --network=host \
    --name fdb \
    ac2-registry.cn-hangzhou.cr.aliyuncs.com/ac2/fdb:7.3.63-ubuntu22.04

docker run -d \
    --network=host \
    --name monitor \
    --ulimit memlock=-1 \
    --privileged \
    --device=/dev/infiniband/uverbs0 \
    --device=/dev/infiniband/rdma_cm \
    ac2-registry.cn-hangzhou.cr.aliyuncs.com/ac2/3fs:b71ffc55-fdb7.3.63-fuse3.16.2-ubuntu22.04 ./monitor.sh

docker run -d \
    --network=host \
    --name mgmtd \
    --ulimit memlock=-1 \
    --privileged \
    --device=/dev/infiniband/uverbs0 \
    --device=/dev/infiniband/rdma_cm \
    --env FDB_CLUSTER="$(docker exec fdb cat /etc/foundationdb/fdb.cluster)" \
    --env REMOTE_IP="192.168.0.100:8000" \
    --env MGMTD_SERVER_ADDRESSES="RDMA://192.168.0.100:8000" \
    ac2-registry.cn-hangzhou.cr.aliyuncs.com/ac2/3fs:b71ffc55-fdb7.3.63-fuse3.16.2-ubuntu22.04 ./mgmtd.sh

docker run -d \
    --network=host \
    --name meta \
    --ulimit memlock=-1 \
    --privileged \
    --device=/dev/infiniband/uverbs0 \
    --device=/dev/infiniband/rdma_cm \
    ac2-registry.cn-hangzhou.cr.aliyuncs.com/ac2/3fs:b71ffc55-fdb7.3.63-fuse3.16.2-ubuntu22.04 ./meta.sh

docker ps

echo "--------------------------------"
echo "meta 服务器快速配置脚本完成"