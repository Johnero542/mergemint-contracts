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

## Storage Rent and TTL Management

### What Is TTL?

Soroban persistent storage is not free indefinitely. Each stored entry has a Time-To-Live (TTL) measured in **ledger sequences**. When an entry's TTL expires, the entry becomes archived and inaccessible until explicitly restored (at additional cost).

### Default TTL

- **Persistent storage default**: ~100,000 ledger sequences (~6 months)
- Current Soroban network: ~5-10 minute confirmation time per ledger

### Implications for MergeMint

If a bounty or contributor profile is not accessed for an extended period, its entry may expire. This is critical because:

1. **Bounties**: Unexpired bounties remain accessible until TTL expires
2. **Contributor Profiles**: Reputation data and earnings history could become inaccessible if not extended
3. **Escrow Risk**: Any escrowed tokens held against an expired bounty entry cannot be transferred until the entry is restored

### TTL Extension Strategy

Currently, MergeMint does not extend TTLs within contract logic. Future versions should:

- Extend entry TTLs on each access (read or write)
- Implement a separate TTL management contract function
- Document expected uptime requirements for contracts

For production deployments, monitor entry access patterns and explicitly extend TTLs before expiration.
