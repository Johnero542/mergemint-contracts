// SPDX-License-Identifier: MIT
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

use crate::types::{Bounty, BountyMeta, Contributor, DataKey};

/// Approximate 1 year in ledger sequences at 5 seconds per ledger.
const STORAGE_TTL_LEDGERS: u32 = 6_307_200;
/// Extend TTL when remaining life falls below half a year.
const STORAGE_TTL_THRESHOLD: u32 = STORAGE_TTL_LEDGERS / 2;

/// Returns the total number of bounties ever created.
///
/// Storage key: `DataKey::BountyCount`
/// Stored type: `u64`
/// Default value: `0` (returns 0 if no bounties have been created)
///
/// This is a singleton counter that monotonically increases with each
/// `create_bounty` call. The value is used as part of the bounty ID
/// generation to produce deterministic, unique identifiers.
pub fn get_bounty_count(env: &Env) -> u64 {
    let key = DataKey::BountyCount;
    let count: Option<u64> = env.storage().persistent().get(&key);
    if count.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    count.unwrap_or(0)
}

pub fn set_bounty_count(env: &Env, count: &u64) {
    let key = DataKey::BountyCount;
    env.storage().persistent().set(&key, count);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

pub fn store_bounty(env: &Env, id: &BytesN<32>, bounty: &Bounty) {
    let key = DataKey::Bounty(id.clone());
    env.storage().persistent().set(&key, bounty);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

pub fn get_bounty(env: &Env, id: &BytesN<32>) -> Option<Bounty> {
    let key = DataKey::Bounty(id.clone());
    let bounty: Option<Bounty> = env.storage().persistent().get(&key);
    if bounty.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    bounty
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
    let key = DataKey::Contributor(address.clone());
    env.storage().persistent().set(&key, contributor);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

pub fn get_contributor(env: &Env, address: &Address) -> Option<Contributor> {
    let key = DataKey::Contributor(address.clone());
    let contributor: Option<Contributor> = env.storage().persistent().get(&key);
    if contributor.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    contributor
}

// #269: contributor bounty index
pub fn get_contributor_bounties(env: &Env, address: &Address) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::ContributorBounties(address.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn append_contributor_bounty(env: &Env, address: &Address, bounty_id: &BytesN<32>) {
    let mut list = get_contributor_bounties(env, address);
    list.push_back(bounty_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::ContributorBounties(address.clone()), &list);
}

pub fn get_bounties_by_status(env: &Env, status: &Symbol) -> Vec<BytesN<32>> {
    let key = DataKey::StatusIndex(status.clone());
    let result: Option<Vec<BytesN<32>>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    result.unwrap_or_else(|| Vec::new(env))
}

pub fn set_bounties_by_status(env: &Env, status: &Symbol, bounties: &Vec<BytesN<32>>) {
    let key = DataKey::StatusIndex(status.clone());
    env.storage().persistent().set(&key, bounties);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

pub fn add_bounty_to_status(env: &Env, bounty_id: &BytesN<32>, status: &Symbol) {
    let mut current = get_bounties_by_status(env, status);
    if current.iter().all(|id| id != *bounty_id) {
        current.push_back(bounty_id.clone());
    }
    set_bounties_by_status(env, status, &current);
}

pub fn remove_bounty_from_status(env: &Env, bounty_id: &BytesN<32>, status: &Symbol) {
    let current = get_bounties_by_status(env, status);
    let mut updated = Vec::new(env);
    for id in current.iter() {
        if id != *bounty_id {
            updated.push_back(id);
        }
    }
    set_bounties_by_status(env, status, &updated);
}

pub fn move_bounty_status(
    env: &Env,
    bounty_id: &BytesN<32>,
    old_status: &Symbol,
    new_status: &Symbol,
) {
    if old_status != new_status {
        remove_bounty_from_status(env, bounty_id, old_status);
        add_bounty_to_status(env, bounty_id, new_status);
    }
}

pub fn get_open_bounties(env: &Env) -> Vec<BytesN<32>> {
    let key = DataKey::OpenBounties;
    let result: Option<Vec<BytesN<32>>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    result.unwrap_or_else(|| Vec::new(env))
}

pub fn set_open_bounties(env: &Env, bounties: &Vec<BytesN<32>>) {
    let key = DataKey::OpenBounties;
    env.storage().persistent().set(&key, bounties);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}
