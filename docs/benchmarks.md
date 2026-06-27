# Benchmarks

Performance notes for the MergeMint contract. Instruction counts are measured via Soroban's simulated execution environment (`simulateTransaction`), which returns `cost.cpuInsns` in the response.

---

## `complete_bounty` — storage read/write restructuring

**Branch:** `perf/complete-bounty-batch-writes`  
**Commit:** `perf: restructure complete_bounty to batch storage reads and writes`

### Change

Before, `complete_bounty` interleaved storage reads and writes with the token transfer:

```
read  bounty
            transfer tokens   (external call)
read  contributor
write contributor
```

After, all reads happen before the external call and all writes happen after:

```
read  bounty
read  contributor
            transfer tokens   (external call)
write contributor
```

The number of storage operations is unchanged (2 reads, 1 write). The improvement comes from access pattern locality: both ledger entries are fetched before the host executes the cross-contract token transfer, so the host can load them in the same scheduling window rather than suspending between the external call and the second read. This also eliminates the window between the external call and the second storage read where a reentrant call could observe stale contributor state.

### Instruction counts

Instruction counts are not yet captured here. To measure:

```bash
# Build
cargo build --release --target wasm32-unknown-unknown

# Deploy to testnet, then invoke complete_bounty via Stellar CLI
# and inspect the simulateTransaction response:
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source-account <ACCOUNT> \
  -- complete_bounty \
  --verifier <VERIFIER> \
  --bounty_id <BOUNTY_ID>
```

The `simulateTransaction` RPC response includes:

```json
{
  "cost": {
    "cpuInsns": "<before>",
    "memBytes": "<before>"
  }
}
```

Update this table once measurements are taken against both the old and new WASM:

| Version | `cpuInsns` | `memBytes` |
|---------|-----------|------------|
| Before  | —         | —          |
| After   | —         | —          |
| Delta   | —         | —          |
