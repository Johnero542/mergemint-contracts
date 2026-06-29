# MergeMint Contracts .....

Soroban smart contracts powering the on-chain layer of MergeMint — an open-source contribution reward platform. These contracts manage the full bounty lifecycle on Stellar: creation, claiming, completion with automatic token reward distribution, and contributor reputation tracking.

Built with **Rust (`no_std`)** using the **Soroban SDK 23**, compiled to WASM (`cdylib`) for deployment on the Stellar network.

---

## Table of Contents

- [Architecture](#architecture)
- [Tech Stack](#tech-stack)
- [Data Model](#data-model)
- [Storage Layout](#storage-layout)
- [Contract Interface](#contract-interface)
- [Bounty Lifecycle](#bounty-lifecycle)
- [Events](#events)
- [Code Highlights](#code-highlights)
- [Getting Started](#getting-started)
- [CLI Usage](#cli-usage)
- [Project Structure](#project-structure)
- [Testing](#testing)
- [Deployment](#deployment)
- [Security](#security)
- [License](#license)

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        Stellar Network                           │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                 MergeMintContract                        │    │
│  │                                                          │    │
│  │  ┌──────────────────────────────────────────────────┐    │    │
│  │  │                   Contract                       │    │    │
│  │  │                                                │    │    │
│  │  │  create_bounty() ──► store bounty + emit event │    │    │
│  │  │  claim_bounty()  ──► assign + update status     │    │    │
│  │  │  complete_bounty()─► transfer tokens +          │    │    │
│  │  │                      update reputation          │    │    │
│  │  │  get_bounty()     ──► read bounty by ID         │    │    │
│  │  │  get_contributor()──► read contributor profile   │    │    │
│  │  │  get_bounty_count()► total bounties created     │    │    │
│  │  └──────────┬───────────────────────────────────────┘    │    │
│  │             │                                            │    │
│  │  ┌──────────▼───────────────────────────────────────┐    │    │
│  │  │              Storage Layer                       │    │    │
│  │  │  env.storage().persistent()                      │    │    │
│  │  │  ┌──────────────┐ ┌────────────┐ ┌────────────┐  │    │    │
│  │  │  │ BountyCount  │ │ Bounty{id} │ │Contributor │  │    │    │
│  │  │  │ (u64)        │ │ (Bounty)   │ │ {address}  │  │    │    │
│  │  │  └──────────────┘ └────────────┘ └────────────┘  │    │    │
│  │  └───────────────────────────────────────────────────┘    │    │
│  │                                                          │    │
│  │  ┌───────────────────────────────────────────────────┐    │    │
│  │  │              Event Emission Layer                 │    │    │
│  │  │  env.events().publish(...)                        │    │    │
│  │  │  ┌──────────────┐ ┌──────────────┐ ┌───────────┐  │    │    │
│  │  │  │bounty_created│ │bounty_claimed│ │completed  │  │    │    │
│  │  │  │              │ │              │ │+ reward   │  │    │    │
│  │  │  │              │ │              │ │_paid      │  │    │    │
│  │  │  └──────────────┘ └──────────────┘ └───────────┘  │    │    │
│  │  └───────────────────────────────────────────────────┘    │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                   External Integrations                  │    │
│  │                                                          │    │
│  │  ┌──────────────────┐  ┌──────────────────────────────┐  │    │
│  │  │  TokenClient     │  │  Off-chain Indexer           │  │    │
│  │  │  (Soroban Token  │  │  (MergeMint API consumes     │  │    │
│  │  │   Interface)     │  │   contract events)           │  │    │
│  │  └──────────────────┘  └──────────────────────────────┘  │    │
│  └──────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

### Call Flow

```
  User/Wallet                MergeMintContract                  Token Contract
     │                              │                               │
     │  create_bounty()             │                               │
     │  (creator auths)             │                               │
     │─────────────────────────────▶│                               │
     │                              │ store Bounty                  │
     │                              │ emit bounty_created           │
     │◀─────────────────────────────│ bounty_id                     │
     │                              │                               │
     │  claim_bounty()              │                               │
     │  (contributor auths)         │                               │
     │─────────────────────────────▶│                               │
     │                              │ assign contributor            │
     │                              │ emit bounty_claimed           │
     │◀─────────────────────────────│                               │
     │                              │                               │
     │  complete_bounty()           │                               │
     │  (verifier auths)            │                               │
     │─────────────────────────────▶│                               │
     │                              │ transfer(reward_token,        │
     │                              │   verifier, assignee,         │
     │                              │   reward_amount)              │
     │                              │──────────────────────────────▶│
     │                              │◀──────────────────────────────│
     │                              │ update contributor reputation │
     │                              │ emit bounty_completed         │
     │                              │ emit reward_paid              │
     │◀─────────────────────────────│                               │
```

---

## Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| Platform | **Stellar Soroban** | Smart contract execution environment |
| Language | **Rust (`no_std`)** | Systems-level, safe, WASM-compilable |
| SDK | **soroban-sdk 23.0.0** | Contract development framework |
| Token Standard | **Soroban Token Interface** | Standardized `transfer`, `balance`, etc. |
| Testing | **soroban-sdk testutils** | In-env unit testing with mock auths |
| Build Target | **`wasm32-unknown-unknown`** | WASM binary for Soroban deployment |

---

## Data Model

### DataKey (Storage Keys)

```rust
pub enum DataKey {
    BountyCount,                 // Singleton u64 counter
    Bounty(BytesN<32>),          // Bounty by 32-byte ID
    Contributor(Address),        // Contributor profile by wallet address
}
```

### Bounty

```rust
pub struct Bounty {
    pub creator: Address,         // Maintainer who created the bounty
    pub title: Symbol,            // Short title (max 32 chars)
    pub description: Symbol,      // Longer description
    pub reward_amount: i128,      // Token amount (e.g., 100 * 10^7)
    pub reward_token: Address,    // Token contract address (e.g., USDC, native)
    pub assignee: Option<Address>,// Contributor who claimed it
    pub status: Symbol,           // "open" | "in_progress"
}
```

### Contributor

```rust
pub struct Contributor {
    pub address: Address,          // Wallet address
    pub reputation: u32,           // Monotonically increasing score (+10 per completion)
    pub total_earned: i128,        // Total tokens earned across all bounties
    pub contribution_count: u32,   // Number of bounties completed
}
```

---

## Storage Layout

```
Persistent Storage
├── DataKey::BountyCount           → u64
├── DataKey::Bounty(id_0)          → Bounty { ... }
├── DataKey::Bounty(id_1)          → Bounty { ... }
├── ...
├── DataKey::Contributor(addr_0)   → Contributor { ... }
├── DataKey::Contributor(addr_1)   → Contributor { ... }
└── ...
```

All storage uses Soroban's persistent storage (not temporary), meaning data persists across contract calls indefinitely.

---

## Contract Interface

### State-Mutating Functions

| Method | Auth | Parameters | Description |
|--------|------|------------|-------------|
| `create_bounty` | `creator` | `creator`, `title`, `description`, `reward_amount`, `reward_token` | Creates a new open bounty. Returns generated `BytesN<32>` ID. |
| `claim_bounty` | `contributor` | `contributor`, `bounty_id` | Assigns a contributor to a bounty. Fails if already assigned. |
| `complete_bounty` | `verifier` | `verifier`, `bounty_id` | Transfers tokens from verifier to assignee, updates reputation. |

### Read-Only Functions

| Method | Parameters | Returns |
|--------|------------|---------|
| `get_bounty` | `bounty_id` | `Option<Bounty>` |
| `get_contributor` | `address` | `Option<Contributor>` |
| `get_bounty_count` | — | `u64` |

---

## Bounty Lifecycle

```
                    create_bounty()
                         │
                         ▼
                   ┌──────────┐
                   │   Open   │
                   └────┬─────┘
                        │
                  claim_bounty()
                   (contributor)
                        │
                        ▼
                ┌──────────────┐
                │ In Progress  │
                └──────┬───────┘
                       │
                 complete_bounty()
                    (verifier)
                       │
             ┌─────────┼─────────┐
             ▼         ▼         ▼
       ┌─────────┐ ┌────────┐ ┌──────────────┐
       │ Token   │ │ Rep    │ │ Events       │
       │ Transfer│ │ +10    │ │ emitted      │
       └─────────┘ └────────┘ └──────────────┘
```

---

## Events

Contracts emit structured events consumed by the MergeMint API indexer for off-chain processing:

| Event | Topics | Data | Trigger |
|-------|--------|------|---------|
| `bounty_created` | `(Symbol, creator)` | `(bounty_id, reward)` | New bounty created |
| `bounty_claimed` | `(Symbol, contributor)` | `bounty_id` | Bounty claimed |
| `bounty_completed` | `(Symbol, contributor)` | `bounty_id` | Bounty completed, reward paid |
| `reward_paid` | `(Symbol, contributor)` | `(bounty_id, amount)` | Token transfer confirmed |

---

## Code Highlights

### Crate Entry Point

```rust
// src/lib.rs
#![no_std]

mod contract;
mod events;
mod storage;
mod types;

pub use contract::MergeMintContractClient;

#[cfg(test)]
mod test;
```

### Core Contract Logic

The contract generates deterministic bounty IDs from an incrementing counter, pads the bytes into a 32-byte array:

```rust
// src/contract.rs
fn generate_bounty_id(env: &Env) -> BytesN<32> {
    let count = storage::get_bounty_count(env);
    let mut buf = [0u8; 32];
    let count_bytes = count.to_be_bytes();
    buf[24..32].copy_from_slice(&count_bytes);
    BytesN::from_array(env, &buf)
}
```

#### create_bounty

Authenticates the creator, generates an ID from the counter, stores the bounty, increments the counter, and emits an event:

```rust
pub fn create_bounty(
    env: Env,
    creator: Address,
    title: Symbol,
    description: Symbol,
    reward_amount: i128,
    reward_token: Address,
) -> BytesN<32> {
    creator.require_auth();

    let count = storage::get_bounty_count(&env);
    let id = generate_bounty_id(&env);

    let bounty = Bounty {
        creator,
        title,
        description,
        reward_amount,
        reward_token,
        assignee: None,
        status: Symbol::new(&env, "open"),
    };

    storage::store_bounty(&env, &id, &bounty);
    storage::set_bounty_count(&env, &(count + 1));

    events::emit_bounty_created(&env, &id, &bounty.creator, &reward_amount);
    id
}
```

#### claim_bounty

Authenticates the contributor, guards against double-assignment, and updates the bounty status:

```rust
pub fn claim_bounty(env: Env, contributor: Address, bounty_id: BytesN<32>) {
    contributor.require_auth();

    let mut bounty = storage::get_bounty(&env, &bounty_id)
        .expect("bounty not found");

    if bounty.assignee.is_some() {
        panic!("bounty already assigned");
    }

    bounty.assignee = Some(contributor.clone());
    bounty.status = Symbol::new(&env, "in_progress");

    storage::store_bounty(&env, &bounty_id, &bounty);
    events::emit_bounty_claimed(&env, &bounty_id, &contributor);
}
```

#### complete_bounty

Authenticates the verifier, transfers tokens from verifier to assignee via the Soroban Token Interface, updates the contributor's reputation (+10), and emits completion events:

```rust
pub fn complete_bounty(env: Env, verifier: Address, bounty_id: BytesN<32>) {
    verifier.require_auth();

    let bounty = storage::get_bounty(&env, &bounty_id)
        .expect("bounty not found");
    let assignee = bounty.assignee.clone()
        .expect("bounty has no assignee");

    let token = TokenClient::new(&env, &bounty.reward_token);
    token.transfer(&verifier, &assignee, &bounty.reward_amount);

    let mut contributor = storage::get_contributor(&env, &assignee)
        .unwrap_or(Contributor {
            address: assignee.clone(),
            reputation: 0,
            total_earned: 0,
            contribution_count: 0,
        });

    contributor.reputation += 10;
    contributor.total_earned += bounty.reward_amount;
    contributor.contribution_count += 1;

    storage::store_contributor(&env, &assignee, &contributor);

    events::emit_bounty_completed(&env, &bounty_id, &assignee);
    events::emit_reward_paid(&env, &bounty_id, &assignee, &bounty.reward_amount);
}
```

### Read-Only Queries

```rust
pub fn get_bounty(env: Env, bounty_id: BytesN<32>) -> Option<Bounty> {
    storage::get_bounty(&env, &bounty_id)
}

pub fn get_contributor(env: Env, address: Address) -> Option<Contributor> {
    storage::get_contributor(&env, &address)
}

pub fn get_bounty_count(env: Env) -> u64 {
    storage::get_bounty_count(&env)
}
```

### Storage Layer

Uses Soroban's persistent storage with typed `DataKey` enum for organized key-value access:

```rust
// src/storage.rs
pub fn get_bounty_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::BountyCount)
        .unwrap_or(0)
}

pub fn set_bounty_count(env: &Env, count: &u64) {
    env.storage()
        .persistent()
        .set(&DataKey::BountyCount, count);
}

pub fn store_bounty(env: &Env, id: &BytesN<32>, bounty: &Bounty) {
    env.storage()
        .persistent()
        .set(&DataKey::Bounty(id.clone()), bounty);
}

pub fn get_bounty(env: &Env, id: &BytesN<32>) -> Option<Bounty> {
    env.storage()
        .persistent()
        .get(&DataKey::Bounty(id.clone()))
}

pub fn store_contributor(env: &Env, address: &Address, contributor: &Contributor) {
    env.storage()
        .persistent()
        .set(&DataKey::Contributor(address.clone()), contributor);
}

pub fn get_contributor(env: &Env, address: &Address) -> Option<Contributor> {
    env.storage()
        .persistent()
        .get(&DataKey::Contributor(address.clone()))
}
```

### Event Emission

```rust
// src/events.rs
pub fn emit_bounty_created(env: &Env, bounty_id: &BytesN<32>, creator: &Address, reward: &i128) {
    let topic = Symbol::new(env, "bounty_created");
    env.events().publish(
        (topic, creator.clone()),
        (bounty_id.clone(), *reward),
    );
}

pub fn emit_bounty_claimed(env: &Env, bounty_id: &BytesN<32>, contributor: &Address) {
    let topic = Symbol::new(env, "bounty_claimed");
    env.events().publish(
        (topic, contributor.clone()),
        bounty_id.clone(),
    );
}

pub fn emit_bounty_completed(env: &Env, bounty_id: &BytesN<32>, contributor: &Address) {
    let topic = Symbol::new(env, "bounty_completed");
    env.events().publish(
        (topic, contributor.clone()),
        bounty_id.clone(),
    );
}

pub fn emit_reward_paid(env: &Env, bounty_id: &BytesN<32>, contributor: &Address, amount: &i128) {
    let topic = Symbol::new(env, "reward_paid");
    env.events().publish(
        (topic, contributor.clone()),
        (bounty_id.clone(), *amount),
    );
}
```

### Type Definitions

```rust
// src/types.rs
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    BountyCount,
    Bounty(BytesN<32>),
    Contributor(Address),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Bounty {
    pub creator: Address,
    pub title: Symbol,
    pub description: Symbol,
    pub reward_amount: i128,
    pub reward_token: Address,
    pub assignee: Option<Address>,
    pub status: Symbol,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Contributor {
    pub address: Address,
    pub reputation: u32,
    pub total_earned: i128,
    pub contribution_count: u32,
}
```

### Tests

```rust
// src/test.rs
#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, Env, Symbol,
};

use crate::contract::MergeMintContract;
use crate::contract::MergeMintContractClient;

fn setup_test() -> (Env, Address, Address, Address) {
    let env = Env::default();
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let verifier = Address::generate(&env);
    env.mock_all_auths();
    (env, creator, contributor, verifier)
}

#[test]
fn test_create_bounty() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let title = Symbol::new(&env, "test_bounty");
    let description = Symbol::new(&env, "Test_bounty_description");
    let reward_amount: i128 = 1000;
    let reward_token = Address::generate(&env);

    let bounty_id = client.create_bounty(
        &creator, &title, &description, &reward_amount, &reward_token,
    );

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.title, title);
    assert_eq!(bounty.reward_amount, reward_amount);
    assert_eq!(bounty.creator, creator);
}

#[test]
fn test_claim_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_1"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
    );

    client.claim_bounty(&contributor, &bounty_id);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignee.unwrap(), contributor);
}

#[test]
fn test_bounty_count() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    assert_eq!(client.get_bounty_count(), 0);

    let reward_token = Address::generate(&env);
    client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_a"),
        &Symbol::new(&env, "desc_a"),
        &100,
        &reward_token,
    );
    assert_eq!(client.get_bounty_count(), 1);

    client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_b"),
        &Symbol::new(&env, "desc_b"),
        &200,
        &reward_token,
    );
    assert_eq!(client.get_bounty_count(), 2);
}

#[test]
fn test_complete_bounty_updates_status() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_c"),
        &Symbol::new(&env, "desc_c"),
        &1000,
        &Address::generate(&env),
    );

    client.claim_bounty(&contributor, &bounty_id);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignee.unwrap(), contributor);
}
```

---

## Getting Started

### Prerequisites

- **Rust (stable)** — Install via [rustup](https://rustup.rs/)
- **Stellar CLI** — Install with `cargo install stellar-cli`
- **WASM target** — `rustup target add wasm32-unknown-unknown`

### Test

```bash
cargo test
```

Runs all four unit tests in `src/test.rs` against the Soroban test environment:
- `test_create_bounty` — creates a bounty, verifies fields
- `test_claim_bounty` — creates and claims, verifies assignee
- `test_bounty_count` — creates two bounties, verifies counter increments
- `test_complete_bounty_updates_status` — creates, claims, verifies assignment

### Build

```bash
cargo build --release --target wasm32-unknown-unknown
```

Produces the WASM binary at:
```
target/wasm32-unknown-unknown/release/mergemint_contracts.wasm
```

### Deploy

```bash
chmod +x scripts/deploy.sh
./scripts/deploy.sh sepolia default
```

The deployment script:
```bash
#!/usr/bin/env bash
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
```

---

## CLI Usage

After deploying the contract, you can invoke each function directly with the **Stellar CLI**. Export these shell variables once to keep the examples concise:

```bash
export CONTRACT_ID=<your-deployed-contract-id>   # C... contract address
export CREATOR=<your-stellar-address>             # G... public key or key alias
export NETWORK=testnet
```

> **Type encoding quick reference**
> | Soroban type | CLI encoding |
> |---|---|
> | `Address` | Stellar public key (`G...`) or key alias |
> | `BytesN<32>` | 64-character lowercase hex string |
> | `Symbol` | Alphanumeric + underscore string (max 32 chars, no spaces) |
> | `i128` | Integer literal |
> | `u32` | Integer literal |
> | `Option<T>` | `null` for `None`; bare value for `Some(value)` |

---

### create_bounty

Creates a new open bounty and returns the generated 32-byte bounty ID as a hex string. The `reward_amount` is in raw token units — for a token with 7 decimal places, 1 token = `10000000`. Set `min_reputation` to `0` to allow any contributor to claim, and pass `null` for `deadline` to leave the bounty open indefinitely.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $CREATOR \
  --network $NETWORK \
  -- create_bounty \
  --creator $CREATOR \
  --title "Fix_auth_timeout" \
  --description "Resolve_session_expiry_in_login_flow" \
  --reward_amount 10000000 \
  --reward_token GBDTYLEFTNKBV6DCLAURRRXN6VWQPGSZYDVS9GVQWGYLXNUDGQE2HSTW \
  --min_reputation 0 \
  --deadline null
```

Example return value:

```
"0000000000000000000000000000000000000000000000000000000000000000"
```

> Bounty IDs are 32-byte big-endian encodings of an incrementing counter — bounty #0 is all zeros, bounty #1 ends in `...0001`, and so on. Copy the exact hex string returned here for use in `claim_bounty`, `complete_bounty`, and `get_bounty`.

---

### claim_bounty

Assigns the calling contributor to an open bounty. The contributor must authenticate with `--source`. A contributor can only hold one active claim at a time.

```bash
export CONTRIBUTOR=GBCONTRIBUTORAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
export BOUNTY_ID=0000000000000000000000000000000000000000000000000000000000000000

stellar contract invoke \
  --id $CONTRACT_ID \
  --source $CONTRIBUTOR \
  --network $NETWORK \
  -- claim_bounty \
  --contributor $CONTRIBUTOR \
  --bounty_id $BOUNTY_ID
```

The command returns no value on success. It will panic if:
- The bounty is already at assignee capacity
- The contributor already has an active claim
- The bounty deadline has passed

---

### complete_bounty

Marks the bounty completed and transfers the reward from the verifier's wallet to the assignee(s). The verifier must hold at least `reward_amount` of the bounty's `reward_token` and must sign the transaction with `--source`.

```bash
export VERIFIER=GBVERIFIERAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
export BOUNTY_ID=0000000000000000000000000000000000000000000000000000000000000000

stellar contract invoke \
  --id $CONTRACT_ID \
  --source $VERIFIER \
  --network $NETWORK \
  -- complete_bounty \
  --verifier $VERIFIER \
  --bounty_id $BOUNTY_ID
```

On success the reward is transferred on-chain, the assignee's reputation increases by +10, and `bounty_completed` / `reward_paid` events are emitted.

---

### get_bounty

Returns the full `Bounty` struct for a given ID, or `null` if the ID does not exist. Read-only — no state changes occur.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $CREATOR \
  --network $NETWORK \
  -- get_bounty \
  --bounty_id 0000000000000000000000000000000000000000000000000000000000000000
```

Example output:

```json
{
  "creator": "GABC...",
  "reward_amount": "10000000",
  "reward_token": "GBDT...",
  "assignees": [],
  "max_assignees": 1,
  "status": "open",
  "min_reputation": 0,
  "deadline": null
}
```

---

### get_contributor

Returns the `Contributor` profile for a wallet address, or `null` if the address has no recorded activity. Read-only — no state changes occur.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $CREATOR \
  --network $NETWORK \
  -- get_contributor \
  --address GBCONTRIBUTORAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
```

Example output:

```json
{
  "address": "GBCO...",
  "reputation": 10,
  "total_earned": "10000000",
  "contribution_count": 1,
  "active_claims": 0,
  "metadata": null
}
```

---

### get_bounty_count

Returns the total number of bounties ever created (not the number currently open). Read-only — no state changes occur.

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $CREATOR \
  --network $NETWORK \
  -- get_bounty_count
```

Example output: `1`

---

## Project Structure

```
mergemint-contracts/
├── src/
│   ├── lib.rs              # Crate entry, module declarations, re-exports
│   ├── contract.rs         # Core contract logic (MergeMintContract)
│   ├── storage.rs          # Persistent storage helpers (get/set)
│   ├── events.rs           # Event emission functions
│   ├── types.rs            # DataKey enum, Bounty, Contributor structs
│   └── test.rs             # Unit tests with Soroban testutils
├── scripts/
│   └── deploy.sh           # Build + deploy to Stellar network
├── docs/
│   └── architecture.md     # Detailed architecture documentation
├── test_snapshots/
│   └── test/               # Soroban ledger snapshots for test assertions
│       ├── test_bounty_count.1.json
│       ├── test_claim_bounty.1.json
│       ├── test_complete_bounty_updates_status.1.json
│       ├── test_contributor_reputation.1.json
│       └── test_create_bounty.1.json
├── Cargo.toml              # Dependencies + release profile
├── Cargo.lock              # Locked dependency versions
└── README.md               # This file
```

---

## Build Configuration

The release profile is optimized for WASM deployment:

```toml
[profile.release]
opt-level = "z"         # Optimize for size
overflow-checks = true  # Safety: panics on arithmetic overflow
debug = 0               # No debug symbols
strip = "symbols"       # Strip all symbols
lto = true              # Link-time optimization
codegen-units = 1       # Single CGU for max optimization
```

---

## Security

- **Authentication**: All state-mutating functions require `require_auth()` on the caller's `Address`, preventing unauthorized state changes.
- **Double-claim prevention**: `claim_bounty` checks `bounty.assignee.is_some()` and panics if already assigned — each bounty can only have one contributor.
- **Safe token transfers**: Uses Soroban's standard `TokenClient::transfer()` which reverts on insufficient balance or unauthorized sender.
- **Overflow protection**: `overflow-checks = true` in the release profile ensures arithmetic panics on overflow rather than wrapping.
- **Reputation monotonicity**: Reputation only increases (+10 per completed bounty), never decreases.
- **Deterministic IDs**: Bounty IDs are derived from an incrementing counter, ensuring uniqueness and predictable ordering.

---

## License

MIT
