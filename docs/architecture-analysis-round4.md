# Architecture Analysis Round 4 — ASHA

*Date: 2026-07-09. Read-only re-review of the repository at commit `cdd72f29`, comparing against `docs/architecture-analysis-round3.md` (2026-07-06, commit `c0c698c7`). Scope: which of round 3's 6 priority items were implemented, how the new campaigns (game Rust extension boundary, voxel edit history, voxel annotations/assets, mesh animation, browser host) crossed the border, and what new pressure points emerged. 122 commits and ~64k inserted / ~15k deleted lines since round 3.*

## Summary

Second consecutive round in which every priority item moved. The governance rails are now demonstrably shaping new work rather than chasing it: five new protocol crates, four new service/rule crates, one new state crate, and two new TS packages all appeared in `ownership.toml` at birth with tightened allowlists and tests, and every campaign crossed the border in the prescribed protocol → service/rule → bridge → generated contracts → facade order. The headline addition — the compiled game-rule extension boundary (`public-rust/game-rule-extension`) — is the repo's first real answer to "how does a downstream game own Rust behavior without becoming a second engine," and it landed with the fail-closed, replay-evidenced shape the design doc prescribed.

The remaining pathology is concentrated in exactly one lane: the bridge. On the Rust side, the round-3 monolith split moved most of its mass into `reference/mod.rs` (3,735 lines) — which the native napi addon wraps directly, making a module named "reference" the actual production authority body. On the TS side, the source-shape ratchet is being *bumped inside feature commits* rather than honored, and `runtime-session.ts` now sits 6 lines under its raised cap. Both containment mechanisms are weakest precisely where every campaign's payload lands.

## Scorecard against round 3's priority list

### 1. R3-1: unit tests in `rule-lifecycle` — **done**

1 → 13 tests; the crate grew to 1,800 lines and split into `lib.rs` + `lifecycle_primitives.rs`. `FpsRuntimeSessionState` is defined here (not in the bridge), and the bridge's `Cargo.toml` carries a rationale comment ("rule-lifecycle owns ProjectBundle bootstrap, combat/lifecycle, and replay hashes") — the promote-authority-with-tests rule was applied.

### 2. R3-3: split `runtime-bridge-api/lib.rs`; extend source-shape policy to Rust — **done, with the mass displaced rather than dissolved**

`lib.rs` is a 148-line re-export root over `errors.rs` / `handles.rs` / `bridge.rs` / `payloads.rs` / `buffer_provider.rs` / `reference/`. `harness/depgraph/check-rust-source-shape.mjs` + `rust-source-shape-policy.json` exist and run in `check-depgraph.sh`. Two caveats, both material:

- The monolith's mass largely moved into `reference/mod.rs`, now 3,735 lines — bigger than any file the round-3 finding measured except the one it split. See R4-1.
- The Rust exemption list is prose-only. Unlike the TS policy, entries carry no recorded `maxLines` baseline, so exempted files can grow without any diff-visible signal. `protocol-codegen/src/model.rs` did: 2,786 → 4,546 lines this round. See R4-2.

### 3. R3-2: reference launcher out of the root module graph; depgraph guard — **done**

`reference-launcher.ts` exists; `check-runtime-bridge-root-isolation.mjs` walks the root entry point's module graph and fails if it reaches `mock.ts`, `mock-session.ts`, `reference.ts`, `reference-browser.ts`, or `reference-launcher.ts`; wired into `check-depgraph.sh`. The entry-point split is now load-bearing.

### 4. R3-6: `@asha/runtime-session` facade decomposition — **half done; the half that stayed is at its cap**

The package exists (~2,000 lines): `encounter-director`, `enemy-policy`, `combat-feedback`, `combat-readout`, `nav-readout`, `generated-tunnel`, `runtime-action` — the game-domain projection helpers left the transport package as prescribed. But the facade itself did not move: `runtime-session.ts` (1,804), `runtime-session-ecrp.ts` (866), `runtime-session-lifecycle.ts` (586), and `runtime-session-rust-facade.ts` (1,480) remain in `runtime-bridge`, whose `src/` is now ~19.9k lines (~5.6k of it tests). The exemption justification names the destination — "until transport-neutral facade types and focused capability adapters are extracted" — but no task has landed it, and the cap headroom is now 6 lines. See R4-3.

### 5. R3-4: IR round-trip completeness loop — **done; the destination got more expensive**

`missing_round_trip_coverage(model::all_modules())` fails with module names, so new IR modules can no longer skip round-trip coverage. But the hand-maintained IR nearly doubled this round (2,786 → 4,546 lines) as game-rules, game-extension, voxel-asset, voxel-annotation, voxel-edit-history, and animation vocabulary landed. See R4-4.

### 6. R3-5: shrink-only exemption ratchet; split `game-workspace` — **mechanically done; behaviorally leaking**

`check-ts-source-shape.mjs` enforces recorded `maxLines` baselines, and `game-workspace/src/index.ts` was split (`14a271c0`) into manifest modules. But the policy file's history shows the caps being raised inside working feature commits rather than files being split: `runtime-session.ts` 1633 → 1676 (`a6d5a453`, game-rule invocation) → 1800 (`79106f07`, CI greening) → 1810 (`c4992d86`, conversion metadata); `native-fail-closed.test.ts` 1640 → 1720 at HEAD-3 (`40ae9f4c`); `native-bridge/index.ts` 311 → 326 → 360; a fresh `mock.ts` exemption at 1620 (current: 1608). The ratchet produces diff visibility, which is worth something — but nothing distinguishes a reviewed raise from a raise-to-make-CI-pass. See R4-3.

## Growth since round 3 crossed the border correctly

The campaigns themselves are the strongest evidence yet that the architecture is teaching its contributors:

- **Compiled game-rule extension boundary** (`#4488`/`#4516`/`#4517`): `protocol-game-extension` (schema-only DTOs) → `game-rule-extension` (public `GameRuleModule` trait, default hooks fail closed) → `public-rust/game-rule-extension` (downstream facade crate with `[package.metadata.asha.public-surface]` declaring status/role/source-of-truth/allowed-consumer-roles) → generated `gameExtension.ts` → `RuntimeSessionFacade.submitGameExtensionWeaponEffect`. Replay evidence records module id/version/contract hash, input/proposal hashes, and acceptance results. `public-rust/` is a new repo-level concept: the Rust analogue of the public TS surface file. The parallel game-rules substrate (`core-game-rules`, `svc-game-rules`, `rule-game-modifier`, `protocol-game-rules`) landed with 2–10 tests per crate and fixture snapshots.
- **Voxel edit history**: `protocol-voxel-edit-history` → `rule_voxel_edit::history` (Rust-owned timeline + cursor, revert implemented as deterministic replay, transactional append fixed in `4722f908`) → ProjectBundle persistence with fail-closed hash/quota load conditions → bounded bridge read/diff verbs (`MAX_HISTORY_READ_ENTRIES = 1000`, partial-diff classification) → native wiring at HEAD. `docs/voxel-edit-history.md` is the clearest authority-posture doc in the repo — "Studio does not own the undo stack," undo/redo as convenience over the revert cursor, per-lane histories until a cross-surface envelope exists.
- **Voxel annotations and voxel volume assets**: same protocol → service → bundle-load → bridge-ops → consumer-proof ladder, each rung a separate commit.
- **Mesh animation**: classified correctly as projection. `protocol-render` defines `AnimationClipDescriptor`/`AnimationLoopMode` and playback commands documented as "projection-only… renderer mixer inputs"; the Three adapter (`animated-mesh.ts`) owns the mixer; RuntimeSession exposes animation *intent* readouts; the fixture is a real licensed asset (Kenney GLB) with manifest and normalized license text. No authority leaked into the renderer.
- **Browser host**: `@asha/browser-host` installs the native provider with `productAuthority: true`, `reference fallback: false`, and a fail-closed `missing_rust_backend` readout; `508e7b4e` derives the RPC endpoint set from the runtime-bridge manifest instead of a hand-maintained list, deleting a drift source.
- **Agent navigation layer**: `docs/code-map/*` + `docs/agent-code-atlas.md` with a CI checker (`harness/code-map/check-agent-code-atlas.py`).

## New findings

### R4-1. "Reference" now means opposite things on the two sides of the border (small effort, do it before it embeds further)

In TypeScript, "reference" is rigorously *not-product-authority*: `REFERENCE_RUNTIME_BACKEND_PROFILE.productAuthority === false`, separate entry point, root-isolation guard, split evidence lanes. In Rust, `ReferenceBridge` in `runtime-bridge-api/src/reference/` **is the production authority body**: `native-bridge`'s napi addon holds `BTreeMap<u64, ReferenceBridge>` and marshals every native call into it. The module header still reads "Tiny in-crate implementation for smoke tests… The real native body lives in `native-bridge`" — stale in both directions (3,735 lines; and `native-bridge` is a marshaling wrapper around this very struct). HEAD's "Initialize native voxel edit history authority" writes 700 lines into `reference/voxel_history.rs`.

The delegation inside it is correct — voxel history delegates to `rule_voxel_edit::history`, FPS state to `rule-lifecycle` — so this is a *legibility* defect, not an authority inversion. But this repo's premise is that agents cold-start from names and docs, and an agent asking "is this module product authority?" currently gets a hard no from the name and a hard yes from the dependency graph. **Suggestion:** rename the module and struct (`authority/` + `EngineBridge`, or `bridge_impl/`), fix the header to state plainly that this is the shared authority body behind both the in-crate conformance tests and the napi addon, and reserve "reference" repo-wide for not-product-authority surfaces. A `check-vocabulary.sh` entry can hold the line afterward.

### R4-2. Rust exemptions have no numeric baselines (small)

`rust-source-shape-policy.json` exempts six files (17,031 lines total) with prose-only justifications. The TS side records `maxLines` per exemption and fails on growth; the Rust side would have flagged `model.rs` (+1,760 lines this round) and `reference/mod.rs`'s accretion. The checker and policy model already exist on the TS side — port the baseline field and the fail-on-grow branch.

### R4-3. Cap raises ride inside feature commits; the facade extraction the exemption promises keeps not happening (small mechanism + one medium scheduled task)

Round 3's ratchet was adopted, then routed around: four caps were raised across five commits, always in the commit that needed the room. Two suggestions, either sufficient: (a) make `check-ts-source-shape.mjs` reject a raised baseline unless the policy entry carries a dated changelog line (visible, greppable, auditable); or better (b) actually schedule the extraction the `runtime-session.ts` exemption text names — transport-neutral facade types out of `runtime-bridge`, joining the readouts already in `@asha/runtime-session`. At 6 lines of headroom, the next FPS/ECRP campaign forces this decision within days; deciding it deliberately is cheaper than deciding it inside whatever commit hits the cap.

### R4-4. The codegen IR is the last round-1 finding still open, and it nearly doubled this round (medium; stop deferring or explicitly accept it)

`model.rs` is 4,546 lines, hand-maintained, growing ~60% per round as campaigns add protocol modules. The completeness loop (R3-4) caps the *silent-drift* risk, which was the acute danger; what remains is pure maintenance drag plus the residual risk on unsampled fields. This has been priority-ranked in all four analyses. Either schedule the derive-the-IR-from-Rust task (serde-reflection or a proc-macro walk) as real work with a task number, or write an ADR accepting the hand-maintained IR + completeness-loop posture as the durable design and stop re-flagging it. Both are defensible; the current middle path — perpetual "destination still open" — is the only indefensible one.

### R4-5. README structural counts are stale (trivial)

`README.md` says 64 crates / 21 packages; actual: 78 crates / 24 packages. In a repo whose premise is agent cold-start from docs, the orientation numbers should be right — or generated.

## Priority order for the next round

| # | Item | Effort | Payoff |
|---|------|--------|--------|
| 1 | R4-3(b): schedule and land the runtime-bridge facade extraction | Medium | The cap decision arrives within days regardless; every campaign until then bumps caps |
| 2 | R4-1: rename `reference/` / `ReferenceBridge` to authority vocabulary; fix the header | Small | Restores the repo's single most load-bearing vocabulary distinction |
| 3 | R4-2: numeric baselines in `rust-source-shape-policy.json` | Small | Same trick that worked for TS; catches `model.rs`-style silent growth |
| 4 | R4-3(a): changelog-gated cap raises | Small | Makes ratchet bumps reviewable instead of ambient |
| 5 | R4-4: decide the IR endgame — derive task or acceptance ADR | Medium (or trivial if ADR) | Ends a four-round deferral loop either way |
| 6 | R4-5: fix or generate README counts | Trivial | Cold-start accuracy |
