use soroban_sdk::{Address, BytesN, Env, Symbol};

pub fn emit_bounty_created(env: &Env, bounty_id: &BytesN<32>, creator: &Address, reward: &i128) {
    let topic = Symbol::new(env, "bounty_created");
    env.events().publish(
        (topic, bounty_id.clone()),
        (creator.clone(), *reward),
    );
}

pub fn emit_bounty_claimed(env: &Env, bounty_id: &BytesN<32>, contributor: &Address) {
    let topic = Symbol::new(env, "bounty_claimed");
    env.events().publish(
        (topic, bounty_id.clone()),
        contributor.clone(),
    );
}

pub fn emit_reward_paid(env: &Env, bounty_id: &BytesN<32>, contributor: &Address, amount: &i128) {
    let topic = Symbol::new(env, "reward_paid");
    env.events().publish(
        (topic, bounty_id.clone()),
        (contributor.clone(), *amount),
    );
}
