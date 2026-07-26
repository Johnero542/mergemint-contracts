# Security: Prevent Creator Self-Claim

**Status**: Implemented in contract (see [docs/security.md](../docs/security.md) Threat #2).

## Summary
Creator self-claiming is strictly prevented in `claim_bounty` to eliminate wash trading and self-rewarding exploits.
