// SPDX-License-Identifier: MIT
use soroban_sdk::{Address, Env, Symbol, Vec};

use crate::types::{Bounty, BountyId, BountyMeta, Contributor, DataKey};

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

/// Persists a `Bounty` struct under its unique 32-byte identifier.
///
/// Storage key: `DataKey::Bounty(id)`
/// Stored type: `Bounty`
/// Side effects: Overwrites any existing entry for the same `id`.
///
/// This is called during `create_bounty` (initial store) and
/// `claim_bounty` (when the assignee and status are updated).
pub fn store_bounty(env: &Env, id: &BountyId, bounty: &Bounty) {
    let key = DataKey::Bounty(id.clone());
    env.storage().persistent().set(&key, bounty);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

/// Retrieves a `Bounty` by its 32-byte identifier, if it exists.
///
/// Storage key: `DataKey::Bounty(id)`
/// Stored type: `Bounty`
/// Returns: `Some(Bounty)` if found, `None` if no bounty exists for `id`.
///
/// Callers should handle the `None` case (e.g., by panicking with a
/// descriptive message as done in `claim_bounty` and `complete_bounty`).
pub fn get_bounty(env: &Env, id: &BountyId) -> Option<Bounty> {
    let key = DataKey::Bounty(id.clone());
    let bounty: Option<Bounty> = env.storage().persistent().get(&key);
    if bounty.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    bounty
}

pub fn store_bounty_meta(env: &Env, id: &BountyId, meta: &BountyMeta) {
    let key = DataKey::BountyMeta(id.clone());
    env.storage().persistent().set(&key, meta);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

pub fn get_bounty_meta(env: &Env, id: &BountyId) -> Option<BountyMeta> {
    let key = DataKey::BountyMeta(id.clone());
    let meta: Option<BountyMeta> = env.storage().persistent().get(&key);
    if meta.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    meta
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

pub fn get_bounties_by_status(env: &Env, status: &Symbol) -> Vec<BountyId> {
    let key = DataKey::StatusIndex(status.clone());
    let result: Option<Vec<BountyId>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    result.unwrap_or_else(|| Vec::new(env))
}

pub fn set_bounties_by_status(env: &Env, status: &Symbol, bounties: &Vec<BountyId>) {
    let key = DataKey::StatusIndex(status.clone());
    env.storage().persistent().set(&key, bounties);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

pub fn get_status_count(env: &Env, status: &Symbol) -> u32 {
    let key = DataKey::StatusCount(status.clone());
    let count: Option<u32> = env.storage().persistent().get(&key);
    if count.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    count.unwrap_or(0)
}

pub fn set_status_count(env: &Env, status: &Symbol, count: &u32) {
    let key = DataKey::StatusCount(status.clone());
    env.storage().persistent().set(&key, count);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

pub fn add_bounty_to_status(env: &Env, bounty_id: &BountyId, status: &Symbol) {
    let mut current = get_bounties_by_status(env, status);
    let is_new = current.iter().all(|id| id != *bounty_id);
    if is_new {
        current.push_back(bounty_id.clone());
        set_bounties_by_status(env, status, &current);
        increment_status_count(env, status);
    }
}

pub fn remove_bounty_from_status(env: &Env, bounty_id: &BountyId, status: &Symbol) {
    let current = get_bounties_by_status(env, status);
    let mut updated = Vec::new(env);
    let mut removed = false;
    for id in current.iter() {
        if id != *bounty_id {
            updated.push_back(id);
        } else {
            removed = true;
        }
    }
    if removed {
        decrement_status_count(env, status);
    }
    set_bounties_by_status(env, status, &updated);
}

pub fn move_bounty_status(
    env: &Env,
    bounty_id: &BountyId,
    old_status: &Symbol,
    new_status: &Symbol,
) {
    if old_status != new_status {
        remove_bounty_from_status(env, bounty_id, old_status);
        add_bounty_to_status(env, bounty_id, new_status);
    }
}

pub fn get_approvals(env: &Env, bounty_id: &BountyId) -> Vec<Address> {
    let key = DataKey::Approvals(bounty_id.clone());
    let result: Option<Vec<Address>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    result.unwrap_or_else(|| Vec::new(env))
}

pub fn set_approvals(env: &Env, bounty_id: &BountyId, approvals: &Vec<Address>) {
    let key = DataKey::Approvals(bounty_id.clone());
    env.storage().persistent().set(&key, approvals);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

pub fn get_open_bounties(env: &Env) -> Vec<BountyId> {
    let key = DataKey::OpenBounties;
    let result: Option<Vec<BountyId>> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
    result.unwrap_or_else(|| Vec::new(env))
}

pub fn set_open_bounties(env: &Env, bounties: &Vec<BountyId>) {
    let key = DataKey::OpenBounties;
    env.storage().persistent().set(&key, bounties);
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

// ── Status Count ─────────────────────────────────────────────────────────────

fn increment_status_count(env: &Env, status: &Symbol) {
    let count = get_status_count(env, status);
    let key = DataKey::StatusCount(status.clone());
    env.storage().persistent().set(&key, &(count + 1));
    env.storage()
        .persistent()
        .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
}

fn decrement_status_count(env: &Env, status: &Symbol) {
    let count = get_status_count(env, status);
    if count > 0 {
        let key = DataKey::StatusCount(status.clone());
        env.storage().persistent().set(&key, &(count - 1));
        env.storage()
            .persistent()
            .extend_ttl(&key, STORAGE_TTL_THRESHOLD, STORAGE_TTL_LEDGERS);
    }
}

// ── Issue #3: Creator bounties index ─────────────────────────────────────────

pub fn get_creator_bounties(env: &Env, creator: &Address) -> Vec<BountyId> {
    env.storage()
        .persistent()
        .get(&DataKey::ContributorBounties(creator.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn append_creator_bounty(env: &Env, creator: &Address, bounty_id: &BountyId) {
    let mut list = get_creator_bounties(env, creator);
    list.push_back(bounty_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::ContributorBounties(creator.clone()), &list);
}
