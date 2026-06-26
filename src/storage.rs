use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

use crate::types::{Bounty, Contributor, DataKey};

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

pub fn get_status_index(env: &Env, status: &Symbol) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::StatusIndex(status.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_status_index(env: &Env, status: &Symbol, ids: &Vec<BytesN<32>>) {
    env.storage()
        .persistent()
        .set(&DataKey::StatusIndex(status.clone()), ids);
}

pub fn add_to_status_index(env: &Env, status: &Symbol, id: &BytesN<32>) {
    let mut ids = get_status_index(env, status);
    ids.push_back(id.clone());
    set_status_index(env, status, &ids);
}

pub fn remove_from_status_index(env: &Env, status: &Symbol, id: &BytesN<32>) {
    let ids = get_status_index(env, status);
    let mut new_ids = Vec::new(env);
    for existing in ids.iter() {
        if existing != id.clone() {
            new_ids.push_back(existing);
        }
    }
    set_status_index(env, status, &new_ids);
}
