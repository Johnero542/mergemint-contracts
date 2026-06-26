#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
set -euo pipefail

NETWORK="${1:-sepolia}"
ACCOUNT="${2:-default}"

echo "Building contract..."
cargo build --release --target wasm32-unknown-unknown

echo "Deploying to $NETWORK..."
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/mergemint_contracts.wasm \
  --network "$NETWORK" \
  --source-account "$ACCOUNT"

echo "Deployment complete!"
