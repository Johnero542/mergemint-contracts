use crate::state::{Bounty, BountyId, BOUNTIES};
use soroban_sdk::{Env, Vec};

pub fn get_bounty(env: &Env, id: BountyId) -> Option<Bounty> {
    BOUNTIES.get(env, &id)
}

pub fn get_bounties(env: &Env, ids: Vec<BountyId>) -> Vec<Option<Bounty>> {
    let mut results = Vec::new(env);
    for id in ids.iter() {
        results.push_back(BOUNTIES.get(env, &id));
    }
    results
}
