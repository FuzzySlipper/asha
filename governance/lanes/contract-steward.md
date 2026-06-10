# Lane: contract-steward

## Owns
- `engine-rs/crates/protocol/protocol-ids` — shared ID wire types
- `engine-rs/crates/protocol/protocol-script` — policy view, command, and rejection types
- `engine-rs/crates/protocol/protocol-render` — render diff, geometry payload, handle types
- `engine-rs/crates/protocol/protocol-replay` — replay file, step, and hash types
- `engine-rs/crates/protocol/protocol-telemetry` — structured log and trace event types
- `engine-rs/crates/protocol/protocol-codegen` — TypeScript and schema code generator
- `ts/packages/contracts` — generated TypeScript contract package (read-only output)

## May depend on (Rust)
Foundation crates only. Protocol crates may depend on each other in the order above.
`protocol-codegen` may depend on all protocol crates.

## Must never touch
- Product-domain logic or convenience behavior inside protocol types.
- Renderer, sim, rule, or service crates.
- Hand-editing `ts/packages/contracts/src/generated/` — generated files only.

## Required tests / checks
- `cargo test -p protocol-codegen` — codegen produces valid output.
- Protocol fixture diff — `harness/goldens/protocol/` snapshots match current codegen output.
- `pnpm typecheck` in `ts/` passes after any generated file update.
- CI `check-contracts.sh` must pass: working-tree generated files match committed files.

## Required fixtures
- `harness/goldens/protocol/` — golden snapshots of generated TS output per protocol family.
- `harness/fixtures/` entries updated whenever a protocol type changes shape.

## Drift smells reviewers should flag
- Protocol type gaining a method with business logic.
- Manual edit committed to `ts/packages/contracts/src/generated/`.
- Generated diff committed without a matching Rust protocol source change.
- Protocol crate importing a sim, service, or rule crate.
- New protocol variant introduced without a downstream impact note.

## Public API changes that require escalation
Every change to a protocol type is a border change and requires:
1. Updated generated TS contracts (run codegen).
2. Updated golden fixtures.
3. Downstream package impact note (which TS packages are affected and do they still typecheck).
4. Compatibility note if the change is breaking for replay files or saved state.
