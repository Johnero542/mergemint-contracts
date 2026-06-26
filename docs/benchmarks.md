# Performance Benchmarks

## `complete_bounty` — storage access optimisation

### Context

`complete_bounty` must (a) transfer the reward token, (b) update contributor reputation, and
(c) update the bounty's own status to `completed`. Steps (a) and (b) require reading the bounty
once at the top of the function. A naïve implementation of step (c) would issue a second
`storage::get_bounty` call after the contributor write to obtain a mutable binding. This second
read is redundant because the same data is already in scope.

### Before (naïve — two `get_bounty` calls)

```rust
// 1st read — needed for transfer + contributor logic
let bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");
let assignee = bounty.assignee.clone().expect("bounty has no assignee");

let token = TokenClient::new(&env, &bounty.reward_token);
token.transfer(&verifier, &assignee, &bounty.reward_amount);

// ... contributor update + store_contributor ...

// 2nd read — only needed because `bounty` was not declared `mut`
let mut bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");
bounty.status = Symbol::new(&env, STATUS_COMPLETED);
storage::store_bounty(&env, &bounty_id, &bounty);
```

Persistent storage operations: **4 reads, 3 writes** (plus 2 reads + 2 writes for the status
index maintained by `add_to_status_index` / `remove_from_status_index`).

### After (optimised — single `get_bounty` call)

```rust
// Single mutable read — used for transfer, contributor logic, AND status update
let mut bounty = storage::get_bounty(&env, &bounty_id).expect("bounty not found");
let assignee = bounty.assignee.clone().expect("bounty has no assignee");

let token = TokenClient::new(&env, &bounty.reward_token);
token.transfer(&verifier, &assignee, &bounty.reward_amount);

// ... contributor update + store_contributor ...

bounty.status = Symbol::new(&env, STATUS_COMPLETED);
storage::store_bounty(&env, &bounty_id, &bounty);
```

Persistent storage operations: **3 reads, 3 writes** (same index overhead as above).

### Summary

| Metric                    | Before | After | Delta |
|---------------------------|--------|-------|-------|
| `get_bounty` calls        | 2      | 1     | −1    |
| Persistent reads (total)  | 4      | 3     | −1    |
| Persistent writes (total) | 3      | 3     | 0     |

Eliminating the redundant `get_bounty` saves one persistent ledger read per `complete_bounty`
invocation. In the Soroban fee model, each persistent read costs CPU instructions and contributes
to the transaction fee, so removing it directly lowers the cost for every bounty completion.

To measure instruction counts in a test environment:

```rust
env.budget().reset_default();
client.complete_bounty(&verifier, &bounty_id);
let cpu = env.budget().cpu_instruction_count();
let mem = env.budget().memory_bytes_count();
```
