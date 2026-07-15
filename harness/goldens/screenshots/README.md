# Browser screenshot artifacts

Pixel-diff gating remains deferred because GPU/driver/anti-aliasing differences
need a tolerance strategy. This directory may nevertheless contain explicitly
requested, inspectable browser captures that demonstrate a human-visible outcome.

The current render gate is the **structural** snapshot in
`harness/goldens/render-diffs/` (per-handle scene-graph state, built without a GL
context). It is deterministic and CI-friendly; image diffs would add flakiness
(GPU/driver/AA differences) without catching more *logic* regressions than the
structural snapshot already does.

`lit-voxel-showcase.png` is the #5833 capture of the renderer-neutral ambient,
directional, point, and spot light path applied to uploaded meshes with normals.
Its source is `harness/fixtures/browser/lit-voxel-showcase.html`. It is review
evidence, not a byte-for-byte CI assertion.

Capture it from the repository root with a local static server and Chromium:

```bash
python3 -m http.server 38133 --bind 127.0.0.1
chromium --headless --no-sandbox --enable-unsafe-swiftshader \
  --use-gl=angle --use-angle=swiftshader --hide-scrollbars \
  --window-size=1000,600 --virtual-time-budget=4000 \
  --screenshot="$PWD/harness/goldens/screenshots/lit-voxel-showcase.png" \
  http://127.0.0.1:38133/harness/fixtures/browser/lit-voxel-showcase.html
```
