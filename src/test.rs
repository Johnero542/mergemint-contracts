// SPDX-License-Identifier: MIT
#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, Symbol, Vec};

use crate::contract::MergeMintContract;
use crate::contract::MergeMintContractClient;

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

/// Create a bounty using a fresh reward token, min_reputation=0, no deadline,
/// and an empty tags list.
fn make_bounty(
    client: &MergeMintContractClient,
    env: &Env,
    creator: &Address,
    tag: &str,
    deadline: Option<u32>,
) -> crate::types::BountyId {
    client.create_bounty(
        creator,
        &Symbol::new(env, tag),
        &Symbol::new(env, "desc"),
        &1000,
        &Address::generate(env),
        &0,
        &deadline,
        &Vec::new(env),
    )
}

// ===========================================================================
// Issue 1 — bounty tags
// ===========================================================================

/// Tags supplied to create_bounty are stored and returned by get_bounty.
#[test]
fn test_tags_stored_and_retrieved() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let mut tags: Vec<Symbol> = Vec::new(&env);
    tags.push_back(Symbol::new(&env, "bug"));
    tags.push_back(Symbol::new(&env, "docs"));

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "tagged"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
        &tags,
    );

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.tags.len(), 2);
    assert_eq!(bounty.tags.get(0).unwrap(), Symbol::new(&env, "bug"));
    assert_eq!(bounty.tags.get(1).unwrap(), Symbol::new(&env, "docs"));
}

/// An empty tags vector is valid and results in a bounty with zero tags.
#[test]
fn test_empty_tags_valid() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "no_tags"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
        &Vec::new(&env),
    );

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.tags.len(), 0);
}

/// Exactly 5 tags is the maximum — must succeed.
#[test]
fn test_five_tags_allowed() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let mut tags: Vec<Symbol> = Vec::new(&env);
    tags.push_back(Symbol::new(&env, "a"));
    tags.push_back(Symbol::new(&env, "b"));
    tags.push_back(Symbol::new(&env, "c"));
    tags.push_back(Symbol::new(&env, "d"));
    tags.push_back(Symbol::new(&env, "e"));

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "max_tags"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
        &tags,
    );

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.tags.len(), 5);
}

/// Supplying more than 5 tags must panic with "too many tags".
#[test]
#[should_panic(expected = "too many tags")]
fn test_too_many_tags_panics() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let mut tags: Vec<Symbol> = Vec::new(&env);
    for _ in 0..6 {
        tags.push_back(Symbol::new(&env, "tag"));
    }

    client.create_bounty(
        &creator,
        &Symbol::new(&env, "overtags"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
        &tags,
    );
}

// ===========================================================================
// Issue 2 — get_bounties_by_creator
// ===========================================================================

/// Creating 3 bounties from one creator returns all 3 IDs.
#[test]
fn test_get_bounties_by_creator_returns_all() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    assert_eq!(client.get_bounties_by_creator(&creator).len(), 0);

    let id1 = make_bounty(&client, &env, &creator, "b1", None);
    let id2 = make_bounty(&client, &env, &creator, "b2", None);
    let id3 = make_bounty(&client, &env, &creator, "b3", None);

    let ids = client.get_bounties_by_creator(&creator);
    assert_eq!(ids.len(), 3);
    assert_eq!(ids.get(0).unwrap(), id1);
    assert_eq!(ids.get(1).unwrap(), id2);
    assert_eq!(ids.get(2).unwrap(), id3);
}

/// Bounties from different creators are indexed independently.
#[test]
fn test_get_bounties_by_creator_independent_lists() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let creator2 = Address::generate(&env);
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let id1 = make_bounty(&client, &env, &creator, "c1a", None);
    let id2 = make_bounty(&client, &env, &creator2, "c2a", None);

    let list1 = client.get_bounties_by_creator(&creator);
    let list2 = client.get_bounties_by_creator(&creator2);

    assert_eq!(list1.len(), 1);
    assert_eq!(list1.get(0).unwrap(), id1);
    assert_eq!(list2.len(), 1);
    assert_eq!(list2.get(0).unwrap(), id2);
}

/// An address that has never created a bounty returns an empty list.
#[test]
fn test_get_bounties_by_creator_unknown_address_empty() {
    let (env, _creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let stranger = Address::generate(&env);
    assert_eq!(client.get_bounties_by_creator(&stranger).len(), 0);
}

// ===========================================================================
// Issue 3 — dispute guard in complete_bounty
// ===========================================================================

/// complete_bounty on a disputed bounty must panic with "bounty is disputed".
#[test]
#[should_panic(expected = "bounty is disputed")]
fn test_complete_disputed_bounty_panics() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "disp_b", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);

    // Bounty is now "disputed" — complete_bounty must panic.
    client.complete_bounty(&verifier, &bounty_id);
}

/// The assignee raising a dispute also prevents completion.
#[test]
#[should_panic(expected = "bounty is disputed")]
fn test_complete_bounty_after_assignee_dispute_panics() {
    let (env, creator, contributor, verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "disp_c", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&contributor, &bounty_id);

    client.complete_bounty(&verifier, &bounty_id);
}

// ===========================================================================
// Issue 4 — ContractError enum (smoke-test the canonical messages)
// ===========================================================================

#[test]
fn test_contract_error_messages() {
    use crate::errors::{message, ContractError};

    assert_eq!(message(ContractError::BountyNotFound), "bounty not found");
    assert_eq!(
        message(ContractError::BountyAlreadyAssigned),
        "bounty already assigned"
    );
    assert_eq!(message(ContractError::BountyNotOpen), "bounty not open");
    assert_eq!(
        message(ContractError::BountyNotInProgress),
        "bounty not in progress"
    );
    assert_eq!(
        message(ContractError::BountyHasNoAssignee),
        "bounty has no assignee"
    );
    assert_eq!(
        message(ContractError::RewardMustBePositive),
        "reward_amount must be positive"
    );
    assert_eq!(
        message(ContractError::NotBountyCreator),
        "not bounty creator"
    );
    assert_eq!(
        message(ContractError::ContributorHasActiveClaim),
        "contributor already has an active claim"
    );
    assert_eq!(
        message(ContractError::BountyIsDisputed),
        "bounty is disputed"
    );
    assert_eq!(message(ContractError::TooManyTags), "too many tags");
    assert_eq!(
        message(ContractError::OnlyCreatorOrAssigneeCanDispute),
        "only creator or assignee can raise dispute"
    );
    assert_eq!(
        message(ContractError::DeadlineNotPassed),
        "deadline has not passed"
    );
    assert_eq!(
        message(ContractError::BountyDeadlinePassed),
        "bounty deadline passed"
    );
    assert_eq!(
        message(ContractError::BountyNoDeadline),
        "bounty has no deadline"
    );
    assert_eq!(
        message(ContractError::ReputationTooLow),
        "contributor reputation is too low"
    );
}

/// ContractError::TooManyTags is wired to the correct panic message.
#[test]
#[should_panic(expected = "too many tags")]
fn test_fail_too_many_tags_message() {
    use crate::errors::{fail, ContractError};
    fail(ContractError::TooManyTags);
}

/// ContractError::BountyIsDisputed is wired to the correct panic message.
#[test]
#[should_panic(expected = "bounty is disputed")]
fn test_fail_bounty_is_disputed_message() {
    use crate::errors::{fail, ContractError};
    fail(ContractError::BountyIsDisputed);
}

// ===========================================================================
// Existing tests — kept clean and compiling
// ===========================================================================

#[test]
fn test_create_bounty() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let reward_amount: i128 = 1000;
    let reward_token = Address::generate(&env);
    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "test_b"),
        &Symbol::new(&env, "desc"),
        &reward_amount,
        &reward_token,
        &0,
        &None,
        &Vec::new(&env),
    );

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.reward_amount, reward_amount);
    assert_eq!(bounty.creator, creator);
    assert!(bounty.assignees.is_empty());

    let meta = client.get_bounty_meta(&bounty_id).unwrap();
    assert_eq!(meta.title, Symbol::new(&env, "test_b"));
}

#[test]
fn test_claim_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "bounty_1"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
        &Vec::new(&env),
    );
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    let (assignee_addr, share) = bounty.assignees.get(0).unwrap();
    assert_eq!(assignee_addr, contributor);
    assert_eq!(share, 10_000u32);
}

#[test]
fn test_bounty_count() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

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
        &Vec::new(&env),
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
        &Vec::new(&env),
    );
    assert_eq!(client.get_bounty_count(), 2);
}

#[test]
fn test_single_assignee_gets_full_share() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "single", None);
    client.claim_bounty(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.assignees.len(), 1);
    let (addr, share) = bounty.assignees.get(0).unwrap();
    assert_eq!(addr, contributor);
    assert_eq!(share, 10_000u32);
}

// ===========================================================================
// Dispute handling
// ===========================================================================

#[test]
fn test_raise_dispute_creator() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "dispute_1", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "disputed"));
}

#[test]
fn test_raise_dispute_assignee() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "dispute_2", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&contributor, &bounty_id);

    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "disputed"));
}

#[test]
#[should_panic(expected = "only creator or assignee can raise dispute")]
fn test_raise_dispute_third_party_fails() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let third_party = Address::generate(&env);
    let bounty_id = make_bounty(&client, &env, &creator, "dispute_3", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&third_party, &bounty_id);
}

// ===========================================================================
// Claim guards
// ===========================================================================

#[test]
#[should_panic(expected = "contributor already has an active claim")]
fn test_second_claim_rejected_while_active() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id_1 = make_bounty(&client, &env, &creator, "active1", None);
    let bounty_id_2 = make_bounty(&client, &env, &creator, "active2", None);

    client.claim_bounty(&contributor, &bounty_id_1);
    // Second claim on a different bounty must be rejected while the first is active.
    client.claim_bounty(&contributor, &bounty_id_2);
}

#[test]
#[should_panic(expected = "bounty already assigned")]
fn test_second_contributor_cannot_claim_full_bounty() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "full_c", None);
    client.claim_bounty(&contributor, &bounty_id);

    // A different contributor tries to claim a full single-slot bounty.
    let contributor2 = Address::generate(&env);
    client.claim_bounty(&contributor2, &bounty_id);
}

// ===========================================================================
// Status index
// ===========================================================================

#[test]
fn test_status_index_open_on_create() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "status_open", None);

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    assert_eq!(open_ids.len(), 1);
    assert_eq!(open_ids.get(0).unwrap(), bounty_id);
}

#[test]
fn test_status_index_moves_on_cancel() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "bounty_z", None);
    client.cancel_bounty(&creator, &bounty_id);

    let open_ids = client.get_bounties_by_status(&Symbol::new(&env, "open"));
    let cancelled_ids = client.get_bounties_by_status(&Symbol::new(&env, "cancelled"));
    assert_eq!(open_ids.len(), 0);
    assert_eq!(cancelled_ids.len(), 1);
    assert_eq!(cancelled_ids.get(0).unwrap(), bounty_id);
}

// ===========================================================================
// Contributor metadata
// ===========================================================================

#[test]
fn test_update_contributor_metadata_stores_value() {
    let (env, _creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let uri = Symbol::new(&env, "ipfs_hash_1");
    client.update_contributor_metadata(&contributor, &uri);

    let data = client.get_contributor(&contributor).unwrap();
    assert_eq!(data.metadata.unwrap(), uri);
}

#[test]
fn test_update_contributor_metadata_overwrites() {
    let (env, _creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    client.update_contributor_metadata(&contributor, &Symbol::new(&env, "old_uri"));
    client.update_contributor_metadata(&contributor, &Symbol::new(&env, "new_uri"));

    let data = client.get_contributor(&contributor).unwrap();
    assert_eq!(data.metadata.unwrap(), Symbol::new(&env, "new_uri"));
}

// ===========================================================================
// Security: double-completion guard
// ===========================================================================

/// Calling complete_bounty on a bounty in "open" status must panic with
/// "bounty is not in progress".
#[test]
#[should_panic(expected = "bounty is not in progress")]
fn test_double_complete_panics() {
    let (env, creator, _contributor, verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "dbl_complete"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
        &Vec::new(&env),
    );

    // Bounty is "open", not "in_progress" — must panic.
    client.complete_bounty(&verifier, &bounty_id);
}

// ===========================================================================
// Security: self-verification guard
// ===========================================================================

/// The assignee calling complete_bounty as their own verifier must panic.
#[test]
#[should_panic(expected = "verifier cannot be the assignee")]
fn test_assignee_cannot_self_verify() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "self_verify"),
        &Symbol::new(&env, "desc"),
        &1000,
        &Address::generate(&env),
        &0,
        &None,
        &Vec::new(&env),
    );

    client.claim_bounty(&contributor, &bounty_id);

    // The assignee (contributor) attempts to act as their own verifier — must panic.
    client.complete_bounty(&contributor, &bounty_id);
}

// ===========================================================================
// Issue 32 — batch get_bounties query
// ===========================================================================

/// Batch get_bounties returns all bounties for valid IDs and None for unknown IDs.
#[test]
fn test_get_bounties_batch() {
    let (env, creator, _contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let id1 = make_bounty(&client, &env, &creator, "batch1", None);
    let id2 = make_bounty(&client, &env, &creator, "batch2", None);

    // Create a non-existent ID (all ones, different from any generated ID).
    let unknown_id = crate::types::BountyId(soroban_sdk::BytesN::from_array(
        &env,
        &[0xffu8; 32],
    ));

    let mut ids = Vec::new(&env);
    ids.push_back(id1.clone());
    ids.push_back(unknown_id.clone());
    ids.push_back(id2.clone());

    let results = client.get_bounties(&ids);

    assert_eq!(results.len(), 3);

    // First result: bounty exists.
    match results.get(0).unwrap() {
        Some(bounty) => assert_eq!(bounty.reward_amount, 1000),
        None => panic!("expected Some for known ID"),
    }

    // Second result: unknown ID returns None.
    match results.get(1).unwrap() {
        Some(_) => panic!("expected None for unknown ID"),
        None => {} // expected
    }

    // Third result: bounty exists.
    match results.get(2).unwrap() {
        Some(bounty) => assert_eq!(bounty.reward_amount, 1000),
        None => panic!("expected Some for known ID"),
    }
}

// ===========================================================================
// Issue 45 — update_contributor_metadata auth boundary
// ===========================================================================

/// Address A cannot update address B's metadata.
/// Uses a non-mocked-auth environment to verify that require_auth rejects
/// a mismatched signer.
#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn test_update_contributor_metadata_wrong_address_rejected() {
    let env = Env::default();
    let _contributor = Address::generate(&env);
    let stranger = Address::generate(&env);
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Without mock_all_auths(), require_auth() will reject the call because
    // the stranger address was not authorized by the caller.
    let uri = Symbol::new(&env, "test_uri");
    client.update_contributor_metadata(&stranger, &uri);
}

// ===========================================================================
// Issue 42 — resolve_dispute end-to-end coverage
// ===========================================================================

/// A non-creator (non-arbitrator) calling resolve_dispute must panic.
#[test]
#[should_panic(expected = "caller is not authorized to resolve this dispute")]
fn test_resolve_dispute_non_arbitrator_rejected() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "arb1", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);

    // An unrelated third party (non-creator) tries to resolve — must panic.
    let stranger = Address::generate(&env);
    client.resolve_dispute(&stranger, &bounty_id, &Symbol::new(&env, "complete"));
}

/// Resolving a bounty that is not in "disputed" status must panic.
#[test]
#[should_panic(expected = "bounty is not in disputed status")]
fn test_resolve_dispute_non_disputed_rejected() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "ndisp", None);
    client.claim_bounty(&contributor, &bounty_id);

    // Bounty is "in_progress", not "disputed" — must panic.
    client.resolve_dispute(&creator, &bounty_id, &Symbol::new(&env, "complete"));
}

/// Resolving with the "complete" resolution pays assignees from the contract
/// address and transitions the bounty to "completed" status.
#[test]
fn test_resolve_dispute_complete_pays_out() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Register a real token for the payout.
    let token_admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = sac.address();

    // Mint 1000 units to the contract address (the resolve_dispute "complete"
    // path transfers from env.current_contract_address()).
    let token = StellarAssetClient::new(&env, &token_addr);
    token.mint(&contract_id, &1000);

    // Create a bounty with the real token.
    let reward_amount: i128 = 1000;
    let bounty_id = client.create_bounty(
        &creator,
        &Symbol::new(&env, "comp_disp"),
        &Symbol::new(&env, "desc"),
        &reward_amount,
        &token_addr,
        &0,
        &None,
        &Vec::new(&env),
    );

    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);

    // Check pre-resolve balance: contract has 1000, assignee has 0.
    assert_eq!(token.balance(&contract_id), 1000);
    assert_eq!(token.balance(&contributor), 0);

    // Resolve with "complete".
    client.resolve_dispute(&creator, &bounty_id, &Symbol::new(&env, "complete"));

    // After resolve: contract balance is 0, assignee received 1000.
    assert_eq!(token.balance(&contract_id), 0);
    assert_eq!(token.balance(&contributor), 1000);

    // Bounty status is now "completed".
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "completed"));

    // Contributor reputation increased by 10.
    let contrib = client.get_contributor(&contributor).unwrap();
    assert_eq!(contrib.reputation, 10);
}

/// Resolving with the "cancel" resolution transitions the bounty to "cancelled"
/// status without any token transfer.
#[test]
fn test_resolve_dispute_cancel_transitions() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "cancel_disp", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);

    // Resolve with "cancel".
    client.resolve_dispute(&creator, &bounty_id, &Symbol::new(&env, "cancel"));

    // Bounty status is now "cancelled".
    let bounty = client.get_bounty(&bounty_id).unwrap();
    assert_eq!(bounty.status, Symbol::new(&env, "cancelled"));
}

/// An invalid resolution string must panic.
#[test]
#[should_panic(expected = "resolution must be 'complete' or 'cancel'")]
fn test_resolve_dispute_invalid_resolution() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    let bounty_id = make_bounty(&client, &env, &creator, "invalid_res", None);
    client.claim_bounty(&contributor, &bounty_id);
    client.raise_dispute(&creator, &bounty_id);

    // Use an invalid resolution.
    client.resolve_dispute(&creator, &bounty_id, &Symbol::new(&env, "invalid"));
}

// ===========================================================================
// Issue 438 — reputation monotonicity invariant
// ===========================================================================

/// Reputation never decreases across multiple completions for the same
/// contributor. After each of 3 completions, the contributor's reputation
/// must have increased by exactly 10 points.
#[test]
fn test_reputation_never_decreases_across_multiple_completions() {
    let (env, creator, contributor, _verifier) = setup_test();
    let contract_id = env.register(MergeMintContract, ());
    let client = MergeMintContractClient::new(&env, &contract_id);

    // Register a real token so the verifier can transfer.
    let token_admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = sac.address();

    // Mint 10_000 units to the verifier.
    let verifier = Address::generate(&env);
    let token = StellarAssetClient::new(&env, &token_addr);
    token.mint(&verifier, &10_000);

    // Create 3 bounties, each with the real token and 0 min_reputation.
    let mut bounty_ids = Vec::new(&env);
    for _ in 0..3 {
        let id = client.create_bounty(
            &creator,
            &Symbol::new(&env, "rep"),
            &Symbol::new(&env, "desc"),
            &1000,
            &token_addr,
            &0,
            &None,
            &Vec::new(&env),
        );
        bounty_ids.push_back(id);
    }

    // Claim and complete each bounty, checking reputation after each.
    for i in 0..3 {
        let id = bounty_ids.get(i).unwrap();
        client.claim_bounty(&contributor, &id);
        client.complete_bounty(&verifier, &id);

        let contrib = client.get_contributor(&contributor).unwrap();
        let expected_rep: u32 = (i + 1) * 10;
        assert_eq!(
            contrib.reputation, expected_rep,
            "reputation after {} completions should be {}",
            i + 1, expected_rep
        );
    }
}
