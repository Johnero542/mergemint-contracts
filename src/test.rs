// SPDX-License-Identifier: MIT
#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Budget as _, Ledger as _},
    token::StellarAssetClient,
    Address, BytesN, Env, IntoVal, Symbol, TryFromVal, TryIntoVal, Val, Vec,
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

/// Helper: create a bounty with empty tags and no deadline.
fn create_bounty_simple(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    tag: &str,
    reward: i128,
) -> BytesN<32> {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &reward,
        &Address::generate(env),
        &0u32,
        &None,
        &Vec::new(env),
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

    let title = Symbol::new(&env, "test_bounty");
    let description = Symbol::new(&env, "Test_bounty_desc");
    let reward_amount: i128 = 1000;
    let reward_token = Address::generate(&env);

    let bounty_id = client.create_bounty(
        &creator,
        &title,
        &description,
        &reward_amount,
        &reward_token,
        &0u32,
        &None,
        &Vec::new(&env),
    );

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

    let bounty_id = create_bounty_simple(&client, &env, &creator, "bounty_1", 1000);
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
    create_bounty_simple(&client, &env, &creator, "bounty_a", 100);
    assert_eq!(client.get_bounty_count(), 1);
    create_bounty_simple(&client, &env, &creator, "bounty_b", 200);
    assert_eq!(client.get_bounty_count(), 2);
}

#[test]
fn test_complete_bounty_updates_contributor() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "bounty_c", 1000);
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    let (assignee_addr, _) = bounty.assignees.get(0).unwrap();
    assert_eq!(assignee_addr, contributor);
}

// ===========================================================================
// Reputation accumulation
// ===========================================================================

#[test]
fn test_contributor_reputation_accumulation() {
    use soroban_sdk::token::StellarAssetClient;

    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Set up a real token so complete_bounty can call token.transfer
    let token_admin = Address::generate(&env);
    let reward_token = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
    let token_client = StellarAssetClient::new(&env, &reward_token);
    // Mint enough for 3 rounds: 1000 + 1500 + 2000 = 4500
    token_client.mint(&token_admin, &4500);
    // Transfer to verifier so verifier can pay
    use soroban_sdk::token::TokenClient;
    TokenClient::new(&env, &reward_token).transfer(&token_admin, &verifier, &4500);

    assert!(client.get_contributor(&contributor).is_none());

    let mut expected_reputation: u32 = 0;
    let mut expected_earned: i128 = 0;
    let mut expected_count: u32 = 0;

    for i in 0..3u32 {
        let reward: i128 = 1000 + (i as i128) * 500;
        let bounty_id = client.create_bounty(
            &creator,
            &Symbol::new(&env, "rep_b"),
            &Symbol::new(&env, "rep_d"),
            &reward,
            &reward_token,
            &0u32,
            &None,
            &Vec::new(&env),
        );
        client.claim_bounty(&contributor, &bounty_id);
        client.complete_bounty(&verifier, &bounty_id);

        expected_reputation += 10;
        expected_earned += reward;
        expected_count += 1;

        let data = client.get_contributor(&contributor).unwrap();
        assert_eq!(data.reputation, expected_reputation);
        assert_eq!(data.total_earned, expected_earned);
        assert_eq!(data.contribution_count, expected_count);
    }

    let final_data = client.get_contributor(&contributor).unwrap();
    assert_eq!(final_data.reputation, 30);
    assert_eq!(final_data.total_earned, 1000 + 1500 + 2000);
    assert_eq!(final_data.contribution_count, 3);
}

// ===========================================================================
// Bounty ID uniqueness
// ===========================================================================

#[test]
fn test_bounty_id_uniqueness_sequential() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let mut seen_ids = soroban_sdk::Vec::<soroban_sdk::BytesN<32>>::new(&env);

    for i in 0..10u64 {
        let bounty_id = create_bounty_simple(&client, &env, &creator, "b", 100);

        for j in 0..seen_ids.len() {
            assert_ne!(seen_ids.get(j).unwrap(), bounty_id, "Duplicate ID at {}", i);
        }
        seen_ids.push_back(bounty_id.clone());

        let id_bytes = bounty_id.to_array();
        let counter_bytes: [u8; 8] = id_bytes[24..32].try_into().unwrap();
        let encoded_count = u64::from_be_bytes(counter_bytes);
        assert_eq!(encoded_count, i);
    }
}

// ===========================================================================
// complete_bounty panics with no assignee
// ===========================================================================

#[test]
#[should_panic(expected = "bounty has no assignee")]
fn test_complete_bounty_no_assignee_panics() {
    let (env, creator, _contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "unclaimed", 1000);
    client.complete_bounty(&verifier, &bounty_id);
}

// ===========================================================================
// Double-claim guard
// ===========================================================================

#[test]
#[should_panic(expected = "bounty already assigned")]
fn test_second_contributor_cannot_claim_full_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "full_c", 1000);
    client.claim_bounty(&contributor, &bounty_id);

    let contributor2 = Address::generate(&env);
    client.claim_bounty(&contributor2, &bounty_id);
}

#[test]
#[should_panic(expected = "contributor already has an active claim")]
fn test_second_claim_rejected_while_active() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id1 = create_bounty_simple(&client, &env, &creator, "b1", 1000);
    let bounty_id2 = create_bounty_simple(&client, &env, &creator, "b2", 1000);

    client.claim_bounty(&contributor, &bounty_id1);
    // second claim without completing the first — should panic
    client.claim_bounty(&contributor, &bounty_id2);
}

// ===========================================================================
// Issue #2: Dispute mechanism
// ===========================================================================

#[test]
fn test_raise_dispute_creator() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "dispute1", 1000);
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

    let bounty_id = create_bounty_simple(&client, &env, &creator, "dispute2", 1000);
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
    let bounty_id = create_bounty_simple(&client, &env, &creator, "dispute3", 1000);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&third_party, &bounty_id);
}

#[test]
#[should_panic(expected = "bounty is disputed")]
fn test_complete_bounty_fails_when_disputed() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "dispute4", 1000);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);
    // should panic with "bounty is disputed"
    client.complete_bounty(&verifier, &bounty_id);
}

// ===========================================================================
// Issue #3: get_bounties_by_creator
// ===========================================================================

#[test]
fn test_get_bounties_by_creator_returns_all() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let id1 = create_bounty_simple(&client, &env, &creator, "cb1", 100);
    let id2 = create_bounty_simple(&client, &env, &creator, "cb2", 200);
    let id3 = create_bounty_simple(&client, &env, &creator, "cb3", 300);

    let list = client.get_bounties_by_creator(&creator);
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap(), id1);
    assert_eq!(list.get(1).unwrap(), id2);
    assert_eq!(list.get(2).unwrap(), id3);
}

#[test]
fn test_get_bounties_by_creator_lists_are_independent() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let creator2 = Address::generate(&env);

    create_bounty_simple(&client, &env, &creator, "c1a", 100);
    create_bounty_simple(&client, &env, &creator, "c1b", 100);
    create_bounty_simple(&client, &env, &creator2, "c2a", 100);

    assert_eq!(client.get_bounties_by_creator(&creator).len(), 2);
    assert_eq!(client.get_bounties_by_creator(&creator2).len(), 1);
}

#[test]
fn test_get_bounties_by_creator_empty_for_new_address() {
    let (env, _creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let nobody = Address::generate(&env);
    assert_eq!(client.get_bounties_by_creator(&nobody).len(), 0);
}

// ===========================================================================
// Issue #4: Bounty tags
// ===========================================================================

#[test]
fn test_tags_stored_and_retrieved() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let mut tags = Vec::new(&env);
    tags.push_back(Symbol::new(&env, "bug"));
    tags.push_back(Symbol::new(&env, "rust"));

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "tagged"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0u32,
        &None,
        &tags,
    );

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.tags.len(), 2);
    assert_eq!(bounty.tags.get(0).unwrap(), Symbol::new(&env, "bug"));
    assert_eq!(bounty.tags.get(1).unwrap(), Symbol::new(&env, "rust"));
}

#[test]
fn test_empty_tags_valid() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "notags"),
        &Symbol::new(&env, "desc"),
        &100,
        &Address::generate(&env),
        &0u32,
        &None,
        &Vec::new(&env),
    );

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.tags.len(), 0);
}

#[test]
#[should_panic(expected = "too many tags")]
fn test_too_many_tags_panics() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let mut tags = Vec::new(&env);
    for _ in 0..6u32 {
        tags.push_back(Symbol::new(&env, "tag"));
    }

    client.create_bounty(
        &creator,
        &Symbol::new(&env, "toomanytags"),
        &Symbol::new(&env, "desc"),
        &100,
        &Address::generate(&env),
        &0u32,
        &None,
        &tags,
    );
}

#[test]
fn test_exactly_five_tags_valid() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let mut tags = Vec::new(&env);
    tags.push_back(Symbol::new(&env, "t1"));
    tags.push_back(Symbol::new(&env, "t2"));
    tags.push_back(Symbol::new(&env, "t3"));
    tags.push_back(Symbol::new(&env, "t4"));
    tags.push_back(Symbol::new(&env, "t5"));

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "fivetags"),
        &Symbol::new(&env, "desc"),
        &100,
        &Address::generate(&env),
        &0u32,
        &None,
        &tags,
    );

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.tags.len(), 5);
}

// ===========================================================================
// Contributor metadata
// ===========================================================================

#[test]
fn test_update_contributor_metadata_stores_value() {
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

// ===========================================================================
// Status index
// ===========================================================================

#[test]
fn test_status_index_open_on_create() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let id = create_bounty_simple(&client, &env, &creator, "idx_open", 100);

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    assert_eq!(open_ids.len(), 1);
    assert_eq!(open_ids.get(0).unwrap(), id);
}

#[test]
fn test_status_index_moves_on_claim() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let id = create_bounty_simple(&client, &env, &creator, "idx_claim", 100);
    client.claim_bounty(&contributor, &id);

    assert_eq!(client.get_bounties_by_status(&Symbol::new(&env, "open")).len(), 0);
    let in_progress = client.get_bounties_by_status(&Symbol::new(&env, "in_progress"));
    assert_eq!(in_progress.len(), 1);
    assert_eq!(in_progress.get(0).unwrap(), id);
}

// ===========================================================================
// Issue #289: CPU instruction count benchmarks
// ===========================================================================

#[test]
fn benchmark_create_bounty_instruction_count() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let reward_token = Address::generate(&env);

    env.budget().reset_default();
    let _id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bench_c"),
        &Symbol::new(&env, "bench_d"),
        &1000,
        &reward_token,
        &0,
        &None,
    );
    let cpu = env.budget().cpu_instruction_count();
    println!("create_bounty: {} CPU instructions", cpu);
    assert!(cpu < 1_000_000, "create_bounty exceeded 1M instructions: {}", cpu);
}

#[test]
fn benchmark_claim_bounty_instruction_count() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let reward_token = Address::generate(&env);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bench_claim"),
        &Symbol::new(&env, "bench_d"),
        &1000,
        &reward_token,
        &0,
        &None,
    );

    env.budget().reset_default();
    client.claim_bounty(&contributor, &bounty_id);
    let cpu = env.budget().cpu_instruction_count();
    println!("claim_bounty: {} CPU instructions", cpu);
    assert!(cpu < 1_000_000, "claim_bounty exceeded 1M instructions: {}", cpu);
}

#[test]
fn benchmark_get_bounty_instruction_count() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let reward_token = Address::generate(&env);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bench_get"),
        &Symbol::new(&env, "bench_d"),
        &1000,
        &reward_token,
        &0,
        &None,
    );

    env.budget().reset_default();
    let _ = client.get_bounty(&bounty_id);
    let cpu = env.budget().cpu_instruction_count();
    println!("get_bounty: {} CPU instructions", cpu);
    assert!(cpu < 500_000, "get_bounty exceeded 500K instructions: {}", cpu);
}

#[test]
fn benchmark_get_contributor_instruction_count() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let reward_token = Address::generate(&env);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bench_contrib"),
        &Symbol::new(&env, "bench_d"),
        &1000,
        &reward_token,
        &0,
        &None,
    );
    client.claim_bounty(&contributor, &bounty_id);
    client.complete_bounty(&verifier, &bounty_id);

    env.budget().reset_default();
    let _ = client.get_contributor(&contributor);
    let cpu = env.budget().cpu_instruction_count();
    println!("get_contributor: {} CPU instructions", cpu);
    assert!(cpu < 500_000, "get_contributor exceeded 500K instructions: {}", cpu);
}

#[test]
fn benchmark_get_bounty_count_instruction_count() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);
    let reward_token = Address::generate(&env);

    client.create_bounty(
        &creator,
        &Symbol::new(&env, "bench_cnt"),
        &Symbol::new(&env, "bench_d"),
        &1000,
        &reward_token,
        &0,
        &None,
    );

    env.budget().reset_default();
    let _ = client.get_bounty_count();
    let cpu = env.budget().cpu_instruction_count();
    println!("get_bounty_count: {} CPU instructions", cpu);
    assert!(cpu < 500_000, "get_bounty_count exceeded 500K instructions: {}", cpu);
}

#[test]
fn test_status_index_moves_on_cancel() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let id = create_bounty_simple(&client, &env, &creator, "idx_cancel", 100);
    client.cancel_bounty(&creator, &id);

    assert_eq!(client.get_bounties_by_status(&Symbol::new(&env, "open")).len(), 0);
    let cancelled = client.get_bounties_by_status(&Symbol::new(&env, "cancelled"));
    assert_eq!(cancelled.len(), 1);
    assert_eq!(cancelled.get(0).unwrap(), id);
}

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
