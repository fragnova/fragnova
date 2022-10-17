#/bin/bash

apt-get update
apt-get install -y wget

cd /data/shards

wget -q https://dist.ipfs.tech/kubo/v0.16.0/kubo_v0.16.0_linux-amd64.tar.gz
tar -xvzf kubo_v0.16.0_linux-amd64.tar.gz
export PATH=$PATH:/data/shards/go-ipfs

ipfs init
ipfs config profile apply test
ipfs daemon &
sleep 5
shards test-protos-ipfs.edn
