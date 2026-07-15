# Render snapshot goldens (structural)

Each `<name>.snapshot` is the deterministic, text **structural** snapshot of the
Three.js scene after `@asha/renderer-three` applies the matching
`harness/fixtures/render-diffs/<name>.json` fixture. It captures per-handle layer,
shape/asset, transform, visibility, material colours, and sprite framing — built
without a GL context, so it is a pure data snapshot, not pixels.

`static-room.snapshot` is the #4029 upstream demo evidence artifact for the
synthetic enclosed-room render path. It proves retained Three.js scene structure,
not gameplay motion, collision, runtime attachment, or a browser screenshot.

Consumer: `ts/packages/renderer-three/src/golden.test.ts` (run via
`harness/ci/check-render-goldens.sh`).

## Regenerate

These snapshots are committed and compared by string equality. When the renderer's
structural output changes intentionally:

1. If the *input* fixture is Rust-generated, re-bless it first (see the
   `harness/fixtures/render-diffs/` README).
2. Re-run the snapshot test; update the `<name>.snapshot` file to the new
   `renderer.snapshot()` output shown in the mismatch, and review the diff.

```bash
cd ts && pnpm --filter @asha/renderer-three test
```

## Browser captures are evidence, not pixel gates

These structural snapshots are the **current** render gate. True pixel/screenshot
goldens (a real WebGL/offscreen render) remain deferred as a CI gate. Explicit
human-visible captures may live in `harness/goldens/screenshots/`; see its README
for their source and reproduction command. Structural snapshots remain the
deterministic GL-free regression gate.
