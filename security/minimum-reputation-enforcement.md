# Security: Enforce Minimum Reputation Threshold

## Status: Implemented

`min_reputation` is a configurable field on `Bounty` (`src/types.rs:56`) and is
enforced in `claim_bounty` (`src/contract/mutations.rs:148`): a claim panics if
`bounty.min_reputation > 0 && contrib.reputation < bounty.min_reputation`.

This stub predates that implementation and is kept only for historical
reference. See `docs/security.md` for the current, maintained threat model —
this issue is not separately numbered there since it is fully mitigated.

## Issue (historical)
No minimum reputation for claimers.

## Fix (historical)
- Add reputation check before claim
- Configurable threshold
