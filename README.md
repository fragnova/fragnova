# TBA

## Run
```sh
RUST_LOG=bitswap=trace,protos=debug cargo run -- --dev --tmp --rpc-external --rpc-port 9933 --rpc-cors all --ws-external --enable-offchain-indexing 1 --rpc-methods=Unsafe --ipfs-server --storage-chain
```

## Run with spec
```sh
RUST_LOG=bitswap=trace,protos=debug cargo run -- --chain=spec_raw.json --validator --rpc-external --rpc-port 9933 --rpc-cors all --ws-external --enable-offchain-indexing 1 --rpc-methods=Unsafe --ipfs-server --storage-chain -d <DATA PATH>
```

## Populate test assets via docker/cbl
```sh
docker run --rm --user root --network host -v `pwd`:/data chainblocks/cbl cbl /data/chains/add-test-assets.edn
```