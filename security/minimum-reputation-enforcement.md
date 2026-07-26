# Security: Enforce Minimum Reputation Threshold

**Status**: Implemented in contract (see [docs/security.md](../docs/security.md) Threat #1).

## Summary
The `min_reputation` check is fully enforced during `claim_bounty` calls to prevent unverified accounts from claiming high-value tasks.
