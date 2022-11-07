#!/usr/bin/env bash

# This Clamor bash script is inspired from: https://github.com/paritytech/substrate/blob/master/scripts/run_all_benchmarks.sh

cargo build --profile=production --features runtime-benchmarks

# The executable to use.
CLAMOR=./target/production/clamor

PALLETS=("pallet_accounts" "pallet_fragments" "pallet_protos") # "pallet_detach")


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

  # The option "--step=50" takes 50 samples for every benchmark test across the defined variable(s) range.
  # Note: No two samples can have the exact same set of values for all the variables
  #
  # The option "--repeat=20" executes the benchmark test on each sample 20 times
  OUTPUT=$(
    $CLAMOR benchmark pallet \
    --chain=dev \
    --steps=50 \
    --repeat=20 \
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
