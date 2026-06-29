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

/// Register a mock token, mint `amount` to `recipient`, return token address.
fn setup_token(env: &Env, recipient: &Address, amount: i128) -> Address {
    let admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    StellarAssetClient::new(env, &token_id).mint(recipient, &amount);
    token_id
}

/// Create a bounty using a real token (for escrow tests).
fn create_bounty_with_token(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    reward_token: &Address,
    reward_amount: i128,
    tag: &str,
    verifier: Option<Address>,
) -> BytesN<32> {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &reward_amount,
        reward_token,
        &0u32,
        &None,
        &verifier,
    )
}

/// Create a bounty with a dummy token address (for non-escrow-flow tests).
fn create_bounty_simple(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    tag: &str,
) -> BytesN<32> {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &1000,
        &Address::generate(env),
        &0u32,
        &None,
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
    let reward_token = setup_token(&env, &creator, reward_amount);
    let bounty_id = create_bounty_with_token(&client, &env, &creator, &reward_token, reward_amount, "test_b", None);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.reward_amount, reward_amount);
    assert_eq!(bounty.creator, creator);
    assert!(bounty.assignees.is_empty());
    assert!(bounty.verifier.is_none());

    let meta = client.get_bounty_meta(&bounty_id).unwrap();
    assert_eq!(meta.title, Symbol::new(&env, "test_b"));
}

#[test]
fn test_claim_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward_amount: i128 = 1000;
    let reward_token = setup_token(&env, &creator, reward_amount);
    let bounty_id = create_bounty_with_token(&client, &env, &creator, &reward_token, reward_amount, "claim_b", None);

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

    let token_a = setup_token(&env, &creator, 2000);
    client.create_bounty(&creator, &Symbol::new(&env, "b_a"), &Symbol::new(&env, "d"), &100, &token_a, &0u32, &None, &None);
    assert_eq!(client.get_bounty_count(), 1);
    client.create_bounty(&creator, &Symbol::new(&env, "b_b"), &Symbol::new(&env, "d"), &200, &token_a, &0u32, &None, &None);
    assert_eq!(client.get_bounty_count(), 2);
}

#[test]
fn test_bounty_count_increment_loop() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward_token = setup_token(&env, &creator, 5000);
    for i in 0..5u64 {
        client.create_bounty(&creator, &Symbol::new(&env, "b"), &Symbol::new(&env, "d"), &1000, &reward_token, &0u32, &None, &None);
        assert_eq!(client.get_bounty_count(), i + 1);
    }
}

// ===========================================================================
// #263: Escrow tests
// ===========================================================================

#[test]
fn test_escrow_locks_tokens_on_create() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward: i128 = 500;
    let token_id = setup_token(&env, &creator, reward);
    let token = soroban_sdk::token::TokenClient::new(&env, &token_id);

    assert_eq!(token.balance(&creator), reward);
    create_bounty_with_token(&client, &env, &creator, &token_id, reward, "escrow_b", None);
    assert_eq!(token.balance(&creator), 0);
    assert_eq!(token.balance(&contract_id), reward);
}

#[test]
fn test_escrow_pays_assignee_on_complete() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward: i128 = 1000;
    let token_id = setup_token(&env, &creator, reward);
    let token = soroban_sdk::token::TokenClient::new(&env, &token_id);

    let bounty_id = create_bounty_with_token(&client, &env, &creator, &token_id, reward, "escrow_c", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.complete_bounty(&verifier, &bounty_id);

    assert_eq!(token.balance(&contributor), reward);
    assert_eq!(token.balance(&contract_id), 0);

    let c = client.get_contributor(&contributor).unwrap();
    assert_eq!(c.reputation, 10);
    assert_eq!(c.total_earned, reward);
    assert_eq!(c.contribution_count, 1);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "completed"));
}

#[test]
fn test_escrow_refunds_creator_on_cancel() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward: i128 = 800;
    let token_id = setup_token(&env, &creator, reward);
    let token = soroban_sdk::token::TokenClient::new(&env, &token_id);

    let bounty_id = create_bounty_with_token(&client, &env, &creator, &token_id, reward, "cancel_b", None);
    assert_eq!(token.balance(&creator), 0);

    client.cancel_bounty(&creator, &bounty_id);
    assert_eq!(token.balance(&creator), reward);
    assert_eq!(token.balance(&contract_id), 0);
}

// ===========================================================================
// #264: Designated verifier tests
// ===========================================================================

#[test]
fn test_designated_verifier_succeeds() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward: i128 = 1000;
    let token_id = setup_token(&env, &creator, reward);

    let bounty_id = create_bounty_with_token(&client, &env, &creator, &token_id, reward, "desig_b", Some(verifier.clone()));
    client.claim_bounty(&contributor, &bounty_id);
    client.complete_bounty(&verifier, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "completed"));
}

#[test]
#[should_panic(expected = "caller is not the designated verifier")]
fn test_wrong_verifier_panics() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward: i128 = 1000;
    let token_id = setup_token(&env, &creator, reward);

    let bounty_id = create_bounty_with_token(&client, &env, &creator, &token_id, reward, "desig_c", Some(verifier.clone()));
    client.claim_bounty(&contributor, &bounty_id);

    let wrong = Address::generate(&env);
    client.complete_bounty(&wrong, &bounty_id);
}

#[test]
fn test_none_verifier_allows_any_caller() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward: i128 = 1000;
    let token_id = setup_token(&env, &creator, reward);

    let bounty_id = create_bounty_with_token(&client, &env, &creator, &token_id, reward, "any_v", None);
    client.claim_bounty(&contributor, &bounty_id);

    let anyone = Address::generate(&env);
    client.complete_bounty(&anyone, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "completed"));
}

// ===========================================================================
// #271: get_top_contributors leaderboard
// ===========================================================================

#[test]
fn test_get_top_contributors_sorted_by_reputation() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Create 3 contributors with different reputation scores by completing bounties.
    // contrib_a completes 3 bounties (30 rep), contrib_b 1 (10 rep), contrib_c 2 (20 rep).
    let contrib_a = Address::generate(&env);
    let contrib_b = Address::generate(&env);
    let contrib_c = Address::generate(&env);
    let anyone = Address::generate(&env);

    let mut complete_bounties_for = |contributor: &Address, n: u32| {
        for i in 0..n {
            let reward: i128 = 100;
            let token_id = setup_token(&env, &creator, reward);
            let tag = if i == 0 { "b0" } else if i == 1 { "b1" } else { "b2" };
            let bounty_id = create_bounty_with_token(&client, &env, &creator, &token_id, reward, tag, None);
            client.claim_bounty(contributor, &bounty_id);
            client.complete_bounty(&anyone, &bounty_id);
        }
    };

    complete_bounties_for(&contrib_a, 3); // 30 rep
    complete_bounties_for(&contrib_b, 1); // 10 rep
    complete_bounties_for(&contrib_c, 2); // 20 rep

    let top3 = client.get_top_contributors(&3u32);
    assert_eq!(top3.len(), 3);
    assert_eq!(top3.get(0).unwrap().address, contrib_a);
    assert_eq!(top3.get(0).unwrap().reputation, 30);
    assert_eq!(top3.get(1).unwrap().address, contrib_c);
    assert_eq!(top3.get(1).unwrap().reputation, 20);
    assert_eq!(top3.get(2).unwrap().address, contrib_b);
    assert_eq!(top3.get(2).unwrap().reputation, 10);

    // limit = 2 returns only top 2
    let top2 = client.get_top_contributors(&2u32);
    assert_eq!(top2.len(), 2);
    assert_eq!(top2.get(0).unwrap().address, contrib_a);
    assert_eq!(top2.get(1).unwrap().address, contrib_c);
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

#[test]
fn test_contributor_metadata_default_none() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator, &Symbol::new(&env, "meta_b"), &Symbol::new(&env, "d"), &100,
        &Address::generate(&env), &0,
    );
    client.claim_bounty(&contributor, &bounty_id);
    client.complete_bounty(&verifier, &bounty_id);

    let data = client.get_contributor(&contributor).unwrap();
    assert!(data.metadata.is_none());
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
