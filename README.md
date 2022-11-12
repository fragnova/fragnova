# Clamor

[![license](https://img.shields.io/github/license/fragcolor-xyz/clamor)](./LICENSE)
![CI](https://github.com/fragcolor-xyz/clamor/workflows/CI/badge.svg)
[![codecov](https://codecov.io/gh/fragcolor-xyz/clamor/branch/devel/graph/badge.svg?token=4PMT2FQFDS)](https://codecov.io/gh/fragcolor-xyz/clamor)
[![docs](https://img.shields.io/badge/docs-API-blueviolet)](https://fragcolor-xyz.github.io/clamor/)

Clamor is a custom blockchain built on the [Substrate](https://substrate.io/) framework.

It is a protocol and networking stack that enables complete on-chain storage and full synchronization of asset data (protos, fragments, chainblocks code etc.) across the blockchain nodes.

### Rustdocs

The Rustdocs can be read at: https://fragcolor-xyz.github.io/clamor/

## Requirements

Before you start, ensure you've [set up your development environment](https://docs.substrate.io/install/) and [installed Rust](https://www.rust-lang.org/tools/install).

*NOTE - The following instructions are for developing Clamor on Linux (also on WSL) /Mac since Substrate does not yet have a reliable [native Windows support](https://docs.substrate.io/v3/getting-started/windows-users/).*
## Build
### Update system packages

```
# To build the project
cargo build
```

## Run a local node

Run the following command from the root folder of Clamor project:
```
RUST_LOG=bitswap=trace,pallet_protos::pallet=trace,pallet_frag::pallet=trace,pallet_fragments::pallet=trace cargo run -- --dev --tmp --rpc-external --rpc-port 9933 --rpc-cors all --ws-external --enable-offchain-indexing 1 --rpc-methods=Unsafe --ipfs-server
```

If you want to run the Clamor node with a [chain specification](https://docs.substrate.io/v3/runtime/chain-specs/) instead, use this script:

```
cargo run -- --chain=spec_raw.json --validator --rpc-external --rpc-port 9933 --rpc-cors all --ws-external --enable-offchain-indexing 1 --rpc-methods=Unsafe --ipfs-server -d <DATA PATH>
```

## Run the unit tests
To run all the unit tests:
```
cargo test
```
## Run the rpc tests in Javascript
To run the Javascript rpc tests, which calls the APIs provided by Clamor and are located in `/clamor/rpc/test/rpc-tests.js` you need to:
1. run the local node (described [above](#run-a-local-node))
2. upload some test data into the node (described [below](#usage))
3. `cd /rpc && npm install && npm test`

## Usage

### Uploading test data into the node using Docker

Once you have a Clamor node running locally you can programmatically upload test data to it with the following script (using the Docker image of [Shards](https://docs.fragcolor.xyz/shards/)):

```
docker run --rm --user root --network host -v `pwd`:/data chainblocks/shards shards /data/shards/add-test-assets.edn
```
### Uploading test data into the node without Docker

*NOTE - This requires to have Shards built and installed locally to be ran as an executable. Check out the [official Shards documentation](https://docs.fragcolor.xyz/contribute/code/building-shards/).*

Once you have installed [Shards](https://docs.fragcolor.xyz/shards/) locally, just go to the /shards folder and run:
```
>:~/clamor/shards$ shards add-test-assets.edn
```
### Connecting to Polkadot's App Explorer

[Polkadot.js](https://github.com/polkadot-js/) provides a browser based application, [App Explorer](https://polkadot.js.org/apps/#/explorer) (also available as hosted IPFS and IPNS versions). This application allows you to interact with your locally running Substrate node, with minimal setup.

To do this:

1. Run you Clamor node locally
2. Head over to the [App Explorer](https://polkadot.js.org/apps/#/explorer)
3. Click the top-left Pokadot icon on the header of the page
4. Expand the **Development** sub-menu (at the bottom of the list)
5. Click **Local Node** to enable it
6. Click **Switch** at the top of the panel

The App Explorer will now connect with your local node and will show the blocks being produced by your node in real-time.

### Setting up a testnet/mainnet genesis

Build to make sure wasm runtime is uptodate

```
cargo build --release
```

Build the spec, in order to generate the json spec we need to grab stuff from

```
./target/release/clamor build-spec > spec.json
```

Grab `"system"` the wasm runtime and paste it into your template, in our case `testnet.json`

Produce a raw spec

```
./target/release/clamor build-spec --chain testnet.json --raw > testnet-raw.json
```

Run the validator with permissive external rpcs in order to add "aura" and "gran" keys calling author_insertKey rpc

```
 ./target/release/clamor --node-key-file p2p-node.key --chain testnet-raw.json --ipfs-server --validator --enable-offchain-indexing 1 --rpc-methods=Unsafe --rpc-external --rpc-cors all --ws-external --port 30337
```

Now run again in a more restrictive environment, also including rpc/bootstrap known nodes

```
./target/release/clamor --node-key-file p2p-node.key --chain testnet-raw.json --ipfs-server --validator --enable-offchain-indexing 1 --bootnodes /ip4/20.225.200.219/tcp/30337/ws/p2p/12D3KooWQoQhtVUT8j2hV7dXrFpf3pp4Q5FT7c3GdAf2wiKACjD6 --port 30337
```
