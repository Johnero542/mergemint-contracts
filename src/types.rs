// SPDX-License-Identifier: MIT
use soroban_sdk::{contracttype, Address, BytesN, Symbol};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    BountyCount,
    Bounty(BytesN<32>),
    BountyMeta(BytesN<32>),
    Contributor(Address),
    StatusIndex(Symbol),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Bounty {
    pub creator: Address,
    pub reward_amount: i128,
    pub reward_token: Address,
    pub assignee: Option<Address>,
    // Exposed in the contract type for off-chain indexers to track lifecycle state;
    // not read internally since contract logic uses assignee presence as the source of truth.
    #[allow(dead_code)]
    pub status: Symbol,
    pub min_reputation: u32,
    /// Optional ledger sequence number after which the bounty cannot be claimed.
    /// If set, claim_bounty will reject claims from contributors once the ledger
    /// sequence number exceeds this value.
    pub deadline: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BountyMeta {
    pub title: Symbol,
    pub description: Symbol,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Contributor {
    // Stored so off-chain consumers can identify the contributor from the struct alone,
    // without needing to key back through storage.
    #[allow(dead_code)]
    pub address: Address,
    pub reputation: u32,
    pub total_earned: i128,
    pub contribution_count: u32,
    pub active_claims: u32,
}
