#!/bin/bash

# storage 服务器快速配置脚本

apt update
apt install ibverbs-utils
ibv_devinfo
sudo sh -c "echo 'options erdma compat_mode=Y' >> /etc/modprobe.d/erdma.conf"
sudo rmmod erdma
sudo modprobe erdma compat_mode=Y

parted /dev/nvme0n1 mklabel gpt
parted /dev/nvme0n1 mkpart primary xfs 0% 100%
partprobe /dev/nvme0n1
mkfs.xfs -L data0 /dev/nvme0n1p1
mkdir -p /storage/data0
mount -o noatime,nodiratime -L data0 /storage/data0
 echo "LABEL=data0 /storage/data0 xfs noatime,nodiratime 0 0" >> /etc/fstab

df -kh | grep nvme
apt update
apt install docker.io -y


docker run -d \
 --network=host \
 --name storage \
 --ulimit memlock=-1 \
 --privileged \
 -v /storage:/storage \
 --device=/dev/infiniband/uverbs0 \
 --device=/dev/infiniband/rdma_cm \
 --env STORAGE_NODE_ID=10001 \
 --env TARGET_PATHS="/storage/data0/3fs" \
 --env REMOTE_IP="192.168.1.100:10000" \   # Storage自己的IP，根据Storage修改
 --env MGMTD_SERVER_ADDRESSES="RDMA://192.168.0.100:8000" \  # Meta节点的IP
  ac2-registry.cn-hangzhou.cr.aliyuncs.com/ac2/3fs:b71ffc55-fdb7.3.63-fuse3.16.2-ubuntu22.04 ./storage.sh

docker ps

echo "--------------------------------"
echo "storage 服务器快速配置脚本完成"