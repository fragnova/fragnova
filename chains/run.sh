#/bin/bash

PATH=$PATH:/data/go-ipfs
ipfs init
ipfs daemon &
cbl /data/chains/fragments-tests.edn