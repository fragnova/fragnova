#!/bin/bash

ROOT=`pwd`
echo "ROOT is $ROOT"

echo

cd $ROOT/shards/shards-new &&

# we use this Docker container: https://hub.docker.com/r/chainblocks/shards
docker run --rm --user root --network host -v `pwd`:/spiderman chainblocks/shards shards /spiderman/run-create-proto-and-fragment.edn && # shards run-create-proto-and-fragment.edn &&

echo && sleep 10 &&

cd $ROOT/rpc &&
npm install &&
npm test
