#![cfg(test)]

use crate::{BountyContract, BountyContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env, String,
};

#[test]
fn test_claim_bounty_deadline_enforcement() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let hunter = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 100u32;

    client.create_bounty(&creator, &amount, &deadline, &String::from_str(&env, "Test bounty"));

    env.ledger().set(LedgerInfo {
        timestamp: 0,
        protocol_version: 20,
        sequence_number: deadline + 1,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 16,
        max_entry_ttl: 6312000,
    });

    let result = client.try_claim_bounty(&hunter, &0);
    assert!(result.is_err());
}

#[test]
fn test_claim_bounty_before_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let hunter = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 100u32;

    client.create_bounty(&creator, &amount, &deadline, &String::from_str(&env, "Test bounty"));

    env.ledger().set(LedgerInfo {
        timestamp: 0,
        protocol_version: 20,
        sequence_number: deadline - 1,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 16,
        max_entry_ttl: 6312000,
    });

    client.claim_bounty(&hunter, &0);
}
