# Security: Prevent Bounty Creator From Claiming

## Issue
Bounty creator could claim their own bounty.

## Fix
- Add `require(creator != msg.sender)` check
- Validate claimer is not the original funder
