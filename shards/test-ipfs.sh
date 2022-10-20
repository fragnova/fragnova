#!/bin/bash
set -e

# INSTALL IPFS
apt-get update
sudo apt-get install -y wget
wget -q https://dist.ipfs.tech/kubo/v0.16.0/kubo_v0.16.0_linux-amd64.tar.gz
tar -xvzf kubo_v0.16.0_linux-amd64.tar.gz
export PATH=$PATH:`pwd`/kubo

# RUN IPFS
ipfs init # Initializes ipfs configuration files and generates a new keypair. (https://docs.ipfs.tech/how-to/command-line-quick-start/#initialize-the-repository)
ipfs config profile apply test # we are using the "test" profile, which "Reduces external interference of IPFS daemon. Useful when using the daemon in test environments." (https://docs.ipfs.tech/how-to/default-profile/#available-profiles)
ipfs daemon & # The daemon will start listening on ports on the network, which are documented in (and can be modified through) 'ipfs config Addresses'
sleep 5

# RUN SHARDS SCRIPT "run-test-ipfs.edn"
shards /DUMMY-FOLDER/run-test-ipfs.edn  # we use this Docker container: https://hub.docker.com/r/chainblocks/shards
