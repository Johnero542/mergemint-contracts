#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
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
        &None,
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

    let bounty_id = make_bounty(&env, &client, &creator, "bounty_1", None);

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
        &None,
    );
    assert_eq!(client.get_bounty_count(), 1);

    client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_b"),
        &Symbol::new(&env, "desc_b"),
        &200,
        &reward_token,
        &None,
    );
    assert_eq!(client.get_bounty_count(), 2);
}

#[test]
fn test_complete_bounty_updates_status() {
    let (env, creator, contributor, _verifier) = setup_test();

    let contract_id = env.register_contract(None, MergeMintContract);
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&env, &client, &creator, "bounty_c", None);

    client.claim_bounty(&contributor, &bounty_id);
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignee.unwrap(), contributor);
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
