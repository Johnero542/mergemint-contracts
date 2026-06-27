# Security Model

## Current model (no escrow)

In the current design the verifier holds tokens off-chain and calls `complete_bounty` to push
them to the assignee via `token.transfer`. The contract itself never holds a token balance.
The attack surface is limited to:

- **Authentication bypass** — every state-changing function calls `require_auth()` on the
  relevant actor (creator, contributor, or verifier).
- **Double-assignment** — `claim_bounty` panics if `assignee` is already `Some`.
- **Monotonic reputation** — reputation only ever increases; no underflow path exists.

---

## Escrow threat model

When escrow is introduced the contract will hold tokens on behalf of bounty creators
(`create_bounty` transfers reward tokens *into* the contract; `complete_bounty` and
`cancel_bounty` transfer them *out*). This changes the threat surface significantly.

### Token balance invariant

> **The contract's token balance for any given token must always equal the sum of
> `reward_amount` across all bounties in `open` or `in_progress` status that use that token.**

Maintaining this invariant is the primary correctness goal for all escrow-related code paths.
Any deviation — even transient — represents a fund safety bug.

### Attack vectors

#### 1. Stuck funds (locked tokens)

**Description:** A bug prevents a bounty from ever reaching `completed` or `cancelled`, locking
the escrowed tokens permanently.

**Example scenarios:**
- `cancel_bounty` panics unconditionally due to a logic error.
- Status index corruption leaves a bounty in an unresolvable state.
- A missing code path for a status transition leaves a bounty stuck.

**Mitigations:**
- Ensure `cancel_bounty` is callable by the creator for any bounty in `open` or `in_progress`
  status — it must always provide an exit.
- Consider a verifier-only emergency cancel path as a backstop.
- Write invariant-checking tests that verify the contract balance equals the sum of open
  bounty rewards after every state transition.

#### 2. Fund drain (double-completion or reentrancy)

**Description:** An attacker triggers multiple payouts for a single bounty, draining more tokens
than the bounty's `reward_amount`.

**Example scenarios:**
- `complete_bounty` is called twice before the status is persisted as `completed`, paying the
  reward twice (a classic check-effects-interactions violation).
- Reentrancy via a malicious token contract that calls back into `complete_bounty` during
  `token.transfer`.

**Mitigations:**
- Follow checks-effects-interactions strictly: update the bounty status to `completed` and
  persist it *before* calling `token.transfer`.
- Validate that `bounty.status == STATUS_IN_PROGRESS` at the top of `complete_bounty` and
  panic otherwise — this prevents double-completion even without reentrancy guards.
- Note: Soroban's single-contract-at-a-time execution model eliminates classic reentrancy, but
  the ordering discipline should still be enforced for clarity and defence-in-depth.

#### 3. Griefing (gas/fee exhaustion)

**Description:** A malicious actor creates many bounties (locking the minimum viable reward in
each) and immediately cancels them, burning transaction fees and polluting the status index.

**Example scenarios:**
- Rapid create → cancel cycles pad the `cancelled` index, increasing read costs for
  `get_bounties_by_status("cancelled")`.
- Large numbers of `open` bounties with tiny rewards deter legitimate contributors.

**Mitigations:**
- Enforce a minimum `reward_amount` in `create_bounty` to raise the economic cost of griefing.
- Consider a creation fee (paid to the contract or burned) that is separate from the bounty
  reward, making spam attacks self-limiting.
- The status index is unbounded today; if griefing is a concern, cap index length or paginate
  reads.

### High-risk code paths

| Function          | Risk                                      | Key invariant to enforce                              |
|-------------------|-------------------------------------------|-------------------------------------------------------|
| `create_bounty`   | Token transfer in; under-transfer         | `contract_balance += reward_amount` after the call    |
| `complete_bounty` | Token transfer out; double-payment        | Status set to `completed` before `token.transfer`     |
| `cancel_bounty`   | Token transfer out; stuck-fund if blocked | Always reachable by creator; status set before transfer |

### Recommended pre-merge checklist for escrow

- [ ] Fuzz `reward_amount` edge cases (0, `i128::MAX`, negative).
- [ ] Add an integration test that asserts `contract_balance == sum(open + in_progress rewards)`
      after each transition.
- [ ] Confirm `complete_bounty` panics when called on an already-completed bounty.
- [ ] Review token contract for any re-entrant callbacks into this contract.
- [ ] Have at least one contributor who was not the author review the token transfer ordering.
