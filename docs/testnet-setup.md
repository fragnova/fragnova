# Testnet setup

## Steps (dirty)

* run `./target/release/fragnova build-spec > testnet2.json`
* grab the wasm artifact and paste into `testnet.json` (`system`)
* save, don't push `testnet.json`
* generate raw with `./target/release/fragnova build-spec --chain testnet.json --raw > testnet-raw-v2.json` (*this is the genesis state*)

### On the block producer node
* Create a libp2p key into `p2p-node.key` with `...`
* Run first with this line to add the private key `./target/release/fragnova --node-key-file p2p-node.key --chain testnet-raw-v2.json --ipfs-server --validator --enable-offchain-indexing 1 --rpc-cors all --rpc-methods=Unsafe --port 30337`
* Add the keys now with
  * `./target/release/fragnova key insert --key-type sudo --scheme sr25519 --chain testnet-raw-v2.json`
  * `./target/release/fragnova key insert --key-type aura --scheme sr25519 --chain testnet-raw-v2.json`
  * `./target/release/fragnova key insert --key-type frag --scheme ed25519 --chain testnet-raw-v2.json`
  * `./target/release/fragnova key insert --key-type gran --scheme ed25519 --chain testnet-raw-v2.json`
* For further safety run the node without unsafe rpc now as: `./target/release/fragnova --node-key-file p2p-node.key --chain testnet-raw-v2.json --ipfs-server --validator --enable-offchain-indexing 1 --rpc-cors all --port 30337`, might also add a bootstrap as: `--bootnodes /ip4/20.225.200.219/tcp/30337/p2p/12D3KooWQoQhtVUT8j2hV7dXrFpf3pp4Q5FT7c3GdAf2wiKACjD6`
