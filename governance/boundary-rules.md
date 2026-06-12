# Boundary rules

1. TypeScript may never mutate authoritative state.
2. Policy code receives generated read-only views; it returns proposed commands only.
3. Rust validates all commands. TypeScript does not validate.
4. Generated contract files in ts/packages/contracts/src/generated/ are not hand-edited.
5. No lower-level Rust crate may depend on a higher-level crate.
6. Policy/catalog packages may not import renderer, UI, WASM bridge, or Electron packages.
7. Renderer packages may not import policy packages.
8. Tool omniscience must not leak into runtime packages.
9. App/UI/renderer/devtools couple only to the `@asha/runtime-bridge` facade for runtime, not
   to the native addon (`@asha/native-bridge`) or the WASM replay path
   (`@asha/wasm-replay-bridge`). Only the facade imports the native addon. (ADR 0006)
10. `napi-rs` is the runtime transport; WASM is the replay/golden verification target. Neither
    is a public interface. Generated contracts remain the semantic/governance border.
11. Scene documents describe an *authored* initial arrangement; the live Rust `WorldState`
    (`core-scene`) owns runtime truth after bootstrap. An authored `SceneDocument` /
    `FlatSceneDocument` is never runtime authority and is never mutated by runtime movement.
    Scene bootstrap is one atomic authority initialization, not N ordinary create commands.
12. Render handles and the render scene graph are derived projection, never durable/save
    authority. Authority identity is `SceneNodeId` / `EntityId` (`core-ids`); a `RenderHandle`
    must not be treated as authority, save-file truth, or a stable durable id. Renderer/UI/
    devtools packages consume scene/world projections — they may not treat scene documents or
    render handles as authority.
13. Asset references that enter scene/save authority use the typed `AssetRef<T>` vocabulary
    (`core-assets`) with kind-prefixed scoped-kebab-case `AssetId`s — never free strings or
    source paths. Asset catalogs may be TS-authored, but Rust validates asset identity, kind,
    and references before authority accepts them; catalogs do not bypass Rust validation.
