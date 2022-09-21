#!/bin/bash

ROOT=`pwd`
echo "ROOT is $ROOT"

echo

cd $ROOT/shards/shards-new &&
if [ -z "$GITHUB_WORKSPACE" ]
then
  shards run-create-proto-and-fragment.edn
else
  docker run --rm --user root --network host -v ${GITHUB_WORKSPACE}:/data chainblocks/shards shards test-protos-rpc.edn
fi &&

echo && sleep 10 &&

cd $ROOT/rpc &&
npm install &&
npm test
