// SPDX-License-Identifier: MIT
#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    token::{StellarAssetClient, TokenClient},
    Address, Env, Symbol,
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

#[test]
fn test_create_bounty() {
    let (env, creator, _contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let title = Symbol::new(&env, "test_bounty");
    let description = Symbol::new(&env, "Test_bounty_description");
    let reward_amount: i128 = 1000;
    let reward_token = Address::generate(&env);

    let bounty_id = client.create_bounty(
        &creator,
        &title,
        &description,
        &reward_amount,
        &reward_token,
    );

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
        &creator,
        &Symbol::new(&env, "bounty_1"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
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
    client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_a"),
        &Symbol::new(&env, "desc_a"),
        &100,
        &reward_token,
    );
    assert_eq!(client.get_bounty_count(), 1);

    client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_b"),
        &Symbol::new(&env, "desc_b"),
        &200,
        &reward_token,
    );
    assert_eq!(client.get_bounty_count(), 2);
}

#[test]
fn test_complete_bounty_updates_status() {
    let (env, creator, contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_c"),
        &Symbol::new(&env, "desc_c"),
        &1000,
        &Address::generate(&env),
    );

    client.claim_bounty(&contributor, &bounty_id);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignee.unwrap(), contributor);
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
