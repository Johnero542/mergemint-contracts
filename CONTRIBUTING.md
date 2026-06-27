# Contributing to MergeMint Contracts

Thank you for your interest in contributing! This guide covers everything you need to go from zero to a merged pull request.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Branch Naming](#branch-naming)
- [Pull Request Process](#pull-request-process)
- [Test Snapshots](#test-snapshots)
- [Security Considerations](#security-considerations)

---

## Prerequisites

Make sure the following are installed before you start:

### 1. Rust (stable)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify with `rustc --version`. The project targets **stable Rust** — nightly is not required.

### 2. WASM compilation target

```bash
rustup target add wasm32-unknown-unknown
```

This is required to build the contract for Soroban deployment.

### 3. Stellar CLI

```bash
cargo install stellar-cli
```

Verify with `stellar --version`. Used for building, deploying, and inspecting contracts.

---

## Development Workflow

### Clone the repository

```bash
git clone https://github.com/your-org/mergemint-contracts.git
cd mergemint-contracts
```

### Run the test suite

```bash
cargo test
```

All tests run against the Soroban in-process test environment — no live network required. The suite covers `create_bounty`, `claim_bounty`, `complete_bounty`, and the bounty counter.

### Build the WASM binary

```bash
cargo build --release --target wasm32-unknown-unknown
```

Output lands at `target/wasm32-unknown-unknown/release/mergemint_contracts.wasm`. The release profile is tuned for size (`opt-level = "z"`, `lto = true`) with overflow checks enabled.

---

## Code Standards

Both of the following must pass with **zero warnings or errors** before you open a PR. CI enforces both checks.

### Formatting

```bash
cargo fmt
```

Run this before every commit. Do not disable `rustfmt` attributes without a clear reason.

### Linting

```bash
cargo clippy -- -D warnings
```

All Clippy warnings are treated as errors. Fix every diagnostic rather than suppressing it with `#[allow(...)]` unless the lint is demonstrably a false positive and you explain why in the suppression comment.

---

## Branch Naming

Use one of these prefixes followed by a short kebab-case description:

| Prefix | When to use |
|--------|-------------|
| `feat/` | New contract functionality or behaviour |
| `fix/` | Bug fixes |
| `docs/` | Documentation-only changes |
| `test/` | New or updated tests with no production code change |
| `refactor/` | Internal restructuring with no behaviour change |
| `ci/` | Changes to GitHub Actions workflows or scripts |

Examples: `feat/claim-expiry`, `fix/double-claim-guard`, `docs/snapshot-guide`

---

## Pull Request Process

1. **Open or find an issue first.** Every PR should be traceable to a GitHub issue. If no issue exists for your change, open one before starting work so the approach can be discussed.

2. **Link the issue in your PR description.** Use GitHub's closing keyword so the issue closes automatically on merge:
   ```
   Closes #<issue-number>
   ```

3. **Describe what changed and why.** Include:
   - A short summary of the change.
   - The motivation or the problem it solves.
   - Any trade-offs or alternatives you considered.

4. **Paste test output.** Copy the result of `cargo test` into the PR description so reviewers can confirm the suite passes locally without checking out your branch.

5. **Screenshots for UI changes.** MergeMint Contracts is a pure on-chain library with no UI, but if your PR touches the deployment scripts or produces visual output (e.g. `stellar contract inspect` output), include a screenshot or terminal capture.

6. **Keep PRs focused.** One logical change per PR. Split unrelated fixes into separate branches.

7. **Respond to review comments promptly.** A PR that goes two weeks without a response may be closed and re-opened when you are ready to continue.

---

## Test Snapshots

### What are snapshots?

Files under `test_snapshots/` capture the full Soroban ledger state produced by each test. They verify that storage layout, type encodings, and struct field order remain stable across code changes. A snapshot mismatch is a breaking change to the on-chain storage format.

### When snapshots become stale

You must regenerate snapshots whenever you touch a `#[contracttype]` definition — specifically if you:

- Add, remove, or reorder a field in `Bounty`, `Contributor`, or `DataKey`
- Change a field's type (e.g. `u32` → `u64`)
- Rename a field

If you are unsure whether your change affects storage layout, run `cargo test` and check whether any snapshot diffs appear.

### Regenerating snapshots

```bash
cargo test -- --nocapture
```

The Soroban test runner rewrites `test_snapshots/` in place when snapshots are stale. After regeneration:

1. Review the diff with `git diff test_snapshots/` and confirm each change is intentional.
2. Commit the updated snapshot files in the same commit as the struct change — never in a separate commit, because the snapshots and the code must stay in sync.

### Verifying snapshots are current

```bash
cargo test
```

If all tests pass, all snapshots are current.

---

## Security Considerations

- Every state-mutating function must call `require_auth()` on the relevant `Address` argument before touching storage.
- Validate all external inputs at the contract boundary — do not rely on the caller to pass well-formed data.
- Do not introduce arithmetic that bypasses the overflow protection provided by `overflow-checks = true` in the release profile.
- If your change introduces a new trust boundary or changes which address is authorised to perform an action, call it out explicitly in the PR description and link to the relevant section of `docs/security.md`.
