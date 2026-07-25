// SPDX-License-Identifier: MIT
use soroban_sdk::{contracttype, Address, BytesN, String, Symbol, Vec};

/// A type-safe identifier for a bounty.
///
/// Wraps a raw `BytesN<32>` to prevent accidental substitution with other
/// 32-byte values (hashes, keys, nonces). Serialises identically to the
/// inner `BytesN<32>` when used with `#[contracttype]`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BountyId(pub BytesN<32>);

impl From<BytesN<32>> for BountyId {
    fn from(value: BytesN<32>) -> Self {
        BountyId(value)
    }
}

impl From<BountyId> for BytesN<32> {
    fn from(value: BountyId) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    BountyCount,
    Bounty(BountyId),
    BountyMeta(BountyId),
    Contributor(Address),
    ContributorBounties(Address),
    StatusIndex(Symbol),
    OpenBounties,
    Approvals(BountyId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Milestone {
    pub description: Symbol,
    pub reward: i128,
    pub completed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Bounty {
    pub creator: Address,
    pub reward_amount: i128,
    pub reward_token: Address,
    pub assignees: Vec<(Address, u32)>,
    pub max_assignees: u32,
    #[allow(dead_code)]
    pub status: Symbol,
    pub min_reputation: u32,
    pub deadline: Option<u32>,
    /// Optional list of addresses permitted to approve completion.
    /// When set, approve_completion enforces that only listed addresses may vote.
    /// When None, the single-verifier complete_bounty flow applies unchanged.
    pub required_verifiers: Option<Vec<Address>>,
    /// Number of approvals required before completion executes automatically.
    /// Only meaningful when required_verifiers is Some. A value of 0 is treated as 1.
    pub approval_threshold: u32,
    /// Categorisation tags for the bounty (e.g. "bug", "docs", "feature").
    /// At most 5 tags are allowed; `create_bounty` panics with `TooManyTags`
    /// if the caller supplies more than 5.
    pub tags: Vec<Symbol>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BountyMeta {
    pub title: Symbol,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Contributor {
    #[allow(dead_code)]
    pub address: Address,
    pub reputation: u32,
    pub total_earned: i128,
    pub contribution_count: u32,
    pub active_claims: u32,
    pub metadata: Option<Symbol>,
}

impl Contributor {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            reputation: 0,
            total_earned: 0,
            contribution_count: 0,
            active_claims: 0,
            metadata: None,
        }
    }
}
