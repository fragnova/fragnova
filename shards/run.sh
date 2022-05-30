#/bin/bash

apt-get update
apt-get install -y wget

cd /data/chains

wget -q https://dist.ipfs.io/go-ipfs/v0.10.0/go-ipfs_v0.10.0_linux-amd64.tar.gz
tar -xvzf go-ipfs_v0.10.0_linux-amd64.tar.gz
export PATH=$PATH:/data/chains/go-ipfs

ipfs init
ipfs config profile apply test
ipfs daemon &
sleep 5
shards test-protos-ipfs.edn