#/bin/bash

apt-get update
apt-get install -y curl

curl -O https://dist.ipfs.io/go-ipfs/v0.10.0/go-ipfs_v0.10.0_linux-amd64.tar.gz
tar -xvzf go-ipfs_v0.10.0_linux-amd64.tar.gz
export PATH=$PATH:/data/go-ipfs

ipfs init
ipfs daemon &

cbl /data/chains/fragments-tests.edn