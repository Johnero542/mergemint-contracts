# `create_bounty` Multi-Assignee / Multi-Sig Parameter Design

## Overview

This note documents the final signature of `create_bounty` so that all three consuming repos (contract, backend, SDK/frontend) land the same parameter shape in one pass.

## Contract Function Signature

```rust
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
    required_verifiers: Option<Vec<Address>>,
    approval_threshold: u32,
) -> BountyId;
```

### Parameter Order Rationale

- **Creator through tags** follow the existing order, preserving backward compatibility for callers that do not use multi-assignee/multi-sig.
- **`required_verifiers`** comes after tags because it is optional (`Option<Vec<Address>>`) and groups the multi-sig feature together.
- **`approval_threshold`** comes last; it is relevant only when `required_verifiers` is `Some`, but always supplied (default `1`).

## Backend JSON Field Names

| Rust param              | JSON field                 |
|-------------------------|---------------------------|
| `creator`               | `creator`                 |
| `title`                 | `title`                   |
| `description`           | `description`             |
| `reward_amount`         | `reward_amount`           |
| `reward_token`          | `reward_token`            |
| `min_reputation`        | `min_reputation`          |
| `deadline`              | `deadline` (nullable)     |
| `tags`                  | `tags` (string array)     |
| `required_verifiers`    | `required_verifiers` (nullable string array) |
| `approval_threshold`    | `approval_threshold` (u32, default 1) |

## TypeScript SDK Field Names

```typescript
interface CreateBountyParams {
  creator: string;
  title: string;
  description: string;
  rewardAmount: bigint;
  rewardToken: string;
  minReputation: number;
  deadline: number | null;
  tags: string[];
  requiredVerifiers?: string[];
  approvalThreshold?: number;   // defaults to 1
}
```

## Validation Rules

| Condition | Behaviour |
|-----------|-----------|
| `reward_amount <= 0` | Panic `RewardMustBePositive` |
| `tags.len() > 5` | Panic `TooManyTags` |
| `required_verifiers` is `Some` and `approval_threshold > required_verifiers.len()` | Panic `ApprovalThresholdExceedsVerifiers` |
| `required_verifiers` is `None` | `approval_threshold` is stored but unused; `approve_completion` falls back to single-verifier completion |

## Consuming Repos

1. **Contract** (`mergemint-contracts`): Updated signature in `mutations.rs`, validation added.
2. **Backend** (`mergemint-backend`): `routes/tx.rs` must pass the new fields through to the contract call.
3. **SDK** (`sdk/src/index.ts`): `CreateBountyParams` updated with `requiredVerifiers` and `approvalThreshold`.
4. **Frontend**: Create-bounty form gains an optional "Add verifiers" section.
