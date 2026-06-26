#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, Env, Symbol,
};

use crate::contract::MergeMintContract;
use crate::contract::MergeMintContractClient;

fn create_bounty_helper(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    token: &Address,
    name: &str,
) -> soroban_sdk::BytesN<32> {
    client.create_bounty(
        creator,
        &Symbol::new(env, name),
        &Symbol::new(env, "desc"),
        &1000,
        token,
    )
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
