# Contract change process

1. Edit the Rust protocol crate source.
2. Run `cargo run -p protocol-codegen` to regenerate TypeScript contracts.
3. Commit the generated diff together with the Rust source change.
4. Update affected protocol golden fixtures.
5. Note any downstream package impact (typecheck, test).
6. Request contract-steward review.

Manual edits to ts/packages/contracts/src/generated/ are rejected by CI.
