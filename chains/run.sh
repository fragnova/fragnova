#/bin/bash

wget https://dist.ipfs.io/go-ipfs/v0.10.0/go-ipfs_v0.10.0_linux-amd64.tar.gz
tar -xvzf go-ipfs_v0.10.0_linux-amd64.tar.gz
PATH=$PATH:/data/go-ipfs
ipfs init
ipfs daemon &
cbl /data/chains/fragments-tests.edn