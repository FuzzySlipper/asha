# Contract change process

The Rust protocol crates (`engine-rs/crates/protocol/*`) are the single source
of truth for the generated TypeScript contracts in
`ts/packages/contracts/src/generated/`. Those generated files are never edited
by hand.

## The loop

1. Edit the Rust protocol crate source (the border shape).
2. Run `cargo run -p protocol-codegen` to regenerate the TypeScript contracts.
3. Run the downstream typecheck loop:
   - `pnpm --dir ts --filter @asha/contracts typecheck`
   - `pnpm --dir ts --filter @asha/contracts test`
4. Commit the generated diff together with the Rust source change.
5. Update any affected protocol golden fixtures and note downstream package
   impact (typecheck, test).
6. Request contract-steward review.

## Drift gate

`harness/ci/check-contracts.sh` runs `protocol-codegen --check`, which compares
the generator's deterministic output against the committed generated files
without mutating the tree. It fails when:

- a Rust protocol-source change was not regenerated, or
- a generated file under `ts/packages/contracts/src/generated/` was hand-edited.

The committed files in `ts/packages/contracts/src/generated/` (`ids.ts`,
`script.ts`, `render.ts`, `replay.ts`, `index.ts`) are the named, inspectable
golden for protocol output; review contract changes by reading their diff.

### Resolving a failure

The check's output names the offending file and points you to one of:

- **Border shape really changed** — edit the Rust protocol crate(s), then rerun
  codegen.
- **Stale or hand-edited generated file** — run `cargo run -p protocol-codegen`
  to regenerate and commit the result.
- **The new output is the intended golden** — commit it (and any golden
  fixtures) so it becomes the new baseline.

Manual edits to `ts/packages/contracts/src/generated/` are rejected by CI.

## Downstream impact

When a border shape changes, consult
[`protocol-border-consumers.md`](./protocol-border-consumers.md) for the
packages expected to consume each generated family, and note the affected ones
on the change.
