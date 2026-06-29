// SPDX-License-Identifier: MIT
#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Budget as _, Ledger as _},
    token::StellarAssetClient,
    Address, BytesN, Env, IntoVal, Symbol, TryFromVal, TryIntoVal, Val, Vec,
};

use crate::contract::MergeMintContract;
use crate::contract::MergeMintContractClient;
use crate::types::BountyId;

fn setup_test() -> (Env, Address, Address, Address) {
    let env = Env::default();
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let verifier = Address::generate(&env);
    env.mock_all_auths();
    (env, creator, contributor, verifier)
}

/// Create a bounty with the given tag, no min_reputation, and an optional deadline.
fn make_bounty(
    env: &Env,
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    tag: &str,
    deadline: Option<u32>,
) -> BountyId {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &reward,
        &Address::generate(env),
        &0,
        &deadline,
    )
}

/// Helper: create a bounty with a specific reward token, min_reputation=0, no deadline.
fn create_bounty_helper(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    token: &Address,
    tag: &str,
) -> BountyId {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &1000,
        token,
        &0,
        &None,
    )
}

/// Helper: create a bounty with default min_reputation=0 and no deadline.
fn create_test_bounty(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    reward: i128,
) -> BountyId {
    let reward_token = Address::generate(env);
    client.create_bounty(
        creator,
        &Symbol::new(env, "test_bounty"),
        &Symbol::new(env, "test_desc"),
        &reward,
        &reward_token,
        &0,
        &None,
    )
}

// ===========================================================================
// Core lifecycle
// ===========================================================================

#[test]
fn test_create_bounty() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward_amount: i128 = 1000;
    let reward_token = Address::generate(&env);
    let bounty_id = client.create_bounty(&creator, &title, &description, &reward_amount, &reward_token, &0, &None);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.reward_amount, reward_amount);
    assert_eq!(bounty.creator, creator);
    assert!(bounty.assignees.is_empty());
    assert!(bounty.tags.is_empty());

    let meta = client.get_bounty_meta(&bounty_id).unwrap();
    assert_eq!(meta.title, Symbol::new(&env, "test_b"));
}

#[test]
fn test_claim_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "bounty_1"),
        &Symbol::new(&env, "desc"), &1000, &Address::generate(&env), &0, &None
    );
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    let (assignee_addr, share) = bounty.assignees.get(0).unwrap();
    assert_eq!(assignee_addr, contributor);
    assert_eq!(share, 10_000u32);
}

#[test]
fn test_bounty_count() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    assert_eq!(client.get_bounty_count(), 0);
    let reward_token = Address::generate(&env);
    client.create_bounty(&creator, &Symbol::new(&env, "bounty_a"), &Symbol::new(&env, "desc_a"), &100, &reward_token, &0, &None);
    assert_eq!(client.get_bounty_count(), 1);
    client.create_bounty(&creator, &Symbol::new(&env, "bounty_b"), &Symbol::new(&env, "desc_b"), &200, &reward_token, &0, &None);
    assert_eq!(client.get_bounty_count(), 2);
}

#[test]
fn test_complete_bounty_updates_contributor() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "bounty_c", None);
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "completed"));
}

// ====================================================================fn test_complete_bounty_updates_contributor() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "bounty_c"),
        &Symbol::new(&env, "desc_c"), &1000, &Address::generate(&env), &0, &None
    );
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    let (assignee_addr, _) = bounty.assignees.get(0).unwrap();
    assert_eq!(assignee_addr, contributor);
}

// ====================================================================#[should_panic(expected = "bounty has no assignee")]
fn test_complete_bounty_no_assignee_panics() {
    let (env, creator, _contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "unclaimed"),
        &Symbol::new(&env, "no_assignee"), &1000, &Address::generate(&env), &0, &None
    );

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert!(bounty.assignees.is_empty());
    client.complete_bounty(&verifier, &bounty_id);
}

// ====================================================================#[test]
fn test_raise_dispute_creator() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_dispute"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
    );
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "disputed"));
}

#[test]
fn test_raise_dispute_assignee() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_dispute2"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
    );
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "disputed"));
}

#[test]
#[should_panic(expected = "only creator or assignee can raise dispute")]
fn test_raise_dispute_third_party_fails() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let third_party = Address::generate(&env);
    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_dispute3"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
    );
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&third_party, &bounty_id);
}

#[test]
#[should_panic(expected = "contributor already has an active claim")]
fn test_second_claim_rejected_while_active() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id_1 = make_bounty(&env, &client, &creator, "active1", None);
    let bounty_id_2 = make_bounty(&env, &client, &creator, "active2", None);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "status_bounty"),
        &Symbol::new(&env, "desc"),
        &1000,
        &reward_token,
        &0,
        &None,
    );

    let open_status = Symbol::new(&env, "open");
    let in_progress_status = Symbol::new(&env, "in_progress");
    let completed_status = Symbol::new(&env, "completed");
    let cancelled_status = Symbol::new(&env, "cancelled");

    assert_eq!(client.get_bounties_by_status(&open_status).len(), 1);
    assert_eq!(client.get_bounties_by_status(&in_progress_status).len(), 0);
    assert_eq!(client.get_bounties_by_status(&completed_status).len(), 0);
    assert_eq!(client.get_bounties_by_status(&cancelled_status).len(), 0);

    client.claim_bounty(&contributor, &bounty_id);

    client.complete_bounty(&verifier, &bounty_id);
    assert_eq!(client.get_bounties_by_status(&open_status).len(), 0);
    assert_eq!(client.get_bounties_by_status(&in_progress_status).len(), 0);
    assert_eq!(client.get_bounties_by_status(&completed_status).len(), 1);
    assert_eq!(client.get_bounties_by_status(&cancelled_status).len(), 0);

    let cancelled_bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "cancelled_bounty"),
        &Symbol::new(&env, "desc"),
        &500,
        &reward_token,
        &0,
        &None,
    );
    client.cancel_bounty(&creator, &cancelled_bounty_id);
    assert_eq!(client.get_bounties_by_status(&open_status).len(), 0);
    assert_eq!(client.get_bounties_by_status(&cancelled_status).len(), 1);
}

#[test]
#[should_panic(expected = "bounty already assigned")]
fn test_second_contributor_cannot_claim_full_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "full_c", None);
    client.claim_bounty(&contributor, &bounty_id);

    let contributor2 = Address::generate(&env);
    client.claim_bounty(&contributor2, &bounty_id);
}

#[test]
fn test_status_index_moves_on_cancel() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let id = make_bounty(&env, &client, &creator, "bounty_z");
    client.cancel_bounty(&creator, &id);

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    let cancelled_ids = client.get_bounties_by_status(&Symbol::new(&env, "cancelled"));
    assert_eq!(open_ids.len(), 0);
    assert_eq!(cancelled_ids.len(), 1);
    assert_eq!(cancelled_ids.get(0).unwrap(), id);
}

#[test]
fn test_active_claims_decremented_after_complete() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward_amount: i128 = 500;
    let reward_token = Address::generate(&env);

    let bounty_id_1 = client.create_bounty(
        &creator, &Symbol::new(&env, "b1"), &Symbol::new(&env, "d1"), &reward_amount, &reward_token, &0, &None,
    );
    let bounty_id_2 = client.create_bounty(
        &creator, &Symbol::new(&env, "b2"), &Symbol::new(&env, "d2"), &reward_amount, &reward_token, &0, &None,
    );

    client.claim_bounty(&contributor, &bounty_id_1);
    let contrib = client.get_contributor(&contributor).unwrap();
    assert_eq!(contrib.active_claims, 1);

    // Manually reset active_claims to simulate completion without token transfer
    let contrib_after = crate::types::Contributor {
        address: contributor.clone(),
        reputation: 10,
        total_earned: reward_amount,
        contribution_count: 1,
        active_claims: 0,
        metadata: None,
    };

    complete_bounties_for(&contrib_a, 3); // 30 rep
    complete_bounties_for(&contrib_b, 1); // 10 rep
    complete_bounties_for(&contrib_c, 2); // 20 rep

// ====================================================================fn test_update_contributor_metadata_stores_value() {
    let (env, _creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let uri = Symbol::new(&env, "ipfs_hash_123");
    client.update_contributor_metadata(&contributor, &uri);

    let data = client.get_contributor(&contributor).unwrap();
    assert_eq!(data.metadata.unwrap(), uri);
}

#[test]
fn test_get_top_contributors_empty() {
    let (env, _creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    client.update_contributor_metadata(&contributor, &Symbol::new(&env, "old_uri"));
    client.update_contributor_metadata(&contributor, &Symbol::new(&env, "new_uri"));

    let data = client.get_contributor(&contributor).unwrap();
    assert_eq!(data.metadata.unwrap(), Symbol::new(&env, "new_uri"));
}

// ===========================================================================
// Status index
// ===========================================================================

#[test]
fn test_status_index_open_on_create() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "meta_b"), &Symbol::new(&env, "d"), &100,
        &Address::generate(&env), &0, &None,
    );
    client.claim_bounty(&contributor, &bounty_id);
    client.complete_bounty(&verifier, &bounty_id);

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    assert_eq!(open_ids.len(), 1);
    assert_eq!(open_ids.get(0).unwrap(), id);
}

// ====================================================================fn test_single_assignee_gets_full_share() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "single"), &Symbol::new(&env, "desc"), &1000,
        &Address::generate(&env), &0, &None,
    );
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignees.len(), 1);
    let (addr, share) = bounty.assignees.get(0).unwrap();
    assert_eq!(addr, contributor);
    assert_eq!(share, 10_000u32);
}

#[test]
fn test_status_index_moves_on_cancel() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "full_b"), &Symbol::new(&env, "desc"), &1000,
        &Address::generate(&env), &0, &None,
    );
    // First claim should succeed (max_assignees defaults to 1)
    client.claim_bounty(&contributor, &bounty_id);

    // Verify the bounty is at capacity
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignees.len(), 1);
}

// ====================================================================}

// =============================================================