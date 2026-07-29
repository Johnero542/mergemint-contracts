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

    pub fn get_bounty_metas(env: Env, ids: Vec<BountyId>) -> Vec<Option<BountyMeta>> {
        let mut result: Vec<Option<BountyMeta>> = Vec::new(&env);
        for id in ids.iter() {
            result.push_back(storage::get_bounty_meta(&env, &id));
        }
        result
    }

    pub fn get_contributor(env: Env, address: Address) -> Option<Contributor> {
        storage::get_contributor(&env, &address)
    }

    /// Return the total number of bounties ever created (monotonic counter).
    ///
    /// This is the primary metric exposed by `GET /api/bounties/count` in the
    /// REST layer. It reads a single persistent u64 counter and is cheap to
    /// call even when thousands of bounties exist.
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

    pub fn get_status_count(env: Env, status: Symbol) -> u32 {
        storage::get_status_count(&env, &status)
    }

    pub fn get_open_bounties(env: Env) -> Vec<BountyId> {
        storage::get_open_bounties(&env)
    }

    /// Return a page of open bounty IDs — supports `GET /api/bounties?offset=&limit=`.
    ///
    /// `offset` is zero-based; `limit` is capped at 50 to bound ledger CPU cost.
    /// Returns an empty vec when `offset` is beyond the end of the list.
    ///
    /// Example: `get_open_bounties_paged(env, 0, 20)` → first 20 open bounties.
    pub fn get_open_bounties_paged(env: Env, offset: u32, limit: u32) -> Vec<BountyId> {
        let all = storage::get_open_bounties(&env);
        let cap: u32 = 50;
        let effective_limit = if limit > cap { cap } else { limit };
        let len = all.len();
        let mut result = Vec::new(&env);
        if offset >= len {
            return result;
        }
        let end = {
            let e = offset + effective_limit;
            if e > len { len } else { e }
        };
        let mut i = offset;
        while i < end {
            result.push_back(all.get(i).unwrap());
            i += 1;
        }
        result
    }

    /// Return all open bounty IDs that carry the requested tag.
    ///
    /// Supports `GET /api/bounties?tag=<tag>`. Iterates the open-bounties index
    /// and looks up each bounty to check `bounty.tags`; callers can page the
    /// result with `get_open_bounties_paged` first and apply filtering client-side
    /// for large lists.
    pub fn get_bounties_by_tag(env: Env, tag: Symbol) -> Vec<BountyId> {
        let open_ids = storage::get_open_bounties(&env);
        let mut result = Vec::new(&env);
        for id in open_ids.iter() {
            if let Some(bounty) = storage::get_bounty(&env, &id) {
                for t in bounty.tags.iter() {
                    if t == tag {
                        result.push_back(id.clone());
                        break;
                    }
                }
            }
        }
        result
    }

    /// Return the bounty ID of the in-progress bounty assigned to `address`, if any.
    ///
    /// Supports `GET /api/contributors/{address}/active-bounty`. Scans open and
    /// in-progress bounties for an assignee slot matching `address`. Returns `None`
    /// when the contributor has no active claim.
    pub fn get_contributor_active_bounty(env: Env, address: Address) -> Option<BountyId> {
        let in_progress_sym = Symbol::new(&env, "in_progress");
        let in_progress_ids = storage::get_bounties_by_status(&env, &in_progress_sym);
        for id in in_progress_ids.iter() {
            if let Some(bounty) = storage::get_bounty(&env, &id) {
                for (assignee, _weight) in bounty.assignees.iter() {
                    if assignee == address {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    /// Return all bounty IDs created by a specific creator address.
    ///
    /// The list is maintained in `DataKey::ContributorBounties(creator)` and
    /// appended to on each `create_bounty` call. Returns an empty `Vec` if the
    /// address has never created a bounty.
    pub fn get_bounties_by_creator(env: Env, creator: Address) -> Vec<BountyId> {
        storage::get_creator_bounties(&env, &creator)
    }
}
