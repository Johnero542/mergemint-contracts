# Contributing to MergeMint Contracts

## Development Workflow

### Building

```bash
cargo build --release --target wasm32-unknown-unknown
```

### Testing

```bash
cargo test
```

All tests must pass before submitting a PR.

## Test Snapshots

### What Are Snapshots?

Test snapshots in `test_snapshots/` capture the Soroban ledger state after contract operations. They verify that storage, type encodings, and struct layouts remain consistent across code changes.

### When Snapshots Become Stale

Snapshots must be regenerated if:
- A field is added to `Bounty`, `Contributor`, or other `#[contracttype]` structs
- A field is removed or reordered
- A field type changes (e.g., `u32` → `u64`)
- Field visibility or attributes change

### Regenerating Snapshots

If a test fails with a snapshot mismatch, regenerate with:

```bash
cargo test -- --nocapture
```

Then review the diff and commit the updated snapshots.

### Verifying Snapshots

To ensure all snapshots are valid:

```bash
cargo test
```

If tests pass, snapshots are current.

## Code Style

- Use `rustfmt` for formatting
- Follow Soroban SDK conventions
- Document non-obvious logic with inline comments

## Security

- All state-mutating functions require authentication
- Validate all external inputs
- Use `overflow-checks = true` for arithmetic safety
