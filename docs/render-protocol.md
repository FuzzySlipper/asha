# Render protocol

## Model: retained-mode diffs

Rust does not send "everything to draw this frame."
It emits a compact diff of what changed:

```
create handle
update transform
replace mesh payload
set visibility
set material
destroy handle
emit debug overlay
```

The renderer maintains a scene graph keyed by handles. It applies each diff operation
to the relevant handle and lets Babylon.js manage the actual draw calls.

## Why retained-mode

- Traffic stays small — only changes travel across the boundary.
- Fixtures are simple — a diff batch is a short list of operations.
- Renderer tests are fixture-friendly — apply a diff, assert handle registry state.
- Agents have a bounded vocabulary to reason about.

## Protocol types

Defined in `engine-rs/crates/protocol/protocol-render`. Generated TypeScript lives in
`ts/packages/contracts/src/generated/render.ts`.

Key types:
- `RenderHandle` — opaque stable ID for a renderable object
- `RenderDiff` — one diff operation (create/update/destroy/overlay)
- `RenderDiffBatch` — ordered list of diffs for one tick
- `GeometryPayload` — descriptor for mesh data (references a memory handle for large buffers)
- `MaterialRef` — reference to a material by name/ID

## Large payloads

Large geometry or buffer data travels through bridge-owned memory views, not structured messages.

Rules:
- Structured `RenderDiff` carries small metadata only.
- Large buffers use stable bridge memory views referenced by pointer+length or handle.
- Renderer upload behavior is isolated inside `wasm-bridge` and `renderer-babylon`.
- No policy package may access raw WASM memory.

## Renderer boundary rule

The renderer consumes diffs. It does not:
- Inspect `StateStore`.
- Import policy packages.
- Submit authority commands except through approved UI/app paths.

## Debug overlays

`render-debug` emits debug overlay diffs (bounding boxes, nav-mesh visualization, labels).
These are non-authoritative and stripped in release builds.
They are defined in `protocol-render` as a distinct diff variant so the renderer can
opt them in or out without changing the core diff stream.
