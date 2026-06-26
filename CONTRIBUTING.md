# Contributing to MergeMint Contracts

## Development Workflow

### Building

```bash
cargo build --release --target wasm32-unknown-unknown
```

Or with Make:

```bash
make build
```

### Testing

```bash
cargo test
```

Or:

```bash
make test
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

## TypeScript Bindings

Generate TypeScript bindings for the SDK:

```bash
make bindings
```

This produces `sdk/generated/` with typed contract interfaces. Bindings should be regenerated whenever the contract interface changes (function signatures, parameter types).

## Code Style

- Use `rustfmt` for formatting
- Follow Soroban SDK conventions
- Document non-obvious logic with inline comments

## Security

- All state-mutating functions require authentication
- Validate all external inputs
- Use `overflow-checks = true` for arithmetic safety
