#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
    Address, BytesN, Env, Symbol,
};

use crate::contract::MergeMintContract;
use crate::contract::MergeMintContractClient;

fn setup_test() -> (Env, Address, Address, Address) {
    let env = Env::default();
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let verifier = Address::generate(&env);

    env.mock_all_auths();

    (env, creator, contributor, verifier)
}

/// Convenience: create a bounty with an optional deadline.
fn make_bounty(
    env: &Env,
    client: &MergeMintContractClient,
    creator: &Address,
    tag: &str,
    deadline: Option<u32>,
) -> BytesN<32> {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &1000,
        &Address::generate(env),
        &deadline,
    )
}

fn create_test_bounty(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    reward: i128,
) -> soroban_sdk::BytesN<32> {
    let title = Symbol::new(env, "test_b");
    let description = Symbol::new(env, "test_desc");
    let reward_token = Address::generate(env);
    client.create_bounty(creator, &title, &description, &reward, &reward_token, &None)
}

// ===========================================================================
// Original tests (preserved)
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
    let bounty_id = client.create_bounty(&creator, &title, &description, &reward_amount, &reward_token, &None);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.title, title);
    assert_eq!(bounty.reward_amount, reward_amount);
    assert_eq!(bounty.creator, creator);
}

#[test]
fn test_claim_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "bounty_1"),
        &Symbol::new(&env, "desc"), &1000, &Address::generate(&env), &None
    );
    client.claim_bounty(&contributor, &bounty_id);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignee.unwrap(), contributor);
}

#[test]
fn test_bounty_count() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    assert_eq!(client.get_bounty_count(), 0);
    let reward_token = Address::generate(&env);
    client.create_bounty(&creator, &Symbol::new(&env, "bounty_a"), &Symbol::new(&env, "desc_a"), &100, &reward_token, &None);
    assert_eq!(client.get_bounty_count(), 1);
    client.create_bounty(&creator, &Symbol::new(&env, "bounty_b"), &Symbol::new(&env, "desc_b"), &200, &reward_token, &None);
    assert_eq!(client.get_bounty_count(), 2);
}

#[test]
fn test_bounty_count_increment_loop() {
    let (env, creator, _contributor) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    assert_eq!(client.get_bounty_count(), 0);

    let reward_token = Address::generate(&env);
    for i in 0..5 {
        client.create_bounty(
            &creator,
            &Symbol::new(&env, "bounty"),
            &Symbol::new(&env, "desc"),
            &1000,
            &reward_token,
            &0,
        );
        assert_eq!(client.get_bounty_count(), i + 1);
    }

    assert_eq!(client.get_bounty_count(), 5);
}

#[test]
fn test_complete_bounty_updates_status() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "bounty_c"),
        &Symbol::new(&env, "desc_c"), &1000, &Address::generate(&env), &None
    );
    client.claim_bounty(&contributor, &bounty_id);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignee.unwrap(), contributor);
}

// ===========================================================================
// Issue #346: get_bounty_count increments correctly in a loop
// ===========================================================================

#[test]
fn test_bounty_count_increments_in_loop() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    for i in 0..10u64 {
        let _ = create_test_bounty(&client, &env, &creator, 100);
        assert_eq!(
            client.get_bounty_count(), i + 1,
            "Bounty count should be {} after creating bounty #{}", i + 1, i + 1
        );
    }
}

// ===========================================================================
// Issue #259: bounty ID uniqueness across sequential creates
// ===========================================================================

#[test]
fn test_bounty_id_uniqueness_sequential() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let mut seen_ids = soroban_sdk::Vec::<soroban_sdk::BytesN<32>>::new(&env);

    for i in 0..20u64 {
        let bounty_id = create_test_bounty(&client, &env, &creator, 100);

        // Verify no duplicate IDs
        for j in 0..seen_ids.len() {
            assert_ne!(
                seen_ids.get(j).unwrap(), bounty_id,
                "Duplicate bounty ID at iteration {}", i
            );
        }
        seen_ids.push_back(bounty_id.clone());

        // Verify the ID encodes the correct counter value
        let id_bytes = bounty_id.to_array();
        let counter_bytes: [u8; 8] = id_bytes[24..32].try_into().unwrap();
        let encoded_count = u64::from_be_bytes(counter_bytes);
        assert_eq!(encoded_count, i, "Bounty ID should encode counter {}, got {}", i, encoded_count);
    }
}

// ===========================================================================
// Issue #255: complete_bounty panics when no assignee
// ===========================================================================

#[test]
#[should_panic(expected = "bounty has no assignee")]
fn test_complete_bounty_no_assignee_panics() {
    let (env, creator, _contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Create a bounty but do NOT claim it
    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "unclaimed"),
        &Symbol::new(&env, "no_assignee"), &1000, &Address::generate(&env), &None
    );

    // Verify the bounty has no assignee
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert!(bounty.assignee.is_none(), "Bounty should not have an assignee");

    // This should panic
    client.complete_bounty(&verifier, &bounty_id);
}

// ===========================================================================
// Issue #304: contributor reputation accumulates correctly
// ===========================================================================

#[test]
fn test_contributor_reputation_accumulation() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Initial contributor should not exist
    assert!(client.get_contributor(&contributor).is_none());

    let mut expected_reputation: u32 = 0;
    let mut expected_earned: i128 = 0;
    let mut expected_count: u32 = 0;

    for i in 0..3u32 {
        let reward: i128 = 1000 + (i as i128) * 500;
        let bounty_id = client.create_bounty(
            &creator, &Symbol::new(&env, "rep_b"),
            &Symbol::new(&env, "rep_d"), &reward, &Address::generate(&env), &None
        );
        client.claim_bounty(&contributor, &bounty_id);
        client.complete_bounty(&verifier, &bounty_id);

        expected_reputation += 10;
        expected_earned += reward;
        expected_count += 1;

        let data = client.get_contributor(&contributor).expect("Contributor should exist");
        assert_eq!(data.reputation, expected_reputation, "Reputation mismatch after completion {}", i + 1);
        assert_eq!(data.total_earned, expected_earned, "Total earned mismatch after completion {}", i + 1);
        assert_eq!(data.contribution_count, expected_count, "Contribution count mismatch after completion {}", i + 1);
        assert_eq!(data.address, contributor);
    }

    // Final verification
    let final_data = client.get_contributor(&contributor).unwrap();
    assert_eq!(final_data.reputation, 30);
    assert_eq!(final_data.total_earned, 1000 + 1500 + 2000);
    assert_eq!(final_data.contribution_count, 3);
}

#[test]
fn test_contributor_initial_state_after_first_completion() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward: i128 = 500;
    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "first"),
        &Symbol::new(&env, "completion"), &reward, &Address::generate(&env), &None
    );
    client.claim_bounty(&contributor, &bounty_id);
    client.complete_bounty(&verifier, &bounty_id);

    let data = client.get_contributor(&contributor).unwrap();
    assert_eq!(data.reputation, 10);
    assert_eq!(data.total_earned, 500);
    assert_eq!(data.contribution_count, 1);
    assert_eq!(data.address, contributor);
}

#[test]
fn test_raise_dispute_creator() {
    let (env, creator, contributor) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_dispute"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
    );

    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "disputed"));
}

#[test]
fn test_raise_dispute_assignee() {
    let (env, creator, contributor) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_dispute2"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
    );

    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "disputed"));
}

#[test]
#[should_panic(expected = "only creator or assignee can raise dispute")]
fn test_raise_dispute_third_party_fails() {
    let (env, creator, contributor) = setup_test();

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
    );

    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&third_party, &bounty_id);
}

#[test]
fn test_bounty_id_edge_cases() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let edge_cases = vec![0u64, 1, 255, 256, 65535, 65536, u32::MAX as u64, u32::MAX as u64 + 1, u64::MAX - 1];
    let mut ids = vec![];

    for _ in 0..edge_cases.len() {
        let id = client.create_bounty(
            &creator,
            &Symbol::new(&env, "test"),
            &Symbol::new(&env, "test"),
            &1000,
            &Address::generate(&env),
            &0,
        );
        ids.push(id);
    }

    assert_eq!(ids.len(), edge_cases.len());
    for i in 0..ids.len() {
        for j in i + 1..ids.len() {
            assert_ne!(ids[i], ids[j], "IDs at positions {} and {} must differ", i, j);
        }
    }
}

#[test]
fn test_status_index_tracks_bounty_lifecycle() {
    let (env, creator, contributor, verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let reward_token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = StellarAssetClient::new(&env, &reward_token);
    token_client.mint(&token_admin, &1000);
    token_client.transfer(&token_admin, &verifier, &1000);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "status_bounty"),
        &Symbol::new(&env, "desc"),
        &1000,
        &reward_token,
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
        &None,
    );
    client.cancel_bounty(&creator, &cancelled_bounty_id);
    assert_eq!(client.get_bounties_by_status(&open_status).len(), 0);
    assert_eq!(client.get_bounties_by_status(&cancelled_status).len(), 1);
}

// ===========================================================================
// Issue #319: claim_bounty deadline enforcement test
// ===========================================================================

/// Test that claim_bounty rejects claims after the deadline has passed.
/// This test simulates time passing via ledger sequence manipulation and verifies
/// that the deadline check triggers correctly with the expected error message.
#[test]
#[should_panic(expected = "bounty deadline has passed")]
fn test_claim_bounty_deadline_enforcement() {
    let (env, creator, contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Set ledger sequence to 100 (past the deadline)
    env.ledger().set_sequence_number(100);

    // Create a bounty with deadline at sequence 50
    // The deadline has already passed
    let bounty_id = make_bounty(&env, &client, &creator, "deadline_bounty", Some(50));

    // Attempting to claim should panic with "bounty deadline has passed"
    client.claim_bounty(&contributor, &bounty_id);
}

/// Test that claim_bounty allows claims when deadline has NOT passed.
#[test]
fn test_claim_bounty_allowed_before_deadline() {
    let (env, creator, contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Set ledger sequence to 10 (before the deadline)
    env.ledger().set_sequence_number(10);

    // Create a bounty with deadline at sequence 100
    let bounty_id = make_bounty(&env, &client, &creator, "future_bounty", Some(100));

    // Claim should succeed
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignee.unwrap(), contributor);
}

/// Test that claim_bounty works for bounties without a deadline.
#[test]
fn test_claim_bounty_no_deadline() {
    let (env, creator, contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Create a bounty without a deadline
    let bounty_id = make_bounty(&env, &client, &creator, "no_deadline_bounty", None);

    // Claim should succeed
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignee.unwrap(), contributor);
}

// ===========================================================================
// Issue #323: expire_bounty tests
// ===========================================================================

#[test]
fn test_expire_bounty_succeeds_after_deadline() {
    let (env, creator, _contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Ledger at sequence 100; deadline is sequence 50 — already passed.
    env.ledger().set_sequence_number(100);
    let bounty_id = make_bounty(&env, &client, &creator, "bounty_e", Some(50));

    // Permissionless: any authenticated caller can expire a past-deadline bounty.
    let anyone = Address::generate(&env);
    client.expire_bounty(&anyone, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "cancelled"));
}

#[test]
#[should_panic(expected = "bounty deadline has not passed")]
fn test_expire_bounty_fails_before_deadline() {
    let (env, creator, _contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Ledger at sequence 10; deadline is sequence 100 — not yet passed.
    env.ledger().set_sequence_number(10);
    let bounty_id = make_bounty(&env, &client, &creator, "bounty_f", Some(100));

    let anyone = Address::generate(&env);
    client.expire_bounty(&anyone, &bounty_id);
}

#[test]
#[should_panic(expected = "bounty is not open")]
fn test_expire_bounty_fails_on_completed_bounty() {
    let (env, creator, contributor, verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Set ledger to a high value to ensure deadline has passed
    env.ledger().set_sequence_number(100);

    // Create bounty with deadline in the past
    let bounty_id = make_bounty(&env, &client, &creator, "bounty_g", Some(50));

    // The claim will fail because deadline has passed, so we need to use a bounty
    // with no deadline or a future deadline that we can claim
    let claimable_bounty = make_bounty(&env, &client, &creator, "claimable_bounty", Some(1000));
    client.claim_bounty(&contributor, &claimable_bounty);
    client.complete_bounty(&verifier, &claimable_bounty);

    // Now try to expire the completed bounty - should fail because it's not open
    let anyone = Address::generate(&env);
    client.expire_bounty(&anyone, &claimable_bounty);
}

#[test]
#[should_panic(expected = "bounty has no deadline")]
fn test_expire_bounty_fails_without_deadline() {
    let (env, creator, _contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Create bounty without a deadline
    let bounty_id = make_bounty(&env, &client, &creator, "no_deadline_expiry", None);

    // Should panic because there's no deadline to expire
    let anyone = Address::generate(&env);
    client.expire_bounty(&anyone, &bounty_id);
}

// ===========================================================================
// Issue #322: cancel_bounty tests - non-creator cannot cancel
// ===========================================================================

#[test]
#[should_panic(expected = "not bounty creator")]
fn test_non_creator_cannot_cancel_bounty() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let attacker = Address::generate(&env);

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "bounty_d", None);

    // Must panic: attacker is not the bounty creator.
    client.cancel_bounty(&attacker, &bounty_id);
}
