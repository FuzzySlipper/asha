import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import assert from 'node:assert/strict';

import { parseAshaGameManifestToml, type AshaGameManifest } from './manifest.js';
import type { AshaGameAssetCatalog } from './assets.js';

const fixturesRoot = resolve(import.meta.dirname, '../src/fixtures');

export function fixture(name: string): string {
  return readFileSync(resolve(fixturesRoot, name), 'utf8');
}

export function validManifest(): AshaGameManifest {
  const result = parseAshaGameManifestToml(fixture('asha.game.toml'));
  assert.equal(result.ok, true);
  if (!result.ok) {
    throw new Error('golden manifest should validate');
  }
  return result.manifest;
}

export function validCatalog(): AshaGameAssetCatalog {
  return {
    schemaVersion: 1,
    entries: [
      {
        id: 'mesh.demo-cube',
        kind: 'static_mesh',
        source: 'assets/meshes/demo-cube.mesh.json',
        importProfile: 'inline-static-mesh.v0',
        importMetadata: {
          sourceHash: 'sha256:mesh',
          cacheKey: 'dev-cache/static_mesh/mesh.demo-cube/sha256-mesh',
          generatedArtifactVersion: 'asset-import.v1',
        },
        dependencies: ['material.demo-copper'],
        publish: { include: true, outputKey: 'meshes/demo-cube.mesh.json' },
        diagnostics: { owner: 'asha-demo', notes: [] },
      },
      {
        id: 'material.demo-copper',
        kind: 'material',
        source: 'assets/materials/demo-copper.material.json',
        importProfile: 'inline-material.v0',
        importMetadata: {
          sourceHash: 'sha256:material',
          cacheKey: 'dev-cache/material/material.demo-copper/sha256-material',
          generatedArtifactVersion: 'asset-import.v1',
        },
        dependencies: ['texture.demo-checker'],
        publish: { include: true, outputKey: 'materials/demo-copper.material.json' },
        diagnostics: { owner: 'asha-demo', notes: [] },
      },
      {
        id: 'texture.demo-checker',
        kind: 'texture',
        source: 'assets/textures/demo-checker.texture.json',
        importProfile: 'inline-texture.v0',
        importMetadata: {
          sourceHash: 'sha256:texture',
          cacheKey: 'dev-cache/texture/texture.demo-checker/sha256-texture',
          generatedArtifactVersion: 'asset-import.v1',
        },
        dependencies: [],
        publish: { include: true, outputKey: 'textures/demo-checker.texture.json' },
        diagnostics: { owner: 'asha-demo', notes: [] },
      },
    ],
  };
}
