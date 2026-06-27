# Contributing to MergeMint Contracts

## Development Workflow

A `Makefile` at the repository root provides shortcuts for all common tasks:

| Command | Description |
|---------|-------------|
| `make build` | Build the WASM contract |
| `make test` | Run the full test suite |
| `make lint` | Run Clippy (warnings as errors) and check formatting |
| `make fmt` | Auto-format source files with rustfmt |
| `make deploy` | Deploy the contract via `scripts/deploy.sh` |
| `make clean` | Remove build artifacts |

### Building

```bash
make build
# or: cargo build --release --target wasm32-unknown-unknown
```

### Testing

```bash
make test
# or: cargo test
```

All tests must pass before submitting a PR.

### Linting and Formatting

```bash
make lint   # clippy + fmt check
make fmt    # auto-format
```

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
