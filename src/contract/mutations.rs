const STATUS_OPEN: &str = "open";
const STATUS_IN_PROGRESS: &str = "in_progress";
const STATUS_COMPLETED: &str = "completed";
const STATUS_CANCELLED: &str = "cancelled";
const STATUS_DISPUTED: &str = "disputed";

fn generate_bounty_id(env: &Env, count: u64) -> BountyId {
    let mut buf = [0u8; 32];
    let count_bytes = count.to_be_bytes();
    buf[24..32].copy_from_slice(&count_bytes);
    BountyId(BytesN::from_array(env, &buf))
}

#[contractimpl]
impl MergeMintContract {
    /// Create a new bounty.
    ///
    /// The caller must be the `creator` (enforced by `require_auth`).
    /// The bounty is initialised with `"open"` status and `max_assignees = 1`.
    ///
    /// # Arguments
    /// * `creator` - Wallet that will own and manage this bounty.
    /// * `title` - Short human-readable title (max 32 chars via `Symbol`).
    /// * `description` - Longer description of the work required.
    /// * `reward_amount` - Raw token units for the reward. Must be positive.
    /// * `reward_token` - Soroban token contract address used for payout.
    /// * `min_reputation` - Minimum reputation score required to claim (0 = no minimum).
    /// * `deadline` - Optional ledger sequence deadline after which the bounty cannot be claimed.
    /// * `tags` - Categorisation tags (e.g. "bug", "docs"). At most 5 tags allowed.
    ///
    /// # Returns
    /// The newly generated `BountyId` that uniquely identifies this bounty.
    ///
    /// # Panics
    /// * If `reward_amount` is not strictly positive.
    /// * If `tags.len() > 5` (`ContractError::TooManyTags`).
    ///
    /// # Authorization
    /// Requires auth from `creator`.
    pub fn create_bounty(
        env: Env,
        creator: Address,
        title: Symbol,
        description: Symbol,
        reward_amount: i128,
        reward_token: Address,
        min_reputation: u32,
        deadline: Option<u32>,
        tags: Vec<Symbol>,
    ) -> BountyId {
        // Validated first, ahead of auth and all storage interaction: a non-positive
        // reward is a malformed request regardless of who is asking.
        if reward_amount <= 0 {
            fail(ContractError::RewardMustBePositive);
        }

        // Validate tags length before auth to fail fast on malformed input.
        if tags.len() > 5 {
            fail(ContractError::TooManyTags);
        }

        creator.require_auth();

        let count = storage::get_bounty_count(&env);
        let id = generate_bounty_id(&env, count);

        let bounty = Bounty {
            creator: creator.clone(),
            reward_amount,
            reward_token,
            assignees: Vec::new(&env),
            max_assignees: 1,
            status: Symbol::new(&env, STATUS_OPEN),
            min_reputation,
            deadline,
            required_verifiers: None,
            approval_threshold: 1,
            tags,
        };

        storage::store_bounty(&env, &id, &bounty);
        storage::store_bounty_meta(&env, &id, &BountyMeta { title, description });
        storage::set_bounty_count(&env, &(count + 1));
        storage::add_bounty_to_status(&env, &id, &bounty.status);
        storage::append_creator_bounty(&env, &creator, &id);

        let mut open = storage::get_open_bounties(&env);
        open.push_back(id.clone());
        storage::set_open_bounties(&env, &open);

        events::emit_bounty_created(&env, &id, &creator, &reward_amount);
        id
    }

    /// Claim an open bounty.
    ///
    /// A contributor receives 10 000 basis points (full reward) when claiming a
    /// single-assignee bounty (`max_assignees == 1`). The contributor is added to
    /// the bounty's `assignees` list and the status transitions to `"in_progress"`.
    ///
    /// # Arguments
    /// * `contributor` - Wallet claiming the bounty.
    /// * `bounty_id` - The bounty to claim.
    ///
    /// # Panics
    /// * If `bounty_id` does not exist.
    /// * If the bounty is already at `max_assignees` capacity.
    /// * If the contributor is already an assignee on this bounty.
    /// * If the contributor already has an active claim on another bounty.
    /// * If the bounty deadline has passed (`env.ledger().sequence() > deadline`).
    /// * If the contributor's reputation is below `min_reputation`.
    ///
    /// # Authorization
    /// Requires auth from `contributor`.
    pub fn claim_bounty(env: Env, contributor: Address, bounty_id: BountyId) {
        contributor.require_auth();

        let mut bounty = match storage::get_bounty(&env, &bounty_id) {
            Some(b) => b,
            None => fail(ContractError::BountyNotFound),
        };

        if bounty.assignees.len() >= bounty.max_assignees {
            fail(ContractError::BountyAlreadyAssigned);
        }

        for (addr, _) in bounty.assignees.iter() {
            if addr == contributor {
                fail(ContractError::BountyAlreadyAssigned);
            }
        }

        // #275: use Contributor::new for default construction
        let mut contrib = storage::get_contributor(&env, &contributor)
            .unwrap_or_else(|| Contributor::new(contributor.clone()));

        if contrib.active_claims >= 1 {
            fail(ContractError::ContributorHasActiveClaim);
        }

        // Deadline enforcement: reject claims once the deadline ledger sequence has passed.
        if let Some(deadline) = bounty.deadline {
            if env.ledger().sequence() > deadline {
                fail(ContractError::BountyDeadlinePassed);
            }
        }

        if bounty.min_reputation > 0 && contrib.reputation < bounty.min_reputation {
            panic!("contributor reputation is too low");
        }

        // For single-assignee bounties the sole claimant gets 10 000 basis points (100%).
        let share_bp: u32 = 10_000;
        bounty.assignees.push_back((contributor.clone(), share_bp));

        let previous_status = bounty.status.clone();
        bounty.status = Symbol::new(&env, STATUS_IN_PROGRESS);
        storage::store_bounty(&env, &bounty_id, &bounty);
        storage::move_bounty_status(&env, &bounty_id, &previous_status, &bounty.status);

        contrib.active_claims += 1;
        storage::store_contributor(&env, &contributor, &contrib);

        // Remove from open bounties list.
        let open = storage::get_open_bounties(&env);
        let mut new_open = Vec::new(&env);
        for existing_id in open.iter() {
            if existing_id != bounty_id {
                new_open.push_back(existing_id);
            }
        }
        storage::set_open_bounties(&env, &new_open);

        events::emit_bounty_claimed(&env, &bounty_id, &contributor);
    }

    /// Complete a bounty and distribute the reward.
    ///
    /// Transfers `reward_amount` from `verifier` to each assignee proportionally
    /// according to their basis-point share. Each assignee's reputation increases
    /// by 10 and their `active_claims` is decremented. The bounty status transitions
    /// to `"completed"`.
    ///
    /// # Arguments
    /// * `verifier` - Wallet that holds the tokens and initiates the payout.
    /// * `bounty_id` - The bounty to complete.
    ///
    /// # Panics
    /// * If `bounty_id` does not exist.
    /// * If the bounty status is not `"in_progress"` (prevents double-completion).
    /// * If the bounty has no assignees.
    /// * If `verifier` is one of the bounty assignees (prevents self-verification).
    /// * If the token transfer fails (insufficient balance, no allowance, etc.).
    ///
    /// # Authorization
    /// `verifier.require_auth()` is the **first** operation in this function.
    /// No storage reads or business logic execute before authentication is checked.
    pub fn complete_bounty(env: Env, verifier: Address, bounty_id: BountyId) {
        // AUTH: must be first — no storage reads or side-effects before this line.
        verifier.require_auth();

        let mut bounty = match storage::get_bounty(&env, &bounty_id) {
            Some(b) => b,
            None => fail(ContractError::BountyNotFound),
        };

        // GUARD 1 — dispute prevention.
        // If the bounty is in disputed status, complete_bounty must not execute.
        // A disputed bounty must be resolved via resolve_dispute first.
        if bounty.status == Symbol::new(&env, STATUS_DISPUTED) {
            fail(ContractError::BountyIsDisputed);
        }

        // GUARD 2 — double-completion prevention.
        // Reject the call if the bounty is not currently in progress. This blocks
        // repeat calls on already-completed bounties and any other terminal state.
        // Depends on claim_bounty having written STATUS_IN_PROGRESS and
        // complete_bounty writing STATUS_COMPLETED below (checks-effects-interactions).
        if bounty.status != Symbol::new(&env, STATUS_IN_PROGRESS) {
            panic!("{}", errors::BOUNTY_NOT_IN_PROGRESS);
        }

        if bounty.assignees.is_empty() {
            fail(ContractError::BountyHasNoAssignee);
        }

        // GUARD 3 — self-verification prevention.
        // The verifier must be a party independent from the assignees. Allowing the
        // same address to both claim and verify would let a contributor manufacture
        // reputation and, once escrow is introduced, drain contract funds unilaterally.
        for (assignee, _) in bounty.assignees.iter() {
            if assignee == verifier {
                panic!("verifier cannot be the assignee");
            }
        }

        // Checks-effects-interactions pattern:
        // 1. Compute all payouts and update contributor state in memory.
        // 2. Persist the status change (marking the bounty completed) BEFORE any
        //    cross-contract token transfer. This ensures that a reentrant call back
        //    into complete_bounty would be rejected by GUARD 2 above.
        // 3. Execute token transfers last.
        let token = TokenClient::new(&env, &bounty.reward_token);
        let mut payouts: Vec<(Address, i128)> = Vec::new(&env);

        for (assignee, share_bp) in bounty.assignees.iter() {
            let payout = bounty.reward_amount * (share_bp as i128) / 10_000_i128;
            payouts.push_back((assignee.clone(), payout));

            let mut contrib = storage::get_contributor(&env, &assignee)
                .unwrap_or_else(|| Contributor::new(assignee.clone()));

            contrib.reputation += 10;
            contrib.total_earned += payout;
            contrib.contribution_count += 1;
            if contrib.active_claims > 0 {
                contrib.active_claims -= 1;
            }

            // Persist the updated contributor profile before the token transfer.
            storage::store_contributor(&env, &assignee, &contrib);
        }

        // Persist status transition before any cross-contract call.
        let (primary_assignee, _) = bounty.assignees.get(0).unwrap();
        let previous_status = bounty.status.clone();
        bounty.status = Symbol::new(&env, STATUS_COMPLETED);
        storage::store_bounty(&env, &bounty_id, &bounty);
        storage::move_bounty_status(&env, &bounty_id, &previous_status, &bounty.status);

        // Now safe to execute token transfers — bounty is already marked completed.
        for (assignee, payout) in payouts.iter() {
            token.transfer(&verifier, &assignee, &payout);
            events::emit_reward_paid(&env, &bounty_id, &assignee, &payout);
        }

        events::emit_bounty_completed(&env, &bounty_id, &primary_assignee);
    }

    /// Record one verifier's approval for a multi-sig bounty completion.
    /// When the number of unique approvals reaches approval_threshold, completion executes automatically.
    /// Falls back to single-verifier behaviour when required_verifiers is None (any verifier completes directly).
    ///
    /// # Authorization
    /// `verifier.require_auth()` is the **first** operation in this function.
    /// No storage reads or business logic execute before authentication is checked.
    pub fn approve_completion(env: Env, verifier: Address, bounty_id: BountyId) {
        verifier.require_auth();

        let mut bounty = match storage::get_bounty(&env, &bounty_id) {
            Some(b) => b,
            None => panic!("{}", errors::BOUNTY_NOT_FOUND),
        };

        if bounty.assignees.is_empty() {
            panic!("{}", errors::BOUNTY_HAS_NO_ASSIGNEE);
        }

        // If no required_verifiers list is set, fall back to immediate single-verifier completion.
        if bounty.required_verifiers.is_none() {
            MergeMintContract::complete_bounty(env, verifier, bounty_id);
            return;
        }

        let required = bounty.required_verifiers.clone().unwrap();
        let is_authorized = required.iter().any(|v| v == verifier);
        if !is_authorized {
            panic!("{}", errors::VERIFIER_NOT_AUTHORIZED);
        }

        let mut approvals = storage::get_approvals(&env, &bounty_id);

        // Guard against duplicate votes from the same verifier.
        let already_voted = approvals.iter().any(|v| v == verifier);
        if already_voted {
            panic!("{}", errors::ALREADY_APPROVED);
        }

        approvals.push_back(verifier.clone());
        storage::set_approvals(&env, &bounty_id, &approvals);

        let approval_count = approvals.len();
        events::emit_approval_recorded(&env, &bounty_id, &verifier, approval_count);

        let threshold = if bounty.approval_threshold == 0 { 1 } else { bounty.approval_threshold };

        if approval_count >= threshold {
            let token = TokenClient::new(&env, &bounty.reward_token);

            for (assignee, share_bp) in bounty.assignees.iter() {
                let payout = (bounty.reward_amount as i128) * (share_bp as i128) / 10_000_i128;
                token.transfer(&verifier, &assignee, &payout);

                let mut contrib = storage::get_contributor(&env, &assignee)
                    .unwrap_or(Contributor {
                        address: assignee.clone(),
                        reputation: 0,
                        total_earned: 0,
                        contribution_count: 0,
                        active_claims: 0,
                        metadata: None,
                    });

                contrib.reputation += 10;
                contrib.total_earned += payout;
                contrib.contribution_count += 1;
                if contrib.active_claims > 0 { contrib.active_claims -= 1; }

                storage::store_contributor(&env, &assignee, &contrib);
                events::emit_reward_paid(&env, &bounty_id, &assignee, &payout);
            }

            let (primary_assignee, _) = bounty.assignees.get(0).unwrap();
            let previous_status = bounty.status.clone();
            bounty.status = Symbol::new(&env, STATUS_COMPLETED);
            storage::store_bounty(&env, &bounty_id, &bounty);
            storage::move_bounty_status(&env, &bounty_id, &previous_status, &bounty.status);
            events::emit_bounty_completed(&env, &bounty_id, &primary_assignee);
        }
    }

    /// Raise a dispute on a bounty.
    ///
    /// Only the bounty creator or an existing assignee may call this.
    /// Transitions the bounty status to `"disputed"`.
    ///
    /// # Authorization
    /// `caller.require_auth()` is the **first** operation in this function.
    /// No storage reads or business logic execute before authentication is checked.
    pub fn raise_dispute(env: Env, caller: Address, bounty_id: BountyId) {
        caller.require_auth();

        let mut bounty = match storage::get_bounty(&env, &bounty_id) {
            Some(b) => b,
            None => fail(ContractError::BountyNotFound),
        };

        let is_assignee = bounty.assignees.iter().any(|(addr, _)| addr == caller);
        if caller != bounty.creator && !is_assignee {
            fail(ContractError::OnlyCreatorOrAssigneeCanDispute);
        }

        let previous_status = bounty.status.clone();
        bounty.status = Symbol::new(&env, STATUS_DISPUTED);
        storage::store_bounty(&env, &bounty_id, &bounty);
        storage::move_bounty_status(&env, &bounty_id, &previous_status, &bounty.status);
        events::emit_bounty_disputed(&env, &bounty_id, &caller);
    }

    /// Resolve a disputed bounty. Only the bounty creator (acting as arbitrator) may call this.
    /// resolution must be the Symbol "complete" (pay assignees) or "cancel" (refund creator).
    ///
    /// # Authorization
    /// `arbitrator.require_auth()` is the **first** operation in this function.
    /// No storage reads or business logic execute before authentication is checked.
    pub fn resolve_dispute(
        env: Env,
        arbitrator: Address,
        bounty_id: BountyId,
        resolution: Symbol,
    ) {
        arbitrator.require_auth();

        let mut bounty = match storage::get_bounty(&env, &bounty_id) {
            Some(b) => b,
            None => panic!("{}", errors::BOUNTY_NOT_FOUND),
        };

        if bounty.status != Symbol::new(&env, STATUS_DISPUTED) {
            panic!("{}", errors::BOUNTY_NOT_DISPUTED);
        }

        // The arbitrator must be the bounty creator; there is no separate admin address.
        if arbitrator != bounty.creator {
            panic!("{}", errors::NOT_ARBITRATOR);
        }

        let resolve_complete = Symbol::new(&env, "complete");
        let resolve_cancel = Symbol::new(&env, "cancel");

        if resolution == resolve_complete {
            if bounty.assignees.is_empty() {
                fail(ContractError::BountyHasNoAssignee);
            }

            let token = TokenClient::new(&env, &bounty.reward_token);

            for (assignee, share_bp) in bounty.assignees.iter() {
                let payout =
                    (bounty.reward_amount as i128) * (share_bp as i128) / 10_000_i128;
                token.transfer(&env.current_contract_address(), &assignee, &payout);

                let mut contrib = storage::get_contributor(&env, &assignee)
                    .unwrap_or(Contributor {
                        address: assignee.clone(),
                        reputation: 0,
                        total_earned: 0,
                        contribution_count: 0,
                        active_claims: 0,
                        metadata: None,
                    });

                contrib.reputation += 10;
                contrib.total_earned += payout;
                contrib.contribution_count += 1;
                if contrib.active_claims > 0 {
                    contrib.active_claims -= 1;
                }

                storage::store_contributor(&env, &assignee, &contrib);
                events::emit_reward_paid(&env, &bounty_id, &assignee, &payout);
            }

            let previous_status = bounty.status.clone();
            bounty.status = Symbol::new(&env, STATUS_COMPLETED);
            storage::store_bounty(&env, &bounty_id, &bounty);
            storage::move_bounty_status(&env, &bounty_id, &previous_status, &bounty.status);
        } else if resolution == resolve_cancel {
            // Escrow refund to creator goes here once escrow is implemented.
            let previous_status = bounty.status.clone();
            bounty.status = Symbol::new(&env, STATUS_CANCELLED);
            storage::store_bounty(&env, &bounty_id, &bounty);
            storage::move_bounty_status(&env, &bounty_id, &previous_status, &bounty.status);
        } else {
            panic!("resolution must be 'complete' or 'cancel'");
        }

        events::emit_dispute_resolved(&env, &bounty_id, &arbitrator, &resolution);
    }

    /// Update the on-chain metadata URI for a contributor profile.
    ///
    /// The `metadata` value is typically an IPFS hash or URL pointing to a JSON
    /// document containing the contributor's name, avatar, GitHub username, etc.
    ///
    /// # Arguments
    /// * `contributor` - Wallet whose metadata is being updated.
    /// * `metadata` - New metadata URI (`Symbol`) to store.
    ///
    /// # Authorization
    /// Requires auth from `contributor`. No other address may modify this field.
    pub fn update_contributor_metadata(env: Env, contributor: Address, metadata: Symbol) {
        contributor.require_auth();

        // #275: use Contributor::new for default construction
        let mut contrib = storage::get_contributor(&env, &contributor)
            .unwrap_or_else(|| Contributor::new(contributor.clone()));

        contrib.metadata = Some(metadata);
        storage::store_contributor(&env, &contributor, &contrib);
    }

    /// Cancel an open bounty.
    ///
    /// Only the creator may cancel. The bounty must be in `"open"` status.
    /// Transitions the bounty status to `"cancelled"`.
    ///
    /// # Arguments
    /// * `caller` - Wallet requesting cancellation (must be the creator).
    /// * `bounty_id` - The bounty to cancel.
    ///
    /// # Panics
    /// * If `bounty_id` does not exist.
    /// * If `caller` is not the bounty creator.
    /// * If the bounty is not in `"open"` status.
    ///
    /// # Authorization
    /// Requires auth from `caller`. Only the creator may cancel.
    pub fn cancel_bounty(env: Env, caller: Address, bounty_id: BountyId) {
        caller.require_auth();

        let mut bounty = match storage::get_bounty(&env, &bounty_id) {
            Some(b) => b,
            None => fail(ContractError::BountyNotFound),
        };

        if caller != bounty.creator {
            fail(ContractError::NotBountyCreator);
        }

        if bounty.status != Symbol::new(&env, STATUS_OPEN) {
            fail(ContractError::BountyNotOpen);
        }

        let previous_status = bounty.status.clone();
        bounty.status = Symbol::new(&env, STATUS_CANCELLED);
        storage::store_bounty(&env, &bounty_id, &bounty);
        storage::move_bounty_status(&env, &bounty_id, &previous_status, &bounty.status);

        events::emit_bounty_cancelled(&env, &bounty_id, &caller);
    }

    /// Expire an open bounty whose deadline has passed.
    ///
    /// Permissionless: any caller may trigger expiry to keep the open-bounty list
    /// clean. The bounty must have a deadline set and the current ledger sequence
    /// must exceed that deadline. Transitions to `"cancelled"`.
    ///
    /// # Arguments
    /// * `caller` - Wallet triggering the expiry (any authenticated address).
    /// * `bounty_id` - The bounty to expire.
    ///
    /// # Panics
    /// * If `bounty_id` does not exist.
    /// * If the bounty has no deadline set.
    /// * If the deadline has not yet passed.
    /// * If the bounty is not in `"open"` status.
    ///
    /// # Authorization
    /// Requires auth from `caller` (any authenticated wallet may trigger expiry).
    pub fn expire_bounty(env: Env, caller: Address, bounty_id: BountyId) {
        caller.require_auth();

        let mut bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");

        let deadline = match bounty.deadline {
            Some(d) => d,
            None => panic!("{}", errors::BOUNTY_NO_DEADLINE),
        };

        if env.ledger().sequence() <= deadline {
            fail(ContractError::DeadlineNotPassed);
        }

        if bounty.status != Symbol::new(&env, STATUS_OPEN) {
            fail(ContractError::BountyNotOpen);
        }

        let previous_status = bounty.status.clone();
        bounty.status = Symbol::new(&env, STATUS_CANCELLED);
        storage::store_bounty(&env, &bounty_id, &bounty);
        storage::move_bounty_status(&env, &bounty_id, &previous_status, &bounty.status);

        // Escrow refund goes here once escrow is implemented.
        events::emit_bounty_expired(&env, &bounty_id, &bounty.creator);
    }
}
