# Getting Started with MergeMint Contracts

## Prerequisites

- Rust (stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Stellar CLI: `cargo install stellar-cli`
- WASM target: `rustup target add wasm32-unknown-unknown`

## Setup

1. **Generate testnet account:**
   ```bash
   stellar keys generate testnet-account
   ```

2. **Fund with Friendbot:**
   ```bash
   stellar account fund testnet-account --network testnet
   ```

3. **Build contract:**
   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```

## Deploy to Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/mergemint_contracts.wasm \
  --network testnet \
  --source-account testnet-account
```

Save the contract ID from output.

## Test Contract

```bash
cargo test
```

## Deploy Example Bounty

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source-account testnet-account \
  -- create_bounty \
  --creator testnet-account \
  --title "Fix_bug" \
  --description "Fix_auth_issue" \
  --reward_amount 1000000 \
  --reward_token <USDC_ADDRESS> \
  --min_reputation 0
```

Replace `<CONTRACT_ID>` with your deployed contract ID and `<USDC_ADDRESS>` with actual token address.
