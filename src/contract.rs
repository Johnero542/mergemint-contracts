use soroban_sdk::{
    contract, contractimpl, token::TokenClient, Address, BytesN, Env, Symbol,
};

use crate::events;
use crate::storage;
use crate::types::{Bounty, Contributor};

const STATUS_OPEN: &str = "open";
const STATUS_IN_PROGRESS: &str = "in_progress";
const STATUS_COMPLETED: &str = "completed";

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

        let mut bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");

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

        let bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");
        let assignee = bounty.assignee.clone().expect("bounty has no assignee");

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

        let mut completed_bounty = storage::get_bounty(&env, &bounty_id).unwrap();
        completed_bounty.status = Symbol::new(&env, STATUS_COMPLETED);
        storage::store_bounty(&env, &bounty_id, &completed_bounty);

        events::emit_bounty_completed(&env, &bounty_id, &assignee);
        events::emit_reward_paid(&env, &bounty_id, &assignee, &bounty.reward_amount);
    }

    pub fn update_bounty(
        env: Env,
        creator: Address,
        bounty_id: BytesN<32>,
        new_reward_amount: i128,
    ) {
        creator.require_auth();

        let mut bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");

        if bounty.creator != creator {
            panic!("only the creator can update a bounty");
        }

        let open = Symbol::new(&env, STATUS_OPEN);
        let in_progress = Symbol::new(&env, STATUS_IN_PROGRESS);
        let completed = Symbol::new(&env, STATUS_COMPLETED);

        if bounty.status == in_progress {
            panic!("cannot update a bounty that is in progress");
        }
        if bounty.status == completed {
            panic!("cannot update a completed bounty");
        }
        if bounty.status != open {
            panic!("bounty is not open");
        }

        bounty.reward_amount = new_reward_amount;
        storage::store_bounty(&env, &bounty_id, &bounty);
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
