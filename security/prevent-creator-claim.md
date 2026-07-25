# Security: Prevent Bounty Creator From Claiming

## Status: Not yet enforced

This is tracked in detail as threat vector **#3 (Self-claim)** in
`docs/security.md`, which is the current source of truth. Per that writeup,
the `CREATOR_CANNOT_CLAIM` error constant exists in `src/errors.rs` but is not
enforced in `contract.rs`; the fix is a single guard at the top of
`claim_bounty` comparing `contributor == bounty.creator`. See
`docs/security.md` for full description, residual risk, and the escrow
pre-merge checklist item that also references this fix.

## Issue (historical)
Bounty creator could claim their own bounty.

## Fix (historical)
- Add `require(creator != msg.sender)` check
- Validate claimer is not the original funder
