# Architecture Analysis — ASHA

*Date: 2026-07-01. Read-only review of the repository at commit `af052878`. Scope: structural/architectural assessment with improvement suggestions; no code changes.*

*Addendum (same date): findings 10–11 cover the cross-repo product shape with `../asha-studio` and `../asha-testing` (formerly `asha-demo` — renamed to reflect its actual role as the boundary-conformance consumer; a fresh `asha-demo` pursuing a human-facing deliverable is planned separately).*

## Summary

ASHA's architecture is unusually disciplined for a repo of this size (63 Rust crates, 20 TS packages): a single authority boundary ("Rust owns authority, TypeScript owns expression"), machine-readable lane ownership, generated contracts with drift checks, and fail-closed bridge semantics. The documented architecture and the code largely agree, which is the hardest property to keep.

The most valuable improvements are not new layers but closing gaps in the enforcement the architecture already promises:

1. **`may_depend_on` allowlists are documentation-only** — the dep-graph verifiers enforce only denylists, so most of `ownership.toml` is not actually checked.
2. **`ts/packages/game-workspace` has no ownership entry at all**, and the TS verifier (unlike the Rust one) has no completeness check to catch that.
3. **The protocol codegen IR is a 2,400-line hand-maintained mirror** of the Rust protocol crates — a second source of truth on the border the architecture says has exactly one.
4. **`runtime-bridge/src/index.ts` (1,946 lines) has become a grab-bag** mixing contract types, error taxonomy, the mock backend, the native backend, and the game-runtime launcher facade in one module.

Details and lower-priority findings below.

## What the architecture is (observed, not just documented)

- **Rust workspace** (`engine-rs/crates/`) layered as foundation → state → protocol / sim / services / rules → render / bridge / wasm / tools. Layering is real: spot-checked `Cargo.toml` files match the intended direction, and state crates carry explicit `may_not_depend_on` guards against render/bridge back-edges.
- **Contract border**: Rust protocol crates + `protocol-codegen` emit committed TS files under `ts/packages/contracts/src/generated/`; `check-contracts.sh --check` catches both stale regeneration and hand-edits. This is a good design — deterministic output compared in CI, no separate git-diff guard needed.
- **Runtime bridge**: `@asha/runtime-bridge` is the transport-neutral facade; native (napi), reference/mock, and WASM-replay backends sit behind it. The fail-closed rule (no silent native→mock downgrade) is implemented and tested (`native-fail-closed.test.ts`).
- **Governance as code**: `governance/ownership.toml` assigns every crate/package to a lane with dependency rules; `harness/depgraph/` verifies it; ADRs and lane docs carry the prose rationale.
- **Composition root**: `composeAppShell` in `@asha/app` is the single shell composition for Electron, browser, and headless CLI; `electron-main` is genuinely thin (148 lines, imports no runtime packages). This matches the docs exactly.

## Strengths worth preserving

- **One authority plane, stated everywhere and mostly enforced.** The command →
  validation → owner fact/state → gameplay-fabric delivery → projection pipeline
  keeps its categories distinct rather than collapsing into an untyped ambient
  bus.
- **Contracts drift-checked in CI**, with error messages that tell the agent which of three causes applies and how to resolve each. The CI scripts throughout are unusually good at explaining failures — keep this bar.
- **Ownership completeness is enforced on the Rust side**: every workspace member must have an ownership entry or a documented exemption. (The TS side lacks this — see finding 2.)
- **Test discipline is broadly good**: 50 of 63 crates have tests, and most of the untested remainder are empty placeholders (see finding 6). TS packages pair nearly every module with a `.test.ts`.
- **Honest runtime-mode reporting** (`native` / `reference` / `degraded` / `unavailable`) instead of silent fallback.

## Findings and suggestions

### 1. `may_depend_on` allowlists are never enforced (high value, low cost)

`governance/ownership.toml` declares 63 `may_depend_on` allowlists with careful per-crate comments, but `harness/depgraph/verify-rust-deps.sh` checks only `may_not_depend_on`, and `verify-ts-deps.sh` checks only `may_not_import`. A crate can add any dependency that isn't explicitly denylisted and CI passes. Since the denylists mostly guard a handful of known-bad edges (render/bridge back-references), the great majority of the declared dependency policy is aspirational.

For a repo explicitly designed so "many short-lived coding agents work in bounded lanes," this is the single highest-leverage fix: agents *will* add conveniences that no one thought to denylist. **Suggestion:** make the verifiers treat `may_depend_on` as an allowlist — any actual dependency not in the list (and not a shared external crate) fails with the same route-through-the-border message the TS verifier already prints. If some crates intentionally have open dependencies, mark them `may_depend_on = "unrestricted"` explicitly rather than leaving enforcement silently absent.

### 2. `game-workspace` is outside governance; TS side has no completeness check

`ts/packages/game-workspace` (1,480 hand-written lines, actively developed) has no entry in `ownership.toml`. Nothing failed because the TS verifier iterates over ownership entries rather than over actual packages. The Rust verifier already solves this exact problem with its ownership-completeness loop and `ownership_exempt` escape hatch.

**Suggestion:** port the completeness check to `verify-ts-deps.sh`, then add the missing `game-workspace` entry (lane, allowed imports). Two smaller hardening notes on the same script: `pkg_dir.rglob("*.ts")` also matches `dist/**/*.d.ts` (and would match `node_modules` if packages ever gain nested ones), and the import regex won't match multiline imports — restricting the scan to `src/` and adding the package.json check it already does would be sufficient.

### 3. The codegen IR is a hand-maintained second description of the border

`protocol-codegen/src/model.rs` (90 KB, ~2,400 lines) is, per its own doc comment, "the hand-maintained IR that mirrors the Rust protocol crates… kept in lockstep" by discipline plus the committed-goldens check. But the goldens check only catches drift between the IR and the *generated TS* — nothing mechanical catches drift between the IR and the *Rust protocol types*. If someone adds a field to a Rust protocol struct and forgets `model.rs`, CI passes: the contracts are "in sync" with an IR that no longer matches Rust. The doc comment notes that some values (branded ID list, replay format version) are sourced directly from protocol crates "so those facts have a single home" — an acknowledgment that the rest have two homes.

**Suggestion (incremental, in order of increasing effort):**
- Extend the "source directly from the protocol crate" pattern: constants, enum variant lists, and discriminant tags can be pulled from the Rust crates today.
- Add round-trip tests: serialize a sample of each Rust protocol type with serde and validate the JSON against the IR's field/variant description (or against the generated TS via a JSON-schema emitted from the same IR). This turns "kept in lockstep by discipline" into a CI failure.
- Longer term, derive the IR from the Rust types themselves (a proc-macro on protocol types, or `serde-reflection` tracing) so the border has one home. Given how central "generated contracts define the border" is to this architecture, this is the right eventual destination; the IR builder DSL in `model.rs` is well-factored, which will make migration easier.

### 4. `@asha/runtime-bridge` index.ts is a monolith mixing five concerns

`runtime-bridge/src/index.ts` is 1,946 lines containing: branded handle types and validation helpers; the `RuntimeBridgeError` taxonomy; the `RuntimeBridge` interface; the full `MockRuntimeBridge` (~450 lines); the `NativeRuntimeBridge` and native error classification; and the entire `GameRuntime*` launcher facade (~40 exported types plus three launcher factories) with camera math (`basisFromPose`, `matrixKey`) inlined. `render-decode.ts` shows the package already knows how to split modules.

**Suggestion:** split along the seams that already exist in the file — `errors.ts`, `bridge.ts` (interface + handle types), `mock.ts`, `native.ts`, `launcher.ts` (the GameRuntime facade), `camera.ts` — with `index.ts` as re-exports so no consumer changes. Two structural questions worth deciding while splitting:
- Should the **mock backend ship in the production facade package** at all? Fail-closed semantics get easier to trust when the reference backend is a separate entry point (`@asha/runtime-bridge/reference`) or package, so production bundles can't accidentally construct it.
- The `GameRuntimeLauncher` facade (see `docs/game-runtime-launcher-facade.md`) is arguably a layer *above* the transport bridge; if it keeps growing it deserves its own package with `runtime-bridge` as a dependency.

### 5. The single-file-package pattern doesn't scale past ~1,000 lines

Several packages concentrate nearly everything in one `src/index.ts`: `game-workspace` (42 KB / 1,034 lines), `renderer-three` (1,100 lines), `ui-dom` (533), plus the `runtime-bridge` case above. `devtools` shows the better pattern already in use — one module per panel with a small `index.ts`. For an agent-oriented repo this matters more than usual: bounded tasks against a 1,000-line file collide; bounded tasks against `scene-outliner.ts` don't.

**Suggestion:** adopt a soft convention (lane docs or a lint) that `index.ts` is exports-only once a package passes a few hundred lines, and split `game-workspace` and `renderer-three` before they grow further.

### 6. Placeholder crates/packages blur the map

`rule-lifecycle`, `rule-process`, `rule-relationship`, `rule-state-machine`, `svc-physics`, `svc-pathfinding`, `svc-rng` are 1-line (or near-empty) Rust crates; `catalog-core`, `catalog-examples`, `cosmetic` are `export {};` TS packages. They inflate the workspace count (the "60 crates" in README includes them), dominate the untested-crate list, and force every workspace-wide operation to touch them. They also carry ownership entries indistinguishable from real crates, so an orchestrator routing work by `ownership.toml` can't tell a lane with code from a reserved name.

**Suggestion:** either delete them until needed (lanes can be reserved in `ownership.toml` with a `status = "reserved"` field instead of an empty crate), or mark them explicitly (`#![doc = "placeholder"]` plus a `status` field) so tooling and reviewers can filter them.

### 7. Genuine test gaps on the border

Setting placeholders aside, the untested list reduces to a few crates that are *not* trivial: `bridge/native-bridge` (the napi addon — presumably exercised indirectly via `check-native.sh` and the TS `native-fail-closed` tests, but it has no Rust-side tests), and `protocol-policy-view` / `protocol-telemetry` (border contract crates, where shape regressions are exactly what you want caught closest to the source). `tools/protocol-dump`, `snapshot-diff`, `state-inspector` are lower priority but are also the tools agents rely on to diagnose everything else.

**Suggestion:** add at least serialization round-trip tests to the two protocol crates, and a smoke test to native-bridge that can run without the compiled addon (validating manifest/registration logic).

### 8. Repo hygiene: committed session artifacts at the root

Seven files from a dev session of task 3465 are committed at the repository root: `asha-studio-3465-dev*.log`, `*.pid`, and three `asha-studio-3465-agora-*.json`. `.gitignore` covers `harness/*-out/` but not root-level logs/pids. These are exactly the kind of files agent sessions generate.

**Suggestion:** remove them and add root patterns (`*.log`, `*.pid`, session-artifact naming) to `.gitignore`, or better, point whatever wrote them at `harness/`-style output directories already ignored.

### 9. Documentation has three overlapping entry points

`README.md` (10 KB), `docs/design.md` (48 KB), `docs/architecture-overview.md` (2 KB), and `governance/architecture.md` (10 KB) all describe the architecture at different altitudes and ages. The README explicitly warns "do not infer current work from old phase language," which suggests drift has already bitten once. With Den as the declared source of truth for current work, the repo docs' job is durable architecture only.

**Suggestion:** make `docs/design.md` the single canonical architecture document, reduce `architecture-overview.md` and `governance/architecture.md` to short pointers (or fold the governance one into the lane docs), and date-stamp sections that describe posture rather than invariants.

## Cross-repo findings (asha-studio / asha-testing)

The engine is consumed by two sibling repos via local package links: `../asha-studio` (Angular/Nx editor frontend) and `../asha-testing` (formerly `asha-demo`; the boundary-conformance consumer). Both carry their own machine-readable boundary policies and honor the border — no `src/*`, generated-path, or raw-transport imports; native access stays fail-closed behind `runtime-bridge.v0` metadata. The two findings below are where that arrangement is drifting.

### 10. The public surface is defined by the consumers, not the engine — and the copies have diverged

There is no single place where the engine declares which of its packages are public. Instead, each consumer maintains its own allow-list, and asha's compatibility doc maintains a third view. As of this review the four sources disagree:

| Source | Packages treated as public |
|---|---|
| `asha/docs/consumer-compatibility.md` (Tier 1 metadata table) | 2: `contracts`, `runtime-bridge` |
| `asha-studio/boundary-policy.json` `allowedSourceImports` | 6: adds `command-registry`, `devtools`, `editor-tools`, `game-workspace` |
| `asha-studio/package.json` `ashaStudio.allowedAshaPackages` | 5: same list *minus* `devtools` — while `devtools` sits in the same file's `dependencies` **and** in its `deferredPublicPackages` list |
| `asha-testing/boundary-policy.json` `allowedPackages` | 4: `contracts`, `runtime-bridge`, `devtools`, `game-workspace`, plus `renderer-three` as "unstableDemoPackages" |

Concrete drift this has already produced:

- **`@asha/game-workspace` became de-facto public without the engine ratifying it.** Both consumers depend on it, its `package.json` carries a `compatibility` block pointing at `docs/consumer-compatibility.md#game-workspace-compatibility-log` — an anchor that does not exist in that document — and (finding 2) it has no entry in `governance/ownership.toml`. A package crossed the border with none of the engine-side bookkeeping the border is supposed to require.
- **Studio's two in-repo truths contradict each other** on `devtools` (allowed source import vs. deferred public package), so an agent reading one manifest gets a different border than an agent reading the other.
- **`renderer-three`'s status is unresolved**: asha-testing may use it as "unstable," studio is forbidden from it and re-implemented its own Three.js viewport projection locally. Whichever way that decision goes (promote render-diff application to a public consumable, or declare renderers consumer-owned and demote `renderer-three` to an example), it should be recorded engine-side, not inferred from two consumers' divergent policies.

**Suggestion:** the engine declares its own export tiers in one machine-readable file (extend `harness/public-surface/` metadata or add `public-surface.json`), listing each package's tier (`public` / `unstable` / `internal`), its compatibility marker, and its changelog anchor. Consumer boundary checkers then *validate their allow-lists against* that file rather than maintaining independent truths. `check-public-boundary.py` should verify that every package claiming a `compatibility` block has a real changelog section and an `ownership.toml` entry — that check alone would have caught the `game-workspace` case. This also gives the planned clean `asha-demo` repo a ready-made contract to start from instead of copying a third allow-list.

### 11. The agent-proof exoskeleton was copied into the product repos and is crowding out the product

In asha, the evidence/harness machinery is the point — the repo exists so agents can work in bounded lanes with proof obligations. In the consumer repos that same machinery has been reproduced at full strength, where it obscures what the repo *is*:

- **`asha-studio/package.json` is a task journal.** The `ashaStudio.knownLimitations` array holds ~20 paragraph-length task write-ups (3042–3047, 3215–3220, 2730-series); the task 3219 Agora-compositor entry alone is a full page of prose embedded in a manifest. The README is likewise a task-by-task accretion log rather than a description of an editor.
- **The prose has already drifted from the machine reality it sits next to.** `knownLimitations` and the README cite verification commands — `proof:gizmo`, `proof:inspector`, `proof:agora-compositor`, `proof:visual-contract`, `proof:visual-capability` — that do not exist in the same file's `scripts` block (renamed, consolidated, or moved to Nx targets without the journal being updated). Roughly 35 of studio's 43 npm scripts are `proof:*` entries; the app itself has `dev`, `build`, `test`.
- **The same pressure created the identity problem in asha-testing** (previously asha-demo): its script surface is almost entirely `proof:*`/`check:*`/`publish:evidence`. For a conformance consumer that *is* the correct shape — which is why the rename resolves it — but it demonstrates the pattern: evidence apparatus, left unchecked, becomes the repo.

The root cause is that task evidence is being stored *in the product manifests* because it has nowhere else durable to go, even though Den is already declared the source of truth for task state.

**Suggestion:**
- Treat `boundary-policy.json` as the **only** machine-readable policy in each consumer repo; delete the redundant `ashaStudio.allowedAshaPackages` block (or reduce it to a pointer). asha-testing's README already states this single-source rule — apply it in studio.
- Move `knownLimitations` content to `docs/` (or Den task records), one file per surface area rather than one array entry per task. A manifest field that must be hand-scrolled is not agent-observable anyway.
- Separate proof harness from product: keep `proof:*` runners in a `proofs/` workspace or a shared dev-dependency tool, so the product's script surface is `dev` / `build` / `test` / `verify` plus a single `proofs` entry point. Add a CI check (the studio already has `check:boundaries`) that any command name cited in README/docs exists — the same discipline asha applies to contracts, applied to its own prose.
- For the new `asha-demo`, start with the product README and add evidence docs later — the inverted order is what happened here.

## Formalizing the TS stack on the rusty-view layer pattern

*Added after review of the Den convention doc `patch/rusty-view-ui-architecture-pattern` ("treat the frontend like backend infrastructure: typed layers, strict boundaries, generated contracts, testable domain logic, mechanical enforcement"). The pattern was distilled from `rusty-view` and already applied to `asha-studio` with good results. This section assesses applying it to the engine's `ts/` workspace and across the repo family.*

### Assessment: the pattern is ~70% present in `ts/` — the missing part is enforcement, not structure

First, the tooling relationship, since these are often conflated: **tsc** is the compiler (rustc), a **pnpm workspace** is a Cargo workspace, **Nx** is a task orchestrator plus dependency-graph linter layered on top (tooling, not runtime), and **Angular** is the only actual runtime framework of the four. They are not alternatives. Studio correctly uses all four because it is a browser app; the engine `ts/` correctly uses only pnpm+tsc because its packages are libraries — the pattern's own `protocol`/`transport`/`domain`/`testing-fixtures` layers are explicitly framework-free. **Do not adopt Nx or Angular in the engine repo.** The pattern's value here is its framework-free half: layer direction, boundary mechanics, barrels, generators, strict lint.

The engine's TS packages already implement the pattern's layer model in substance:

| Pattern layer | asha `ts/` package(s) |
|---|---|
| protocol | `contracts` (generated, barrel-exported — the pattern's ideal form) |
| transport | `runtime-bridge`, `native-bridge`, `wasm-replay-bridge` |
| domain | `editor-tools`, `command-registry`, `game-workspace`, `policy-*`, `script-sdk`, `script-host` |
| renderer | `renderer-three` |
| components | `ui-dom`, `devtools` |
| shell | `app`, `electron-main` |
| testing-fixtures | `smoke`, harness fixtures |

Compiler posture already nearly matches the pattern's required flags (`strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noImplicitOverride` are on; only `noImplicitReturns` and `noPropertyAccessFromIndexSignature` are missing). What's missing is the mechanical-enforcement half, which maps directly onto findings 1, 2, 4, and 5.

### The five moves, in order of leverage

**1. Generate lint boundaries from `ownership.toml` instead of hand-syncing three copies.** Boundary truth currently lives in three hand-maintained places: `governance/ownership.toml`, the regex-based depgraph verifiers, and `ts/eslint.config.mjs` (whose own comments acknowledge it duplicates the depgraph for faster local feedback). Apply the repo's proven contracts trick: generate the ESLint `no-restricted-imports` configuration from `ownership.toml`, with a `--check` drift mode in CI. This collapses three truths into one and is the natural vehicle for finding 1 — enforcing `may_depend_on` as allowlists is exactly Nx's `onlyDependOnLibsWithTags` semantics, implemented without Nx.

**2. Add the pattern's two-axis tag model to `ownership.toml`.** Lanes are already the `scope:` axis. Add a `type:` axis (`lib` / `shell` / `testing` / `tool`) and a `layer:` field carrying the directional rule (protocol → transport → domain → renderer/components → shell; lower layers never import higher). Record the mapping table above in `governance/architecture.md` as the definitive statement: the `ts/` workspace implements the rusty-view layer model, lanes are scopes. Agents in all repos then read one architecture instead of inferring it per-repo.

**3. Enforce public-API barrels workspace-wide.** The "no deep imports" rule currently exists only at the outer border (studio/asha-testing boundary policies forbid `src/*` paths) and for policy/catalog packages internally. Require an `exports` field in every package.json (several already have one) and extend the deep-import lint to all packages. This is also the precondition that makes finding 4's `runtime-bridge` split invisible to consumers — they only ever saw the barrel.

**4. Add a package scaffold script — the pattern's generators without Nx.** A `ts/scripts/new-package.ts` that creates the directory, tsconfig, `exports`-bearing package.json, test stub, **and the `ownership.toml` entry** in one step. The `game-workspace` governance hole (findings 2, 10) happened because hand-creating a package has no step that touches governance; a generator makes the correct path the lazy path.

**5. Enable the type-aware lint tier.** `parserOptions.project: true` is already set, so the pattern's heavier rules (`no-floating-promises`, `no-unsafe-*`, `explicit-function-return-type` for exported functions, `no-explicit-any`, `no-non-null-assertion`) are cheap to enable, plus the two missing compiler flags.

### Pattern-shaped resolutions to open questions

- **`runtime-bridge` monolith (finding 4):** in pattern terms, `GameRuntimeLauncher` is domain/orchestration logic living inside the transport layer. The split is the pattern applied, not extra work.
- **Renderer ownership (finding 10):** the pattern's renderer layer is "infrastructure behind a stable API, receiving projected data." The engine already owns projection (render diffs, decode); the open question is only whether retained-scene *application* becomes a public consumable. The pattern-consistent answer: promote a framework-free render-diff-application library (no Three.js types in its API) that studio's canvas and future demos bind to their own scene graphs — rather than either forking renderers per frontend or forcing `@asha/renderer-three` wholesale on consumers.
- **Studio boundary surface (finding 10):** studio should introduce local `protocol` and `transport` libs that wrap `@asha/contracts` and `@asha/runtime-bridge` behind studio-owned barrels, so feature code never imports `@asha/*` directly. The boundary policy then shrinks to "only these two libs may touch `@asha`."

## Priority order

| # | Finding | Effort | Payoff |
|---|---------|--------|--------|
| 1 | Enforce `may_depend_on` allowlists | Small (extend two scripts) | Closes the main drift channel the repo was built to prevent |
| 2 | TS ownership completeness + register `game-workspace` | Small | Same |
| 3 | Mechanically tie codegen IR to Rust protocol types | Medium → large | Removes the only second source of truth on the border |
| 4 | Split `runtime-bridge` index.ts; isolate mock backend | Medium | Safer fail-closed story, smaller agent task surfaces |
| 8 | Remove committed logs/pids, extend `.gitignore` | Trivial | Hygiene |
| 6 | Mark or remove placeholder crates | Small | Honest workspace map for orchestrators |
| 7 | Border-crate tests (`protocol-policy-view`, `protocol-telemetry`, `native-bridge`) | Small | Catches contract regressions at the source |
| 5 | Single-file-package convention | Small (convention) | Reduces agent merge collisions |
| 9 | Consolidate architecture doc entry points | Small | Prevents doc drift recurrence |
| 10 | Engine-declared public surface; consumers validate against it | Small–medium | Ends four-way allow-list divergence; gives the new `asha-demo` a contract to start from |
| 11 | Evict task journals from consumer manifests; separate proof harness from product scripts | Small | Product repos legible as products; stops prose/script drift |
| 12 | Rusty-view formalization moves 1–5 (generated lint boundaries, two-axis tags, barrels, scaffold script, type-aware lint) | Small–medium each | One boundary truth, one documented layer model, agent rails without adopting Nx/Angular in the engine |
