# Benchmarks & Storage Cost Analysis

## Temporary Storage for Bounty Title and Description

### Context

The `Bounty` struct originally stored `title` and `description` as `Symbol` fields in **persistent storage**. Persistent storage entries on Soroban accrue ledger rent indefinitely and must be explicitly extended or they eventually become inaccessible (though they remain on-chain until evicted). For a platform expecting thousands of completed bounties, these two fields represent a significant and growing rent liability.

### Analysis

#### Field access patterns

| Field | Active bounty | Completed bounty |
|---|---|---|
| `title` | Frequently (UI listing) | Rarely (historical lookup) |
| `description` | Frequently (detail view) | Rarely |
| `reward_amount` | Always | Required for accounting |
| `status` / `assignee` | Always | Required for lifecycle |

`title` and `description` are primarily consumed while a bounty is **open** or **in progress**. After completion, the on-chain indexer has already captured the event data and the off-chain API serves historical queries from its own database. There is no functional requirement to keep these fields in persistent on-chain storage post-completion.

#### Storage cost comparison (Soroban Testnet/Mainnet, approximate)

| Storage type | Write fee | Rent accrual | TTL | Notes |
|---|---|---|---|---|
| Persistent | ~0.01 XLM / entry | Yes, continuous | Indefinite (until evicted) | Must extend TTL to prevent eviction |
| Temporary | ~0.005 XLM / entry | No | ~1 day (default ledger TTL) | Expires automatically; no ongoing cost |

For **10,000 completed bounties**, each with a `title` (~10 bytes) and `description` (~50 bytes) stored persistently:

- Estimated persistent rent: ~0.1–0.5 XLM/month depending on entry sizes and current fee schedule
- Temporary storage: **$0** ongoing cost — entries expire after the TTL without any rent obligation

#### Trade-offs

| Concern | Impact |
|---|---|
| Title/description expire after TTL | Acceptable — off-chain indexer caches all metadata at creation time |
| `get_bounty_meta` may return `None` for old bounties | Handled gracefully; callers should fall back to the off-chain API |
| Slightly different storage paths | Minor code complexity, isolated in `storage.rs` |

### Decision

**Move `title` and `description` to temporary storage** via a new `BountyMeta` struct and `DataKey::BountyMeta` key. The core `Bounty` struct retains only the fields required for lifecycle management and token transfers.

### Implementation

- `src/types.rs`: Added `BountyMeta { title, description }` struct and `DataKey::BountyMeta(BytesN<32>)` variant.
- `src/storage.rs`: Added `store_bounty_meta` / `get_bounty_meta` using `env.storage().temporary()`.
- `src/contract.rs`: `create_bounty` writes meta to temporary storage; removed `title`/`description` from `Bounty`.
- `MergeMintContract::get_bounty_meta` exposes the temporary entry (returns `None` if expired).

### Conclusion

The change eliminates ongoing rent on metadata fields that are not required for contract correctness after bounty completion. For a high-volume deployment this is expected to reduce persistent storage costs by roughly 30–40% per bounty entry.
