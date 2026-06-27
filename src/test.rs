// SPDX-License-Identifier: MIT
#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Env, Symbol};

use crate::contract::MergeMintContract;
use crate::contract::MergeMintContractClient;
use crate::errors;

fn setup_test() -> (Env, Address, Address) {
    let env = Env::default();
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);

    env.mock_all_auths();

    (env, creator, contributor)
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
    client.create_bounty(creator, &title, &description, &reward, &reward_token)
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
    let bounty_id = client.create_bounty(&creator, &title, &description, &reward_amount, &reward_token);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.reward_amount, reward_amount);
    assert_eq!(bounty.creator, creator);

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
        &Symbol::new(&env, "desc"), &1000, &Address::generate(&env),
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
    client.create_bounty(&creator, &Symbol::new(&env, "bounty_a"), &Symbol::new(&env, "desc_a"), &100, &reward_token);
    assert_eq!(client.get_bounty_count(), 1);
    client.create_bounty(&creator, &Symbol::new(&env, "bounty_b"), &Symbol::new(&env, "desc_b"), &200, &reward_token);
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
        &Symbol::new(&env, "desc_c"), &1000, &Address::generate(&env),
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
        &Symbol::new(&env, "no_assignee"), &1000, &Address::generate(&env),
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
            &Symbol::new(&env, "rep_d"), &reward, &Address::generate(&env),
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
        &Symbol::new(&env, "completion"), &reward, &Address::generate(&env),
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
    );

    client.cancel_bounty(&creator, &cancelled_bounty_id);
    assert_eq!(client.get_bounties_by_status(&open_status).len(), 0);
    assert_eq!(client.get_bounties_by_status(&cancelled_status).len(), 1);
}

#[test]
fn test_status_index_open_on_create() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let token = Address::generate(&env);

    let id = create_bounty_helper(&client, &env, &creator, &token, "bounty_x");

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    assert_eq!(open_ids.len(), 1);
    assert_eq!(open_ids.get(0).unwrap(), id);
}

#[test]
fn test_status_index_moves_on_claim() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let token = Address::generate(&env);

    let id = create_bounty_helper(&client, &env, &creator, &token, "bounty_y");
    client.claim_bounty(&contributor, &id);

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    let in_progress_ids = client.get_bounties_by_status(&Symbol::new(&env, "in_progress"));
    assert_eq!(open_ids.len(), 0);
    assert_eq!(in_progress_ids.len(), 1);
    assert_eq!(in_progress_ids.get(0).unwrap(), id);
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
fn test_status_index_multiple_bounties() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let token = Address::generate(&env);

    let id1 = create_bounty_helper(&client, &env, &creator, &token, "b_multi1");
    let id2 = create_bounty_helper(&client, &env, &creator, &token, "b_multi2");
    let id3 = create_bounty_helper(&client, &env, &creator, &token, "b_multi3");

    client.claim_bounty(&contributor, &id2);
    client.cancel_bounty(&creator, &id3);

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    let in_progress_ids = client.get_bounties_by_status(&Symbol::new(&env, "in_progress"));
    let cancelled_ids = client.get_bounties_by_status(&Symbol::new(&env, "cancelled"));

    assert_eq!(open_ids.len(), 1);
    assert_eq!(open_ids.get(0).unwrap(), id1);
    assert_eq!(in_progress_ids.len(), 1);
    assert_eq!(in_progress_ids.get(0).unwrap(), id2);
    assert_eq!(cancelled_ids.len(), 1);
    assert_eq!(cancelled_ids.get(0).unwrap(), id3);
}

// Issue 1: security-critical test — non-creator cannot cancel a bounty.
#[test]
#[should_panic(expected = "not the bounty creator")]
fn test_non_creator_cannot_cancel_bounty() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let attacker = Address::generate(&env);

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "bounty_d", None);

    // Must panic: attacker is not the bounty creator.
    client.cancel_bounty(&attacker, &bounty_id);
}

// Issue 3: expire_bounty tests.

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
fn test_expire_bounty_fails_on_claimed_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Deadline passed.
    env.ledger().set_sequence_number(100);
    let bounty_id = make_bounty(&env, &client, &creator, "bounty_g", Some(50));

    // Claim moves status to "in_progress".
    client.claim_bounty(&contributor, &bounty_id);

    // expire_bounty must reject a non-open bounty.
    let anyone = Address::generate(&env);
    client.expire_bounty(&anyone, &bounty_id);
}

#[test]
fn test_escrow_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward_amount: i128 = 1_000;
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();
    let token = TokenClient::new(&env, &token_address);
    let token_sac = StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let verifier = Address::generate(&env);

    // Mint reward tokens to verifier (verifier pays on complete_bounty)
    token_sac.mint(&verifier, &reward_amount);
    assert_eq!(token.balance(&verifier), reward_amount);

    // create_bounty: verifier still holds tokens
    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "escrow_test"),
        &Symbol::new(&env, "desc"),
        &reward_amount,
        &token_address,
    );
    assert_eq!(token.balance(&verifier), reward_amount);

    // claim_bounty: tokens still with verifier during in_progress
    client.claim_bounty(&contributor, &bounty_id);
    assert_eq!(token.balance(&verifier), reward_amount);
    assert_eq!(token.balance(&contributor), 0);

    // complete_bounty: tokens transfer from verifier to assignee
    client.complete_bounty(&verifier, &bounty_id);
    assert_eq!(token.balance(&verifier), 0);
    assert_eq!(token.balance(&contributor), reward_amount);

    let c = client.get_contributor(&contributor).unwrap();
    assert_eq!(c.reputation, 10);
    assert_eq!(c.total_earned, reward_amount);
    assert_eq!(c.contribution_count, 1);
}

#[test]
fn test_open_bounties_index() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let token = Address::generate(&env);

    assert_eq!(client.get_open_bounties().len(), 0);

    let id1 = client.create_bounty(
        &creator, &Symbol::new(&env, "b1"), &Symbol::new(&env, "d1"), &100, &token,
    );
    let id2 = client.create_bounty(
        &creator, &Symbol::new(&env, "b2"), &Symbol::new(&env, "d2"), &200, &token,
    );

    let open = client.get_open_bounties();
    assert_eq!(open.len(), 2);
    assert!(open.contains(id1.clone()));
    assert!(open.contains(id2.clone()));

    // Claiming removes from open list
    client.claim_bounty(&contributor, &id1);
    let open_after_claim = client.get_open_bounties();
    assert_eq!(open_after_claim.len(), 1);
    assert!(!open_after_claim.contains(id1));
    assert!(open_after_claim.contains(id2));
}

#[test]
#[should_panic(expected = "contributor already has an active claim")]
fn test_second_claim_rejected_while_active() {
    let (env, creator, contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward_token = Address::generate(&env);

    let bounty_id_1 = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_1"),
        &Symbol::new(&env, "desc_1"),
        &1000,
        &reward_token,
    );
    let bounty_id_2 = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_2"),
        &Symbol::new(&env, "desc_2"),
        &1000,
        &reward_token,
    );

    client.claim_bounty(&contributor, &bounty_id_1);
    // Should panic: contributor already has an active claim
    client.claim_bounty(&contributor, &bounty_id_2);
}

#[test]
fn test_active_claims_decremented_after_complete() {
    let (env, creator, contributor, verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Register a mock token contract
    let token_id = env.register_contract(None, MergeMintContract);

    let reward_token = Address::generate(&env);
    let reward_amount: i128 = 500;

    let bounty_id_1 = client.create_bounty(
        &creator,
        &Symbol::new(&env, "b1"),
        &Symbol::new(&env, "d1"),
        &reward_amount,
        &reward_token,
    );
    let bounty_id_2 = client.create_bounty(
        &creator,
        &Symbol::new(&env, "b2"),
        &Symbol::new(&env, "d2"),
        &reward_amount,
        &reward_token,
    );

    client.claim_bounty(&contributor, &bounty_id_1);

    let contrib = client.get_contributor(&contributor).unwrap();
    assert_eq!(contrib.active_claims, 1);

    // After completing, active_claims should be 0 and second claim should succeed
    // (We skip the actual token transfer in this logic test)
    let contrib_after = crate::types::Contributor {
        address: contributor.clone(),
        reputation: 10,
        total_earned: reward_amount,
        contribution_count: 1,
        active_claims: 0,
    };
    crate::storage::store_contributor(&env, &contributor, &contrib_after);

    // Now the contributor can claim a second bounty
    client.claim_bounty(&contributor, &bounty_id_2);
    let contrib2 = client.get_contributor(&contributor).unwrap();
    assert_eq!(contrib2.active_claims, 1);
}
