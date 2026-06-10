# Lane: ts-shell

## Owns
- `ts/packages/wasm-bridge` — WASM loading, memory views, protocol encode/decode, render diff stream
- `ts/packages/renderer-babylon` — Babylon.js scene, handle registry, geometry/material registries, diff application
- `ts/packages/ui-dom` — DOM panels, inspectors, command palette, state view-models
- `ts/packages/cosmetic` — non-authoritative particles, transient animation, screen effects
- `ts/packages/electron-main` — window/process/IPC/platform integration (main process only)
- `ts/packages/app` — runtime loop, wiring of render diffs, UI commands, policy host

## May import
- `@asha/contracts` in all packages
- `wasm-bridge` may import contracts only
- `renderer-babylon`, `ui-dom`, `cosmetic`, `app` may import `@asha/wasm-bridge`
- `app` may import `@asha/script-host`
- `electron-main` runs in its own process; it may not import runtime packages

## Must never import (policy boundary)
- `@asha/policy-core`, `@asha/policy-examples` directly into renderer or UI
- Policy packages may only reach the runtime through `app` → `script-host` wiring
- Renderer packages may not inspect `StateStore` — consume render diffs only

## Required tests
- `wasm-bridge`: WASM load + one command encode/decode round-trip.
- `renderer-babylon`: render diff fixture test — apply a diff batch, assert handle registry state.
- `ui-dom`: command palette emits correct command type on user action.
- `app`: runtime loop wiring test (headless, no renderer).

## Required fixtures
- `harness/fixtures/render-diffs/` — diff batches for renderer fixture tests.
- `harness/goldens/screenshots/` — headless screenshot goldens once renderer is active (Phase 5+).

## Drift smells reviewers should flag
- Renderer package importing a policy package.
- UI package maintaining a shadow copy of authoritative state.
- `app` accumulating feature logic instead of wiring.
- Electron main/preload gaining policy execution or product-domain logic.
- `cosmetic` package influencing replay truth or simulation output.
- `wasm-bridge` exposing raw WASM memory pointers to policy packages.

## Public API changes that require escalation
- Changes to the render diff stream API in `wasm-bridge` — affects renderer.
- Changes to Electron IPC surface — affects preload and renderer process boundary.
