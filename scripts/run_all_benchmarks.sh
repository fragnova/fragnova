#!/usr/bin/env bash

# This Clamor bash script is inspired from: https://github.com/paritytech/substrate/blob/master/scripts/run_all_benchmarks.sh

cargo build --profile=production --features runtime-benchmarks

# The executable to use.
CLAMOR=./target/production/clamor

PALLETS=("pallet_accounts" "pallet_detach" "pallet_fragments" "pallet_protos")


# Define the error file.
ERR_FILE="benchmarking_errors.txt"
# Delete the error file before each run.
rm -f $ERR_FILE

# Benchmark each pallet.
for PALLET in ${PALLETS[@]}
do
  FOLDER="$(echo "${PALLET#*_}" | tr '_' '-')"
  WEIGHT_FILE="./pallets/${FOLDER}/src/weights.rs"
  echo "[+] Benchmarking $PALLET with weight file $WEIGHT_FILE"

  OUTPUT=$(
    $CLAMOR benchmark pallet \
    --chain=dev \
    --steps=50 \ # across the defined component(s)/variable(s) range, take 50 samples. Note: No two samples can have the exact same set of component values
    --repeat=20 \ # execute the benchmark on each sample 20 times
    --pallet="$PALLET" \
    --extrinsic="*" \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output="$WEIGHT_FILE" \
    --template=./.maintain/frame-weight-template.hbs 2>&1
  )
  if [ $? -ne 0 ]; then
    echo "$OUTPUT" >> "$ERR_FILE"
    echo "[-] Failed to benchmark $PALLET. Error written to $ERR_FILE; continuing..."
  fi
done
