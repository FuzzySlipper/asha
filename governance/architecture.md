# Architecture overview

See docs/design.md for the full design document.

Core split: Rust owns authority. TypeScript owns expression and projection.
Generated contracts define the border.

Layer order (lowest to highest): foundation → state → protocol → sim/services/rules → render/wasm/tools.

## Scene / world / asset foundation

- `core-assets` (foundation) — typed asset-reference vocabulary: `AssetKind`, validated
  kind-prefixed scoped-kebab-case `AssetId`, and typed `AssetRef<T>` / kind-erased
  `AssetReference`. Identity/validation only; the full catalog (resolution, DAG, locks,
  fallback) is deferred to the asset-registry work.
- `core-scene` (state) — authored scene documents and live world authority:
  - `SceneTree` (authoring/visualization) ⇄ `FlatSceneDocument` (canonical, flat, validated,
    deterministically serialized) with order-preserving round-trip.
  - `validate` — classified scene validation (duplicate/unknown-parent/cycle/transform/
    wrong-kind-asset-ref).
  - `WorldState` — live runtime authority; bootstrap copies initial transforms in, then
    runtime transforms are authority-owned and may diverge from the authored document.
  - `bootstrap` — atomic scene→authority initialization producing one `BootstrapRecord`
    replay unit with a deterministic world hash and a `scene node → entity` source trace.
- `SceneId` / `WorldId` / `SceneNodeId` live in `core-ids` and are distinct from
  `protocol-render::RenderHandle` (a derived projection handle, not authority).
- Authored scene documents and asset references are Rust-validated; no `protocol`/codegen
  border surface exists for them yet — it lands when scene/bootstrap shapes cross to TS.
