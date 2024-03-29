# Fragnova

[![license](https://img.shields.io/github/license/fragcolor-xyz/fragnova)](./LICENSE)
![CI](https://github.com/fragcolor-xyz/fragnova/workflows/CI/badge.svg)
[![codecov](https://codecov.io/gh/fragcolor-xyz/fragnova/branch/devel/graph/badge.svg?token=4PMT2FQFDS)](https://codecov.io/gh/fragcolor-xyz/fragnova)
[![docs](https://img.shields.io/badge/docs-API-blueviolet)](https://fragcolor-xyz.github.io/fragnova/)

Fragnova is a custom blockchain built on the [Substrate](https://substrate.io/) framework.

It is a protocol and networking stack that enables complete on-chain storage and full synchronization of asset data (protos, fragments, shards scripts etc.) across the blockchain nodes.

## Requirements

Before you start, ensure you've [set up your development environment](https://docs.substrate.io/install/) and [installed Rust](https://www.rust-lang.org/tools/install).

*NOTE - The following instructions are for developing Fragnova on Linux (also on WSL) / Mac since Substrate does not yet have a reliable [native Windows support](https://docs.substrate.io/v3/getting-started/windows-users/).*
## Build
### Update system packages

```
# To build the project
cargo build
```

## Run a local node

Run the following command from the root folder of Fragnova project:
```
RUST_LOG=bitswap=trace,pallet_protos::pallet=trace,pallet_frag::pallet=trace,pallet_fragments::pallet=trace cargo run -- --dev --tmp --rpc-external --rpc-port 9933 --rpc-cors all --ws-external --enable-offchain-indexing true --rpc-methods=Unsafe --pool-kbytes 200000
```

If you want to run the Fragnova node with a [chain specification](https://docs.substrate.io/v3/runtime/chain-specs/) instead, use this script:

```
cargo run -- --chain=spec_raw.json --validator --rpc-external --rpc-port 9933 --rpc-cors all --ws-external --enable-offchain-indexing true --rpc-methods=Unsafe -d <DATA PATH>
```

## Usage

### Connecting to Polkadot's App Explorer

[Polkadot.js](https://github.com/polkadot-js/) provides a browser based application, [App Explorer](https://polkadot.js.org/apps/#/explorer) (also available as hosted IPFS and IPNS versions). This application allows you to interact with your locally running Substrate node, with minimal setup.

To do this:

1. Run you Fragnova node locally
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
./target/release/fragnova build-spec > spec.json
```

Grab `"system"` the wasm runtime and paste it into your template, in our case `testnet.json`

Produce a raw spec

```
./target/release/fragnova build-spec --chain testnet.json --raw > testnet-raw.json
```

Run the validator with permissive external rpcs in order to add "aura" and "gran" keys calling author_insertKey rpc

```
 ./target/release/fragnova --node-key-file p2p-node.key --chain testnet-raw.json --ipfs-server --validator --enable-offchain-indexing true --rpc-methods=Unsafe --rpc-external --rpc-cors all --ws-external --port 30337
```

Now run again in a more restrictive environment, also including rpc/bootstrap known nodes

```
./target/release/fragnova --node-key-file p2p-node.key --chain testnet-raw.json --ipfs-server --validator --enable-offchain-indexing true --bootnodes /ip4/20.225.200.219/tcp/30337/ws/p2p/12D3KooWQoQhtVUT8j2hV7dXrFpf3pp4Q5FT7c3GdAf2wiKACjD6 --port 30337
```
