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

## Changelog

Every pull request that changes the contract interface **must** include a `CHANGELOG.md` entry under the `[Unreleased]` section. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

A "contract interface change" includes:
- Adding, removing, or renaming a public contract function
- Changing the parameters or return type of a public function
- Adding, removing, or reordering fields in `Bounty`, `Contributor`, `BountyMeta`, or `DataKey`
- Changing the set of events emitted by any function

Use the appropriate subsection (`Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`) inside `[Unreleased]`. Example:

```markdown
## [Unreleased]

### Added
- `update_contributor_metadata` — lets contributors update their off-chain profile URI.

### Changed
- `Bounty` — added optional `deadline` field (ledger sequence number).
```

PRs that touch only tests, documentation, CI, or tooling do not require a changelog entry, but one is welcome.

## Test Snapshots

### What Are Snapshots?

Test snapshots in `test_snapshots/` capture the Soroban ledger state after contract operations. They verify that storage, type encodings, and struct layouts remain consistent across code changes.

The current snapshots and the structs they cover:

| Snapshot file | Primary struct tested |
|---|---|
| `test_bounty_count.1.json` | `DataKey::BountyCount` |
| `test_claim_bounty.1.json` | `Bounty`, `Contributor` |
| `test_complete_bounty_updates_status.1.json` | `Bounty` status transitions |
| `test_contributor_reputation.1.json` | `Contributor` |
| `test_create_bounty.1.json` | `Bounty`, `DataKey` |
| `test_status_index_tracks_bounty_lifecycle.1.json` | `DataKey::StatusIndex` |

### How Snapshots Are Generated

Soroban's test infrastructure writes snapshot files automatically when a test that uses `Env::default()` completes. Running `cargo test` with a clean `test_snapshots/` directory will regenerate all files. On subsequent runs the framework compares the live ledger state against the stored JSON; a mismatch fails the test.

### When Snapshots Become Stale

Snapshots must be regenerated if:
- A field is added to `Bounty`, `Contributor`, or other `#[contracttype]` structs
- A field is removed or reordered
- A field type changes (e.g., `u32` → `u64`)
- Field visibility or attributes change
- A new `DataKey` variant is added

### Regenerating Snapshots

Delete the existing snapshots and rerun the test suite:

```bash
rm -f test_snapshots/test/*.json
cargo test
```

The test run will recreate all snapshot files from the current ledger state. Review the new JSON files with `git diff` before committing to confirm the changes are intentional.

If you only want to regenerate a single snapshot, delete that file and run the specific test:

```bash
rm test_snapshots/test/test_create_bounty.1.json
cargo test test_create_bounty
```

### Verifying Snapshots

To ensure all snapshots are current and pass:

```bash
cargo test
```

If all tests pass without modification, the snapshots are valid against the current schema.

## Code Style

- Use `rustfmt` for formatting
- Follow Soroban SDK conventions
- Document non-obvious logic with inline comments

## Security

- All state-mutating functions require authentication
- Validate all external inputs
- Use `overflow-checks = true` for arithmetic safety
