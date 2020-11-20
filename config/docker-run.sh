#!/usr/bin/env bash

# ===================== RUN OPERATIONAL TXS BENCHMARKS ==========================
# DB Read benchmark
echo Running... DB Read benchmark
/provenance-node/target/release/node-bench '::trie::read::large' &> ./database-read.txt

# DB Write benchmark
echo Running... DB Write benchmark
/provenance-node/target/release/node-bench '::trie::write::large' &> ./database-write.txt

# Empty Block Construction
echo Running... Empty Block Construction
/provenance-node/target/release/node-bench '::node::import::wasm::sr25519::noop::rocksdb::empty' &> ./empty-block-construction.txt

# Extrinsic Overhead
echo Running... Extrinsic Overhead
/provenance-node/target/release/node-bench '::node::import::wasm::sr25519::noop::rocksdb::custom' --transactions 10000 &> ./extrinsic-overhead.txt


# # ==================== RUN NON-OPERATIONAL TXS BENCHMARKS =======================

# ----------------------------- PROVENANCE LEDGER ---------------------------------
# add_activity_group
echo Running... add_activity_group
/provenance-node/target/release/substrate benchmark --chain dev \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet provenance_ledger \
    --extrinsic add_activity_group \
    --steps 50 \
    --repeat 200000 \
    --raw &> ./provenance_add_activity_group.txt

# ----------------------------- PALLET BALANCES -----------------------------------
# transfer
echo Running... transfer
/provenance-node/target/release/substrate benchmark --chain dev \
    --execution=wasm --wasm-execution=compiled \
    --pallet pallet_balances \
    --extrinsic transfer \
    --steps 50 \
    --repeat 200000 \
    --raw &> ./balances_transfer.txt

# transfer_keep_alive
echo Running... transfer_keep_alive
/provenance-node/target/release/substrate benchmark --chain dev \
    --execution=wasm --wasm-execution=compiled \
    --pallet pallet_balances \
    --extrinsic transfer_keep_alive \
    --steps 50 \
    --repeat 200000 \
    --raw &> ./balances_transfer_keep_alive.txt

# set_balance_creating
echo Running... set_balance_creating
/provenance-node/target/release/substrate benchmark --chain dev \
    --execution=wasm --wasm-execution=compiled \
    --pallet pallet_balances \
    --extrinsic set_balance_creating \
    --steps 50 \
    --repeat 200000 \
    --raw &> ./balances_set_balance_creating.txt

# set_balance_killinge
echo Running... set_balance_killinge
/provenance-node/target/release/substrate benchmark --chain dev \
    --execution=wasm --wasm-execution=compiled \
    --pallet pallet_balances \
    --extrinsic set_balance_killing \
    --steps 50 \
    --repeat 200000 \
    --raw &> ./balances_set_balance_killing.txt

# force_transfer
echo Running... force_transfer
/provenance-node/target/release/substrate benchmark --chain dev \
    --execution=wasm --wasm-execution=compiled \
    --pallet pallet_balances \
    --extrinsic force_transfer \
    --steps 50 \
    --repeat 200000 \
    --raw &> ./balances_force_transfer.txt


# ==================== RUN THE NODE======================================

FILE=/provenance-node/config/chainSpecRaw.json
if test -f "$FILE"; then
    echo "chainSpecRaw.json provided"
else
  /provenance-node/target/release/node-template build-spec --chain=/provenance-node/config/chainSpec.json --raw --disable-default-bootnode > /provenance-node/config/chainSpecRaw.json
fi

nohup /provenance-node/target/release/node-template \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
	--base-path /tmp/validator_1 \
	--chain=/provenance-node/config/chainSpecRaw.json \
	--port 30333 \
  --pruning 256 \
	--ws-port 9944 \
	--rpc-port 9933 \
  --rpc-cors all \
	--validator \
  --ws-external \
  --rpc-external \
	--rpc-methods=Unsafe \
  --offchain-worker WhenValidating \
	--name node_validator &

sleep 5

curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d "@/provenance-node/config/aura-keys.json"

curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d "@/provenance-node/config/grandpa-keys.json"

curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d "@/provenance-node/config/offchain-worker-fqs!-keys.json"

kill $(ps aux | grep '[t]arget/release/node-template' | awk '{print $2}')

rm -rf ./provenance-node/config/aura-keys.json

rm -rf ./provenance-node/config/grandpa-keys.json

/provenance-node/target/release/node-template \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
	--base-path /tmp/validator_1 \
	--chain=/provenance-node/config/chainSpecRaw.json \
	--port 30333 \
	--ws-port 9944 \
  --pruning 256 \
	--rpc-port 9933 \
  --rpc-cors all \
	--validator \
  --ws-external \
  --rpc-external \
	--rpc-methods=Unsafe \
  --offchain-worker WhenValidating /
	--name node_validator
