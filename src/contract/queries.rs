// Queries are included directly into mod.rs via include!(), so all imports
// from mod.rs are already in scope — no use statements needed here.

#[contractimpl]
impl MergeMintContract {
    pub fn get_bounty(env: Env, bounty_id: BountyId) -> Option<Bounty> {
        storage::get_bounty(&env, &bounty_id)
    }

    pub fn get_bounty_meta(env: Env, bounty_id: BountyId) -> Option<BountyMeta> {
        storage::get_bounty_meta(&env, &bounty_id)
    }

    pub fn get_contributor(env: Env, address: Address) -> Option<Contributor> {
        storage::get_contributor(&env, &address)
    }

    pub fn get_bounty_count(env: Env) -> u64 {
        storage::get_bounty_count(&env)
    }

    pub fn get_bounties(env: Env, ids: Vec<BountyId>) -> Vec<Option<Bounty>> {
        let mut result = Vec::new(&env);
        for id in ids.iter() {
            result.push_back(storage::get_bounty(&env, &id));
        }
        result
    }

    pub fn get_bounties_by_status(env: Env, status: Symbol) -> Vec<BountyId> {
        storage::get_bounties_by_status(&env, &status)
    }

    pub fn get_open_bounties(env: Env) -> Vec<BountyId> {
        storage::get_open_bounties(&env)
    }

    /// Return all bounty IDs created by a specific creator address.
    ///
    /// The list is maintained in `DataKey::ContributorBounties(creator)` and
    /// appended to on each `create_bounty` call. Returns an empty `Vec` if the
    /// address has never created a bounty.
    pub fn get_bounties_by_creator(env: Env, creator: Address) -> Vec<BountyId> {
        storage::get_creator_bounties(&env, &creator)
    }

    /// Return the number of bounties currently in the given status.
    ///
    /// This is a lightweight alternative to calling `get_bounties_by_status`
    /// and measuring the length of the returned `Vec`. It reads a single
    /// `u32` counter instead of the full status-index list, making it
    /// suitable for dashboard badges, status-summary endpoints, and other
    /// places where only the numeric count is needed.
    ///
    /// The counter is maintained automatically by `add_bounty_to_status`
    /// and `remove_bounty_from_status` in the storage layer, so it always
    /// reflects the current number of bounties in the requested status.
    pub fn get_status_count(env: Env, status: Symbol) -> u32 {
        storage::get_status_count(&env, &status)
    }
}
