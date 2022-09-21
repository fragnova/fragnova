#!/bin/bash

ROOT=`pwd`
echo "ROOT is $ROOT"

echo

cd $ROOT/shards/shards-new &&
shards run-create-proto-and-fragment.edn &&

echo && sleep 10 &&

cd $ROOT/rpc &&
npm install &&
npm test
