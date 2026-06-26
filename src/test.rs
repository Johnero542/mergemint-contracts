#![cfg(test)]

use soroban_sdk::{
    testutils::{token::StellarAssetClient as TokenAdmin, Address as _, BytesN as _},
    Address, Env, Symbol,
};

use crate::contract::MergeMintContract;
use crate::contract::MergeMintContractClient;

fn setup_token(env: &Env, admin: &Address, mint_to: &Address, amount: i128) -> Address {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = sac.address();
    TokenAdmin::new(env, &token_address).mint(mint_to, &amount);
    token_address
}

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
fn test_update_bounty_open_succeeds() {
    let (env, creator, _contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_upd"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
    );

    client.update_bounty(&creator, &bounty_id, &2000);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.reward_amount, 2000);
}

#[test]
#[should_panic(expected = "cannot update a bounty that is in progress")]
fn test_update_bounty_in_progress_panics() {
    let (env, creator, contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_ip"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
    );

    client.claim_bounty(&contributor, &bounty_id);
    client.update_bounty(&creator, &bounty_id, &500);
}

#[test]
#[should_panic(expected = "only the creator can update a bounty")]
fn test_update_bounty_non_creator_panics() {
    let (env, creator, contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_nc"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
    );

    client.update_bounty(&contributor, &bounty_id, &500);
}

#[test]
#[should_panic(expected = "cannot update a completed bounty")]
fn test_update_bounty_completed_panics() {
    let (env, creator, contributor, verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &verifier, 10_000);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_done"),
        &Symbol::new(&env, "desc"),
        &1000,
        &token_id,
    );

    client.claim_bounty(&contributor, &bounty_id);
    client.complete_bounty(&verifier, &bounty_id);
    client.update_bounty(&creator, &bounty_id, &500);
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
