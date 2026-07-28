# Proposal: Shared Type Generation Across Contract/Backend/Frontend/SDK

## Problem

`Bounty` and `Contributor` are independently hand-written in at least five places:

- `mergemint-contracts/src/types.rs` (source of truth: the Soroban contract)
- `mergemint-backend/src/scval.rs`
- `mergemint-backend/src/dto.rs`
- `mergemint-contracts/sdk/src/index.ts`
- `mergemint-frontend/src/types.ts`

A contract field rename currently requires manually touching all five, with
nothing in CI enforcing that they stay consistent. This is a latent source of
silent runtime bugs (e.g. a field renamed in the contract but missed in the
frontend type would fail at the JSON/XDR boundary, not at compile time).

## Proposed direction

Generate the backend DTOs and the frontend/SDK TypeScript interfaces from the
contract's Rust types, rather than hand-writing all five independently.

Two viable approaches:

1. **Direct codegen from Rust types.** Use a build-time tool (e.g. `ts-rs` or
   a custom `soroban-sdk` contractspec reader) to emit TypeScript interfaces
   for the SDK and frontend directly from `mergemint-contracts/src/types.rs`.
   The backend's `dto.rs`/`scval.rs` could either consume the same derive
   macro or be generated from the same source.

2. **Shared schema as the source of truth.** Define a JSON-Schema or OpenAPI
   spec for the backend-facing shapes, generate the backend DTOs and the
   frontend/SDK TypeScript types from that spec, and keep the contract's Rust
   types as the authoritative shape that the schema is checked against (e.g.
   via a CI round-trip test).

Option 1 keeps a single source of truth (the contract) and removes an extra
artifact to maintain. Option 2 is more flexible if the backend ever needs
fields that don't map 1:1 to the contract, at the cost of introducing a
fourth artifact (the schema) that itself needs to stay in sync.

## Recommendation

Start with option 1 given the current close 1:1 mapping between contract
types and DTOs. Land the codegen tool + generated SDK/frontend types first
(smallest blast radius), and revisit the backend DTO layer separately since
it has more business-logic-specific fields.

## Scope

This is a design note only. Given the cross-repo scope (contracts, backend,
frontend, and SDK all need coordinated changes), implementation should be
tracked as a separate follow-up ticket with its own plan and tests
(round-trip type checks wired into CI).
