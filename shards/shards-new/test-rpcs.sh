#!/bin/bash

ROOT=`pwd`
echo "ROOT is $ROOT"

echo

cd $ROOT/shards/shards-new &&
if [ 10 -gt 234 ]
then
  shards run-create-proto-and-fragment.edn
else
  # we use this Docker container: https://hub.docker.com/r/chainblocks/shards
  docker run --rm --user root --network host -v `pwd`:/spiderman chainblocks/shards shards /spiderman/run-create-proto-and-fragment.edn
fi &&

echo && sleep 10 &&

cd $ROOT/rpc &&
npm install &&
npm test
