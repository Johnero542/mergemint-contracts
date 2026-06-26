use soroban_sdk::{
    contract, contractimpl, token::TokenClient, Address, BytesN, Env, Symbol,
};

use crate::events;
use crate::storage;
use crate::types::{Bounty, Contributor};

const STATUS_OPEN: &str = "open";
const STATUS_IN_PROGRESS: &str = "in_progress";
const STATUS_CANCELLED: &str = "cancelled";

// Error message constants — used in explicit pre-condition checks below.
mod errors {
    pub const BOUNTY_NOT_FOUND: &str = "bounty not found";
    pub const BOUNTY_HAS_NO_ASSIGNEE: &str = "bounty has no assignee";
    pub const NOT_BOUNTY_CREATOR: &str = "not the bounty creator";
    pub const BOUNTY_NOT_OPEN: &str = "bounty is not open";
    pub const BOUNTY_NO_DEADLINE: &str = "bounty has no deadline";
    pub const DEADLINE_NOT_PASSED: &str = "bounty deadline has not passed";
}

fn generate_bounty_id(env: &Env) -> BytesN<32> {
    let count = storage::get_bounty_count(env);
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
    ) -> BytesN<32> {
        creator.require_auth();

        let count = storage::get_bounty_count(&env);
        let id = generate_bounty_id(&env);

        let bounty = Bounty {
            creator,
            title,
            description,
            reward_amount,
            reward_token,
            assignee: None,
            status: Symbol::new(&env, STATUS_OPEN),
        };

        storage::store_bounty(&env, &id, &bounty);
        storage::set_bounty_count(&env, &(count + 1));

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
            panic!("bounty already assigned");
        }

        bounty.assignee = Some(contributor.clone());
        bounty.status = Symbol::new(&env, STATUS_IN_PROGRESS);

        storage::store_bounty(&env, &bounty_id, &bounty);
        events::emit_bounty_claimed(&env, &bounty_id, &contributor);
    }

    pub fn complete_bounty(env: Env, verifier: Address, bounty_id: BytesN<32>) {
        verifier.require_auth();

        // Explicit pre-condition checks: bounty and assignee must exist.
        let bounty = match storage::get_bounty(&env, &bounty_id) {
            Some(b) => b,
            None => panic!("{}", errors::BOUNTY_NOT_FOUND),
        };
        let assignee = match bounty.assignee.clone() {
            Some(a) => a,
            None => panic!("{}", errors::BOUNTY_HAS_NO_ASSIGNEE),
        };

        let token = TokenClient::new(&env, &bounty.reward_token);
        token.transfer(&verifier, &assignee, &bounty.reward_amount);

        let mut contributor = storage::get_contributor(&env, &assignee).unwrap_or(Contributor {
            address: assignee.clone(),
            reputation: 0,
            total_earned: 0,
            contribution_count: 0,
        });

        contributor.reputation += 10;
        contributor.total_earned += bounty.reward_amount;
        contributor.contribution_count += 1;

        storage::store_contributor(&env, &assignee, &contributor);

        events::emit_bounty_completed(&env, &bounty_id, &assignee);
        events::emit_reward_paid(&env, &bounty_id, &assignee, &bounty.reward_amount);
    }

    /// Cancel an open bounty. Only the bounty creator is authorised.
    /// Once escrow is implemented this will trigger a refund of escrowed tokens.
    pub fn cancel_bounty(env: Env, caller: Address, bounty_id: BytesN<32>) {
        caller.require_auth();

        let mut bounty = match storage::get_bounty(&env, &bounty_id) {
            Some(b) => b,
            None => panic!("{}", errors::BOUNTY_NOT_FOUND),
        };

        // Auth guard: only the bounty creator may cancel.
        if bounty.creator != caller {
            panic!("{}", errors::NOT_BOUNTY_CREATOR);
        }

        if bounty.status != Symbol::new(&env, STATUS_OPEN) {
            panic!("{}", errors::BOUNTY_NOT_OPEN);
        }

        bounty.status = Symbol::new(&env, STATUS_CANCELLED);
        storage::store_bounty(&env, &bounty_id, &bounty);

        events::emit_bounty_cancelled(&env, &bounty_id, &caller);
    }

    pub fn get_bounty(env: Env, bounty_id: BytesN<32>) -> Option<Bounty> {
        storage::get_bounty(&env, &bounty_id)
    }

    pub fn get_contributor(env: Env, address: Address) -> Option<Contributor> {
        storage::get_contributor(&env, &address)
    }

    pub fn get_bounty_count(env: Env) -> u64 {
        storage::get_bounty_count(&env)
    }
}
