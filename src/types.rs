use soroban_sdk::{contracttype, Address, BytesN, Symbol};

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
    // Exposed in the contract type for off-chain indexers to track lifecycle state;
    // not read internally since contract logic uses assignee presence as the source of truth.
    #[allow(dead_code)]
    pub status: Symbol,
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
}
