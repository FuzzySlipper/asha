# asset-import — offline ASHA-native asset importer

A deterministic, **offline** tool that converts a documented source-mesh format
into ASHA-native static-mesh + catalog + texture descriptors, with source
fingerprints, classified diagnostics, asset-lock drift detection, and a dry-run /
write CLI.

**Non-goals (explicit):** no runtime import, no broad DCC pipeline, no glTF runtime
loading, no product asset content. The runtime renderer consumes only the
ASHA-native descriptors this emits (mesh provenance is `staticAsset`). Catalog
validation stays Rust-owned in `core-catalog`.

## Supported source format (schema version 1)

The first-pass importer reads ASHA's own small JSON **source-mesh** format
(`*.mesh.json`) — chosen over a glTF/glb binary subset so the import is
dependency-free and trivially deterministic. A glTF-subset front-end can later
target this same importer core.

```json
{
  "schemaVersion": 1,
  "name": "import-fixture-a",          // base name; importer emits mesh/<name>
  "positions": [x, y, z, ...],         // 3 floats per vertex
  "normals":   [x, y, z, ...],         // 3 floats per vertex (same vertex count)
  "indices":   [i, ...],               // triangle list; length a multiple of 3
  "materials": [
    { "slot": 0, "name": "surface-a", "color": [r, g, b, a], "texture": "surface-a" }
  ],
  "groups": [ { "materialSlot": 0, "start": 0, "count": 3 } ],
  "collision": "visualOnly"            // "visualOnly" | "aabbFallback" | { "proxy": "id" }
}
```

`materials`, `groups`, and `collision` are optional (defaulting to one flat slot,
one group over all indices, and `visualOnly`).

### Rejected features (classified, never silently dropped)

Triangle lists only; separate position/normal streams only; no UV/colour/joint
vertex attributes; no `animations`, `skins`, `morphTargets`, `cameras`, or
`lights`. Any of those is reported as a classified `ImportDiagnostic` with a source
locus and a suggested remedy. A missing external texture and a changed source
fingerprint are likewise classified.

## CLI

```bash
# Write artifacts into <dir> (staged to temp files, then atomically renamed in):
cargo run -p asset-import --bin asha-import -- path/to/foo.mesh.json --out out/dir

# Dry-run: report what would be generated and how a reimport classifies; writes nothing:
cargo run -p asset-import --bin asha-import -- path/to/foo.mesh.json --out out/dir --dry-run
```

Output paths are deterministic: `<name>.catalog.json`, `<name>.staticmesh.json`,
`<name>.import.json`. A failed import writes nothing (no partial/corrupt output). A
reimport is classified `noop` / `visualUpdate` / `structuralReload` against the
prior `<name>.import.json`.

## Regenerating the golden fixtures

The committed goldens under `harness/fixtures/asset-import/` are produced by the
crate's examples and pinned by `tests/*_golden.rs`:

```bash
cargo run -p asset-import --example dump_import             > harness/fixtures/asset-import/imported.golden
cargo run -p asset-import --example dump_import_diagnostics > harness/fixtures/asset-import/diagnostics.golden
cargo run -p asset-import --example dump_cli_report         > harness/fixtures/asset-import/cli-report.golden
```
