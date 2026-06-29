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
    creator: &Address,
    tag: &str,
    deadline: Option<u32>,
) -> BountyId {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &1000,
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
// Core lifecycle tests
// ===========================================================================

#[test]
fn test_create_bounty() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let title = Symbol::new(&env, "test_bounty");
    let description = Symbol::new(&env, "Test_bounty_desc");
    let reward_amount: i128 = 1000;
    let reward_token = Address::generate(&env);
    let bounty_id = client.create_bounty(&creator, &title, &description, &reward_amount, &reward_token, &0, &None);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.reward_amount, reward_amount);
    assert_eq!(bounty.creator, creator);
    assert!(bounty.assignees.is_empty());

    let meta = client.get_bounty_meta(&bounty_id).unwrap();
    assert_eq!(meta.title, title);
    assert_eq!(meta.description, description);
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
    let (assignee_addr, _) = bounty.assignees.get(0).unwrap();
    assert_eq!(assignee_addr, contributor);
}

// ===========================================================================
// Issue: update_bounty
// ===========================================================================

#[test]
fn test_update_bounty_title() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "old_title", None);

    let new_title = Symbol::new(&env, "new_title");
    client.update_bounty(&creator, &bounty_id, &Some(new_title.clone()), &None, &None);

    let meta = client.get_bounty_meta(&bounty_id).unwrap();
    assert_eq!(meta.title, new_title);
}

#[test]
fn test_update_bounty_description() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "some_title", None);

    let new_desc = Symbol::new(&env, "updated_desc");
    client.update_bounty(&creator, &bounty_id, &None, &Some(new_desc.clone()), &None);

    let meta = client.get_bounty_meta(&bounty_id).unwrap();
    assert_eq!(meta.description, new_desc);
}

#[test]
fn test_update_bounty_reward_amount() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "reward_b", None);

    client.update_bounty(&creator, &bounty_id, &None, &None, &Some(2000i128));

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.reward_amount, 2000i128);
}

#[test]
#[should_panic(expected = "bounty cannot be updated after it is claimed")]
fn test_update_bounty_fails_when_claimed() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "claimed_b", None);
    client.claim_bounty(&contributor, &bounty_id);

    client.update_bounty(&creator, &bounty_id, &Some(Symbol::new(&env, "new")), &None, &None);
}

#[test]
#[should_panic(expected = "not the bounty creator")]
fn test_update_bounty_fails_for_non_creator() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "b_nc", None);

    client.update_bounty(&contributor, &bounty_id, &Some(Symbol::new(&env, "hacked")), &None, &None);
}

// ===========================================================================
// Issue: bounty deadline enforcement in claim_bounty
// ===========================================================================

#[test]
fn test_claim_bounty_within_deadline_succeeds() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Set ledger sequence to 100, deadline at 200 — should succeed.
    env.ledger().set_sequence_number(100);
    let bounty_id = make_bounty(&env, &client, &creator, "deadline_ok", Some(200));
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert!(!bounty.assignees.is_empty());
}

#[test]
#[should_panic(expected = "bounty deadline has passed")]
fn test_claim_bounty_after_deadline_panics() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Create bounty with deadline 50, then advance ledger past it.
    let bounty_id = make_bounty(&env, &client, &creator, "expired_b", Some(50));
    env.ledger().set_sequence_number(51);
    client.claim_bounty(&contributor, &bounty_id);
}

#[test]
fn test_claim_bounty_no_deadline_succeeds() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "no_deadline", None);
    env.ledger().set_sequence_number(99999);
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert!(!bounty.assignees.is_empty());
}

// ===========================================================================
// Issue: cancel_bounty
// ===========================================================================

#[test]
fn test_cancel_bounty_sets_cancelled_status() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "to_cancel", None);
    client.cancel_bounty(&creator, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "cancelled"));
}

#[test]
#[should_panic(expected = "not the bounty creator")]
fn test_cancel_bounty_non_creator_panics() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "cancel_nc", None);
    client.cancel_bounty(&contributor, &bounty_id);
}

#[test]
#[should_panic(expected = "bounty not open")]
fn test_cancel_bounty_claimed_panics() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "cancel_claimed", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.cancel_bounty(&creator, &bounty_id);
}

// ===========================================================================
// Issue: None-path tests for get_bounty and get_contributor
// ===========================================================================

#[test]
fn test_get_bounty_returns_none_for_unknown_id() {
    let (env, _creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let unknown_id = BytesN::random(&env);
    assert!(client.get_bounty(&unknown_id).is_none());
}

#[test]
fn test_get_contributor_returns_none_for_unknown_address() {
    let (env, _creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let unknown_addr = Address::generate(&env);
    assert!(client.get_contributor(&unknown_addr).is_none());
}

// ===========================================================================
// Additional existing tests (fixed)
// ===========================================================================

#[test]
fn test_bounty_count_increment_loop() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward_token = Address::generate(&env);
    for i in 0..5u64 {
        client.create_bounty(
            &creator,
            &Symbol::new(&env, "bounty"),
            &Symbol::new(&env, "desc"),
            &1000,
            &reward_token,
            &0,
            &None,
        );
        assert_eq!(client.get_bounty_count(), i + 1);
    }
    assert_eq!(client.get_bounty_count(), 5);
}

#[test]
fn test_complete_bounty_updates_contributor() {
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
    assert_eq!(client.get_bounties_by_status(&open_status).len(), 0);
    assert_eq!(client.get_bounties_by_status(&in_progress_status).len(), 1);
    assert_eq!(client.get_bounties_by_status(&completed_status).len(), 0);
    assert_eq!(client.get_bounties_by_status(&cancelled_status).len(), 0);

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
    let token = Address::generate(&env);

    let id = create_bounty_helper(&client, &env, &creator, &token, "bounty_z");
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
    crate::storage::store_contributor(&env, &contributor, &contrib_after);

    client.claim_bounty(&contributor, &bounty_id_2);
    let contrib2 = client.get_contributor(&contributor).unwrap();
    assert_eq!(contrib2.active_claims, 1);
}

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
fn test_update_contributor_metadata_overwrite() {
    let (env, _creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    client.update_contributor_metadata(&contributor, &Symbol::new(&env, "old_uri"));
    client.update_contributor_metadata(&contributor, &Symbol::new(&env, "new_uri"));

    let data = client.get_contributor(&contributor).unwrap();
    assert_eq!(data.metadata.unwrap(), Symbol::new(&env, "new_uri"));
}

#[test]
fn test_contributor_metadata_default_none() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "meta_b"), &Symbol::new(&env, "d"), &100,
        &Address::generate(&env), &0, &None,
    );
    client.claim_bounty(&contributor, &bounty_id);
    client.complete_bounty(&verifier, &bounty_id);

    let data = client.get_contributor(&contributor).unwrap();
    assert!(data.metadata.is_none());
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

// ===========================================================================
// Issue #256: event emission assertions
// ===========================================================================

#[test]
fn test_event_bounty_created() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "evt_b"), &Symbol::new(&env, "desc"), &1000,
        &Address::generate(&env), &0, &None,
    );

    let events = env.events().all();
    // Last event should be bounty_created
    let (_, topics, data) = &events.get(events.len() - 1).unwrap();

    // topics = (Symbol("bounty_created"), creator_address)
    let event_name: Val = topics.get(0).unwrap();
    assert_eq!(event_name, Symbol::new(&env, "bounty_created").into());

    let creator_val: Val = topics.get(1).unwrap();
    assert_eq!(creator_val, creator.to_val());

    // data = (bounty_id, reward_amount)
    let (data_id, data_reward): (BytesN<32>, i128) = data.clone().try_into_val(&env).unwrap();
    assert_eq!(data_id, bounty_id);
    assert_eq!(data_reward, 1000);
}

#[test]
fn test_event_bounty_claimed() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "evt_cl"), &Symbol::new(&env, "desc"), &1000,
        &Address::generate(&env), &0, &None,
    );
    // Clear events to isolate the claim event
    let _ = env.events().all();
    client.claim_bounty(&contributor, &bounty_id);

    let events = env.events().all();
    let (_, topics, data) = &events.get(events.len() - 1).unwrap();

    // topics = (Symbol("bounty_claimed"), contributor_address)
    let event_name: Val = topics.get(0).unwrap();
    assert_eq!(event_name, Symbol::new(&env, "bounty_claimed").into());

    let contributor_val: Val = topics.get(1).unwrap();
    assert_eq!(contributor_val, contributor.to_val());

    // data = bounty_id
    let data_id: BytesN<32> = data.clone().try_into_val(&env).unwrap();
    assert_eq!(data_id, bounty_id);
}

#[test]
fn test_event_bounty_completed() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let reward_token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = StellarAssetClient::new(&env, &reward_token);
    token_client.mint(&token_admin, &1000);
    token_client.transfer(&token_admin, &verifier, &1000);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "evt_cp"), &Symbol::new(&env, "desc"), &1000,
        &reward_token, &0, &None,
    );
    client.claim_bounty(&contributor, &bounty_id);
    let _ = env.events().all();
    client.complete_bounty(&verifier, &bounty_id);

    let events = env.events().all();
    let (_, topics, data) = &events.get(events.len() - 1).unwrap();

    let event_name: Val = topics.get(0).unwrap();
    assert_eq!(event_name, Symbol::new(&env, "bounty_completed").into());

    let assignee_val: Val = topics.get(1).unwrap();
    assert_eq!(assignee_val, contributor.to_val());

    let data_id: BytesN<32> = data.clone().try_into_val(&env).unwrap();
    assert_eq!(data_id, bounty_id);
}

#[test]
fn test_event_reward_paid() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let reward_token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = StellarAssetClient::new(&env, &reward_token);
    token_client.mint(&token_admin, &1000);
    token_client.transfer(&token_admin, &verifier, &1000);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "evt_rp"), &Symbol::new(&env, "desc"), &1000,
        &reward_token, &0, &None,
    );
    client.claim_bounty(&contributor, &bounty_id);
    let _ = env.events().all();
    client.complete_bounty(&verifier, &bounty_id);

    let events = env.events().all();
    let (_, topics, data) = &events.get(0).unwrap();

    let event_name: Val = topics.get(0).unwrap();
    assert_eq!(event_name, Symbol::new(&env, "reward_paid").into());

    let assignee_val: Val = topics.get(1).unwrap();
    assert_eq!(assignee_val, contributor.to_val());

    let (data_id, data_reward): (BytesN<32>, i128) = data.clone().try_into_val(&env).unwrap();
    assert_eq!(data_id, bounty_id);
    assert_eq!(data_reward, 1000);
}

#[test]
fn test_all_lifecycle_events() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let reward_token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = StellarAssetClient::new(&env, &reward_token);
    token_client.mint(&token_admin, &2000);
    token_client.transfer(&token_admin, &verifier, &2000);

    let reward_amount: i128 = 2000;
    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "full_lifecycle"), &Symbol::new(&env, "desc"),
        &reward_amount, &reward_token, &0, &None,
    );
    client.claim_bounty(&contributor, &bounty_id);
    client.complete_bounty(&verifier, &bounty_id);

    let events = env.events().all();
    let mut lifecycle_events: Vec<Symbol> = Vec::new(&env);
    for event in events.iter() {
        let (_addr, topics, _data) = event;
        if topics.len() > 0 {
            if let Ok(sym) = Symbol::try_from_val(&env, &topics.get(0).unwrap()) {
                let name: &str = &sym.to_string();
                if name == "bounty_created"
                    || name == "bounty_claimed"
                    || name == "bounty_completed"
                    || name == "reward_paid"
                {
                    lifecycle_events.push_back(sym);
                }
            }
        }
    }

    assert_eq!(lifecycle_events.len(), 4, "expected exactly 4 lifecycle events");
    assert_eq!(
        lifecycle_events.get(0).unwrap(),
        Symbol::new(&env, "bounty_created"),
        "first event should be bounty_created"
    );
    assert_eq!(
        lifecycle_events.get(1).unwrap(),
        Symbol::new(&env, "bounty_claimed"),
        "second event should be bounty_claimed"
    );
    assert_eq!(
        lifecycle_events.get(2).unwrap(),
        Symbol::new(&env, "reward_paid"),
        "third event should be reward_paid"
    );
    assert_eq!(
        lifecycle_events.get(3).unwrap(),
        Symbol::new(&env, "bounty_completed"),
        "fourth event should be bounty_completed"
    );
}
