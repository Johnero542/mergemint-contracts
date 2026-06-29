// SPDX-License-Identifier: MIT
#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    Address, Env, Symbol,
};

use crate::contract::MergeMintContract;
use crate::contract::MergeMintContractClient;
use crate::types::BountyId;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn setup_test() -> (Env, Address, Address, Address) {
    let env = Env::default();
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let verifier = Address::generate(&env);
    env.mock_all_auths();
    (env, creator, contributor, verifier)
}

/// Register the MergeMint contract and return a client.
fn register(env: &Env) -> MergeMintContractClient {
    let contract_id = env.register_contract(None, MergeMintContract);
    MergeMintContractClient::new(env, &contract_id)
}

/// Create a minimal bounty (reward=1000, no min_reputation, no deadline).
fn create_bounty_simple(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    tag: &str,
) -> BountyId {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &1000,
        &Address::generate(env),
        &0,
        &None,
    )
}

/// Create a bounty using a specific reward token and amount.
fn create_bounty_with_token(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    tag: &str,
    reward_amount: i128,
    reward_token: &Address,
) -> BountyId {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &reward_amount,
        reward_token,
        &0,
        &None,
    )
}

// ===========================================================================
// Issue #1 — reward_amount validation in create_bounty
// ===========================================================================

/// Passing reward_amount = 0 must panic with a clear message before any
/// storage write occurs.
#[test]
#[should_panic(expected = "reward_amount must be positive")]
fn test_create_bounty_zero_reward_panics() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let client = register(&env);

    client.create_bounty(
        &creator,
        &Symbol::new(&env, "zero_r"),
        &Symbol::new(&env, "desc"),
        &0,
        &Address::generate(&env),
        &0,
        &None,
    );
}

/// Passing a negative reward_amount must panic with the same message.
#[test]
#[should_panic(expected = "reward_amount must be positive")]
fn test_create_bounty_negative_reward_panics() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let client = register(&env);

    client.create_bounty(
        &creator,
        &Symbol::new(&env, "neg_r"),
        &Symbol::new(&env, "desc"),
        &-1,
        &Address::generate(&env),
        &0,
        &None,
    );
}

// ===========================================================================
// Issue #2 — status guard in complete_bounty
// ===========================================================================

/// Calling complete_bounty on a bounty that is still "open" (never claimed)
/// must panic before any token transfer or state mutation occurs.
#[test]
#[should_panic(expected = "bounty is not in progress")]
fn test_complete_open_bounty_panics() {
    let (env, creator, _contributor, verifier) = setup_test();
    let client = register(&env);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "open_b");

    // Bounty is open — no assignee — calling complete_bounty must panic.
    client.complete_bounty(&verifier, &bounty_id);
}

// ===========================================================================
// Issue #4 — full happy-path test for complete_bounty with mock token
// ===========================================================================

/// Full lifecycle: create → claim → complete.
///
/// Verifies:
///   - assignee receives exactly `reward_amount` tokens
///   - contributor.reputation == 10
///   - contributor.total_earned == reward_amount
///   - contributor.contribution_count == 1
///   - bounty.status == "completed"
#[test]
fn test_complete_bounty_full_flow() {
    let env = Env::default();
    env.mock_all_auths();

    // Register a Soroban token (StellarAssetClient provides mint/admin helpers).
    let token_admin = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = TokenClient::new(&env, &token_contract_id.address());
    let stellar_asset = StellarAssetClient::new(&env, &token_contract_id.address());

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let verifier = Address::generate(&env);

    let reward_amount: i128 = 5_000_000; // 0.5 tokens (7 decimals)

    // Mint reward_amount tokens to the verifier (who pays the reward on complete).
    stellar_asset.mint(&verifier, &reward_amount);
    assert_eq!(token_client.balance(&verifier), reward_amount);

    // Register and set up MergeMint contract.
    let client = register(&env);

    // --- create ---
    let bounty_id = create_bounty_with_token(
        &client, &env, &creator, "full_lc", reward_amount, &token_contract_id.address(),
    );

    // Bounty is open; contributor has no profile yet.
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "open"));
    assert!(client.get_contributor(&contributor).is_none());

    // --- claim ---
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "in_progress"));
    let (assignee_addr, share) = bounty.assignees.get(0).unwrap();
    assert_eq!(assignee_addr, contributor);
    assert_eq!(share, 10_000u32); // 100% basis points for single assignee

    // --- complete ---
    client.complete_bounty(&verifier, &bounty_id);

    // Post-completion: token balance assertions.
    assert_eq!(
        token_client.balance(&contributor),
        reward_amount,
        "assignee did not receive reward tokens"
    );
    assert_eq!(
        token_client.balance(&verifier),
        0,
        "verifier balance should be zero after transfer"
    );

    // Post-completion: contributor profile assertions.
    let contrib = client
        .get_contributor(&contributor)
        .expect("contributor profile must exist after completion");
    assert_eq!(contrib.reputation, 10, "reputation must be +10 per completion");
    assert_eq!(contrib.total_earned, reward_amount, "total_earned must equal reward");
    assert_eq!(contrib.contribution_count, 1, "contribution_count must be 1");

    // Post-completion: bounty status assertion.
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(
        bounty.status,
        Symbol::new(&env, "completed"),
        "bounty status must be 'completed'"
    );
}

// ===========================================================================
// Existing tests — kept clean and compiling
// ===========================================================================

#[test]
fn test_create_bounty() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let client = register(&env);

    let reward_amount: i128 = 1000;
    let reward_token = Address::generate(&env);
    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "test_b"),
        &Symbol::new(&env, "test_d"),
        &reward_amount,
        &reward_token,
        &0,
        &None,
    );
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.reward_amount, reward_amount);
    assert_eq!(bounty.creator, creator);
    assert!(bounty.assignees.is_empty());
}

#[test]
fn test_claim_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let client = register(&env);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "bounty_1");
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    let (assignee_addr, share) = bounty.assignees.get(0).unwrap();
    assert_eq!(assignee_addr, contributor);
    assert_eq!(share, 10_000u32);
}

#[test]
fn test_bounty_count() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let client = register(&env);

    assert_eq!(client.get_bounty_count(), 0);
    let reward_token = Address::generate(&env);
    client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_a"),
        &Symbol::new(&env, "desc_a"),
        &100,
        &reward_token,
        &0,
        &None,
    );
    assert_eq!(client.get_bounty_count(), 1);
    client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_b"),
        &Symbol::new(&env, "desc_b"),
        &200,
        &reward_token,
        &0,
        &None,
    );
    assert_eq!(client.get_bounty_count(), 2);
}

#[test]
fn test_raise_dispute_creator() {
    let (env, creator, contributor, _verifier) = setup_test();
    let client = register(&env);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "bounty_dp1");
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "disputed"));
}

#[test]
fn test_raise_dispute_assignee() {
    let (env, creator, contributor, _verifier) = setup_test();
    let client = register(&env);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "bounty_dp2");
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "disputed"));
}

#[test]
#[should_panic(expected = "only creator or assignee can raise dispute")]
fn test_raise_dispute_third_party_fails() {
    let (env, creator, contributor, _verifier) = setup_test();
    let client = register(&env);

    let third_party = Address::generate(&env);
    let bounty_id = create_bounty_simple(&client, &env, &creator, "bounty_dp3");
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&third_party, &bounty_id);
}

#[test]
#[should_panic(expected = "bounty already assigned")]
fn test_second_contributor_cannot_claim_full_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let client = register(&env);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "full_c");
    client.claim_bounty(&contributor, &bounty_id);

    let contributor2 = Address::generate(&env);
    client.claim_bounty(&contributor2, &bounty_id);
}

#[test]
fn test_single_assignee_gets_full_share() {
    let (env, creator, contributor, _verifier) = setup_test();
    let client = register(&env);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "single");
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignees.len(), 1);
    let (addr, share) = bounty.assignees.get(0).unwrap();
    assert_eq!(addr, contributor);
    assert_eq!(share, 10_000u32);
}

#[test]
fn test_status_index_open_on_create() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let client = register(&env);

    let bounty_id = create_bounty_simple(&client, &env, &creator, "open_idx");

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    assert_eq!(open_ids.len(), 1);
    assert_eq!(open_ids.get(0).unwrap(), bounty_id);
}

#[test]
fn test_status_index_moves_on_cancel() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let client = register(&env);

    let id = create_bounty_simple(&client, &env, &creator, "cancel_idx");
    client.cancel_bounty(&creator, &id);

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    let cancelled_ids = client.get_bounties_by_status(&Symbol::new(&env, "cancelled"));
    assert_eq!(open_ids.len(), 0);
    assert_eq!(cancelled_ids.len(), 1);
    assert_eq!(cancelled_ids.get(0).unwrap(), id);
}

#[test]
fn test_update_contributor_metadata_stores_value() {
    let (env, _creator, contributor, _verifier) = setup_test();
    let client = register(&env);

    let uri = Symbol::new(&env, "ipfs_hash_1");
    client.update_contributor_metadata(&contributor, &uri);

    let data = client.get_contributor(&contributor).unwrap();
    assert_eq!(data.metadata.unwrap(), uri);
}

#[test]
fn test_update_contributor_metadata_overwrite() {
    let (env, _creator, contributor, _verifier) = setup_test();
    let client = register(&env);

    client.update_contributor_metadata(&contributor, &Symbol::new(&env, "old_uri"));
    client.update_contributor_metadata(&contributor, &Symbol::new(&env, "new_uri"));

    let data = client.get_contributor(&contributor).unwrap();
    assert_eq!(data.metadata.unwrap(), Symbol::new(&env, "new_uri"));
}

#[test]
fn test_contributor_metadata_default_none() {
    let (env, _creator, contributor, _verifier) = setup_test();
    let client = register(&env);

    // No activity — should have no profile at all.
    assert!(client.get_contributor(&contributor).is_none());
}
