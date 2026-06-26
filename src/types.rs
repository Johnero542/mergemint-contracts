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
    pub status: Symbol,
    pub min_reputation: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Contributor {
    pub address: Address,
    pub reputation: u32,
    pub total_earned: i128,
    pub contribution_count: u32,
}
