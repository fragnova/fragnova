#/bin/bash

apt-get update
sudo apt-get install -y ca-certificates curl gnupg lsb-release wget
sudo mkdir -p /etc/apt/keyrings
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin

cd /data/shards

wget -q https://dist.ipfs.tech/kubo/v0.16.0/kubo_v0.16.0_linux-amd64.tar.gz &&
tar -xvzf kubo_v0.16.0_linux-amd64.tar.gz &&
export PATH=$PATH:/data/shards/kubo &&

# Initializes ipfs configuration files and generates a new keypair. (https://docs.ipfs.tech/how-to/command-line-quick-start/#initialize-the-repository)
ipfs init &&
# we are using the "test" profile, which "Reduces external interference of IPFS daemon. Useful when using the daemon in test environments." (https://docs.ipfs.tech/how-to/default-profile/#available-profiles)
ipfs config profile apply test &&
# The daemon will start listening on ports on the network, which are documented in (and can be modified through) 'ipfs config Addresses'
ipfs daemon &
sleep 5 &&

# we use this Docker container: https://hub.docker.com/r/chainblocks/shards
docker run --rm --user root --network host -v `pwd`:/la-casa chainblocks/shards shards /la-casa/shards-new/run-test-ipfs.edn  #shards test-protos-ipfs.edn
