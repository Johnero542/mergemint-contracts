# Contributing to MergeMint Contracts

## Prerequisites

- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/stellar-cli)

## Developer Setup

### 1. Clone and build

```bash
git clone <repo-url>
cd mergemint-contracts
cargo build
```

### 2. Install pre-commit hooks (recommended)

The hooks run `cargo fmt --check` and `cargo clippy -- -D warnings` before every commit, giving you instant local feedback on the same checks CI enforces.

```bash
bash scripts/install-hooks.sh
```

Once installed, the hooks run automatically. If a check fails the commit is aborted with a clear error message — fix the issue, re-stage, and commit again.

### 3. Run tests

```bash
cargo test
```

## Workflow

1. Fork the repo and create a feature branch off `main`.
2. Make your changes and ensure `cargo test`, `cargo fmt --check`, and `cargo clippy -- -D warnings` all pass.
3. Open a PR with a description that includes `Closes #<issue_id>`.

## Code Style

- Follow standard Rust formatting enforced by `rustfmt`.
- Zero Clippy warnings — the CI pipeline runs `cargo clippy -- -D warnings`.
