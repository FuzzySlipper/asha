//! Deterministic source-mesh fixtures, shared by tests, goldens, and the CLI.
//!
//! Abstract fixture nouns only (no product content): `import-fixture-a`,
//! `surface-a`. These are the documented format's canonical examples.

/// A valid single-triangle mesh: one textured material, one group, visual-only.
pub const VALID_TRIANGLE: &str = r#"{
  "schemaVersion": 1,
  "name": "import-fixture-a",
  "positions": [0, 0, 0, 1, 0, 0, 0, 1, 0],
  "normals": [0, 0, 1, 0, 0, 1, 0, 0, 1],
  "indices": [0, 1, 2],
  "materials": [
    { "slot": 0, "name": "surface-a", "color": [0.8, 0.2, 0.1, 1], "texture": "surface-a" }
  ],
  "groups": [ { "materialSlot": 0, "start": 0, "count": 3 } ],
  "collision": "aabbFallback"
}"#;

/// A two-triangle quad with two material slots and two groups (untextured).
pub const VALID_QUAD: &str = r#"{
  "schemaVersion": 1,
  "name": "import-fixture-b",
  "positions": [0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0],
  "normals": [0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
  "indices": [0, 1, 2, 0, 2, 3],
  "materials": [
    { "slot": 0, "name": "surface-a", "color": [0.5, 0.5, 0.5, 1] },
    { "slot": 1, "name": "surface-b", "color": [0.1, 0.6, 0.2, 1] }
  ],
  "groups": [
    { "materialSlot": 0, "start": 0, "count": 3 },
    { "materialSlot": 1, "start": 3, "count": 3 }
  ],
  "collision": "visualOnly"
}"#;

/// Recognised-but-unsupported feature (`animations`) — rejected, not dropped.
pub const UNSUPPORTED_FEATURE: &str = r#"{
  "schemaVersion": 1,
  "name": "import-fixture-a",
  "positions": [0, 0, 0, 1, 0, 0, 0, 1, 0],
  "normals": [0, 0, 1, 0, 0, 1, 0, 0, 1],
  "indices": [0, 1, 2],
  "animations": [ { "name": "spin" } ]
}"#;

/// Non-triangle topology (index count not a multiple of 3).
pub const BAD_TOPOLOGY: &str = r#"{
  "schemaVersion": 1,
  "name": "import-fixture-a",
  "positions": [0, 0, 0, 1, 0, 0, 0, 1, 0],
  "normals": [0, 0, 1, 0, 0, 1, 0, 0, 1],
  "indices": [0, 1]
}"#;
