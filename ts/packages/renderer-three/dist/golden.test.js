// Render golden check: apply a named render-diff fixture to the Three.js
// renderer and diff its deterministic scene snapshot against a committed golden.
// Run headlessly (no GL context). Driven by `harness/ci/check-render-goldens.sh`.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { ThreeRenderer } from './index.js';
const repoRoot = resolve(import.meta.dirname, '../../../..');
function loadFixture(name) {
    return JSON.parse(readFileSync(resolve(repoRoot, 'harness/fixtures/render-diffs', `${name}.json`), 'utf8'));
}
function loadGolden(name) {
    return readFileSync(resolve(repoRoot, 'harness/goldens/render-diffs', `${name}.snapshot`), 'utf8');
}
test('scene-showcase fixture renders to the committed golden snapshot', () => {
    const renderer = new ThreeRenderer();
    try {
        renderer.applyEncodedFrame(loadFixture('scene-showcase'));
    }
    catch (e) {
        assert.fail(`RENDERER FAILURE while applying scene-showcase: ${String(e)}`);
    }
    const actual = renderer.snapshot();
    const golden = loadGolden('scene-showcase');
    assert.equal(actual, golden, 'GOLDEN MISMATCH: rendered scene drifted from ' +
        'harness/goldens/render-diffs/scene-showcase.snapshot — regenerate if intended.');
});
//# sourceMappingURL=golden.test.js.map