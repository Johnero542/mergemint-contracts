use soroban_sdk::{contract, contractimpl, token::TokenClient, Address, BytesN, Env, Symbol};

use crate::errors;
use crate::events;
use crate::storage;
use crate::types::{Bounty, BountyMeta, Contributor};

const STATUS_OPEN: &str = "open";
const STATUS_IN_PROGRESS: &str = "in_progress";
const STATUS_COMPLETED: &str = "completed";

fn generate_bounty_id(env: &Env, count: u64) -> BytesN<32> {
    let mut buf = [0u8; 32];
    let count_bytes = count.to_be_bytes();
    buf[24..32].copy_from_slice(&count_bytes);
    BytesN::from_array(env, &buf)
}

#[contract]
pub struct MergeMintContract;

#[contractimpl]
impl MergeMintContract {
    pub fn create_bounty(
        env: Env,
        creator: Address,
        title: Symbol,
        description: Symbol,
        reward_amount: i128,
        reward_token: Address,
        min_reputation: u32,
    ) -> BytesN<32> {
        creator.require_auth();

        let count = storage::get_bounty_count(&env);
        let id = generate_bounty_id(&env, count);

        let bounty = Bounty {
            creator,
            reward_amount,
            reward_token,
            assignee: None,
            status: Symbol::new(&env, STATUS_OPEN),
            min_reputation,
        };

        let meta = BountyMeta { title, description };

        storage::store_bounty(&env, &id, &bounty);
        storage::store_bounty_meta(&env, &id, &meta);
        storage::set_bounty_count(&env, &(count + 1));
        storage::add_bounty_to_status(&env, &id, &bounty.status);

        let mut open = storage::get_open_bounties(&env);
        open.push_back(id.clone());
        storage::set_open_bounties(&env, &open);

        events::emit_bounty_created(&env, &id, &bounty.creator, &reward_amount);
        id
    }

    pub fn claim_bounty(env: Env, contributor: Address, bounty_id: BytesN<32>) {
        contributor.require_auth();

        // Explicit pre-condition check: bounty must exist.
        let mut bounty = match storage::get_bounty(&env, &bounty_id) {
            Some(b) => b,
            None => panic!("{}", errors::BOUNTY_NOT_FOUND),
        };

        if bounty.assignee.is_some() {
            panic!("{}", errors::BOUNTY_ALREADY_ASSIGNED);
        }

        let mut contrib = storage::get_contributor(&env, &contributor)
            .unwrap_or(Contributor {
                address: contributor.clone(),
                reputation: 0,
                total_earned: 0,
                contribution_count: 0,
                active_claims: 0,
            });

        if contrib.active_claims >= 1 {
            panic!("{}", errors::CONTRIBUTOR_HAS_ACTIVE_CLAIM);
        }

        if bounty.min_reputation > 0 {
            let contributor_profile = storage::get_contributor(&env, &contributor).unwrap_or(Contributor {
                address: contributor.clone(),
                reputation: 0,
                total_earned: 0,
                contribution_count: 0,
            });
            if contributor_profile.reputation < bounty.min_reputation {
                panic!("contributor reputation is too low");
            }
        }

        bounty.assignee = Some(contributor.clone());
        bounty.status = Symbol::new(&env, STATUS_IN_PROGRESS);

        storage::store_bounty(&env, &bounty_id, &bounty);
        storage::move_bounty_status(&env, &bounty_id, &previous_status, &bounty.status);
        events::emit_bounty_claimed(&env, &bounty_id, &contributor);

        let mut open = storage::get_open_bounties(&env);
        if let Some(pos) = open.iter().position(|id| id == bounty_id) {
            open.remove(pos as u32);
        }
        storage::set_open_bounties(&env, &open);
    }

    pub fn complete_bounty(env: Env, verifier: Address, bounty_id: BytesN<32>) {
        verifier.require_auth();

        let mut bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");
        let assignee = bounty.assignee.clone().expect("bounty has no assignee");
        let mut contributor = storage::get_contributor(&env, &assignee).unwrap_or(Contributor {
            address: assignee.clone(),
            reputation: 0,
            total_earned: 0,
            contribution_count: 0,
        });

        // --- external call ---
        TokenClient::new(&env, &bounty.reward_token).transfer(
            &verifier,
            &assignee,
            &bounty.reward_amount,
        );

        // --- in-memory mutations ---
        contributor.reputation += 10;
        contributor.total_earned += bounty.reward_amount;
        contributor.contribution_count += 1;
        if contributor.active_claims > 0 {
            contributor.active_claims -= 1;
        }

        // --- writes ---
        storage::store_contributor(&env, &assignee, &contributor);

        bounty.status = Symbol::new(&env, STATUS_COMPLETED);
        storage::store_bounty(&env, &bounty_id, &bounty);

        events::emit_bounty_completed(&env, &bounty_id, &assignee);
        events::emit_reward_paid(&env, &bounty_id, &assignee, &bounty.reward_amount);
    }

    pub fn raise_dispute(env: Env, caller: Address, bounty_id: BytesN<32>) {
        caller.require_auth();

        let mut bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");
        let assignee = bounty.assignee.clone();

        if caller != bounty.creator && Some(caller.clone()) != assignee {
            panic!("only creator or assignee can raise dispute");
        }

        bounty.status = Symbol::new(&env, STATUS_DISPUTED);
        storage::store_bounty(&env, &bounty_id, &bounty);
        events::emit_bounty_disputed(&env, &bounty_id, &caller);
    }

    pub fn update_bounty(env: Env, creator: Address, bounty_id: BytesN<32>, title: Symbol, description: Symbol) {
        creator.require_auth();

        let mut bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");

        if bounty.creator != creator {
            panic!("only creator can update bounty");
        }

        bounty.title = title;
        bounty.description = description;

        storage::store_bounty(&env, &bounty_id, &bounty);
        events::emit_bounty_updated(&env, &bounty_id, &creator);
    }

    pub fn get_bounty(env: Env, bounty_id: BytesN<32>) -> Option<Bounty> {
        storage::get_bounty(&env, &bounty_id)
    }

    pub fn get_bounty_meta(env: Env, bounty_id: BytesN<32>) -> Option<BountyMeta> {
        storage::get_bounty_meta(&env, &bounty_id)
    }

    pub fn get_contributor(env: Env, address: Address) -> Option<Contributor> {
        storage::get_contributor(&env, &address)
    }

    pub fn get_bounty_count(env: Env) -> u64 {
        storage::get_bounty_count(&env)
    }

    pub fn get_bounties_by_status(env: Env, status: Symbol) -> soroban_sdk::Vec<BytesN<32>> {
        storage::get_bounties_by_status(&env, &status)
    }
}
