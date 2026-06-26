use soroban_sdk::{Address, BytesN, Env};

use crate::types::{Bounty, BountyMeta, Contributor, DataKey};

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

pub fn store_bounty_meta(env: &Env, id: &BytesN<32>, meta: &BountyMeta) {
    env.storage()
        .temporary()
        .set(&DataKey::BountyMeta(id.clone()), meta);
}

pub fn get_bounty_meta(env: &Env, id: &BytesN<32>) -> Option<BountyMeta> {
    env.storage()
        .temporary()
        .get(&DataKey::BountyMeta(id.clone()))
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
