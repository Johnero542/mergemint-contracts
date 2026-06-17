# Contract Architecture

## Data Flow

```
User (Frontend)
    │
    ▼
MergeMintContract
    │
    ├── create_bounty()
    │   ├── Validates creator auth
    │   ├── Stores bounty in persistent storage
    │   └── Emits bounty_created event
    │
    ├── claim_bounty()
    │   ├── Validates contributor auth
    │   ├── Assigns contributor to bounty
    │   └── Emits bounty_claimed event
    │
    └── complete_bounty()
        ├── Validates verifier auth
        ├── Transfers tokens via TokenClient
        ├── Updates contributor reputation
        ├── Emits bounty_completed event
        └── Emits reward_paid event
```

## Storage Layout

- `bounty_count` — u64 counter
- `bounty_{id}` — Bounty struct
- `contributor_{address}` — Contributor struct

## Events

| Event | Topics | Data |
|-------|--------|------|
| bounty_created | (Symbol, creator) | (bounty_id, reward) |
| bounty_claimed | (Symbol, contributor) | bounty_id |
| bounty_completed | (Symbol, contributor) | bounty_id |
| reward_paid | (Symbol, contributor) | (bounty_id, amount) |

## Security Model

- All state-changing functions require caller authentication via `require_auth()`
- Token transfers use Soroban's TokenInterface for safe transfers
- Bounty assignment is one-to-one — cannot claim already-assigned bounties
- Reputation is monotonically increasing
