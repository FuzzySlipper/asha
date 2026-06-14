//! Integration: imported assets participate in asset-lock drift detection and the
//! reimport plan reacts to the kind of change (#2385). Source fingerprints, importer
//! version, and generated-artifact hashes are all carried by the manifest so a lock
//! can detect drift; a reimport is classified, never a silent overwrite.

use asset_import::{
    artifacts, import_text, manifest, ImportManifest, ReimportPlan, IMPORTER_VERSION,
};

const BASE: &str = r#"{
  "schemaVersion": 1,
  "name": "import-fixture-a",
  "positions": [0, 0, 0, 1, 0, 0, 0, 1, 0],
  "normals": [0, 0, 1, 0, 0, 1, 0, 0, 1],
  "indices": [0, 1, 2],
  "materials": [ { "slot": 0, "name": "surface-a", "color": [0.5, 0.5, 0.5, 1] } ],
  "groups": [ { "materialSlot": 0, "start": 0, "count": 3 } ],
  "collision": "visualOnly"
}"#;

fn manifest_for(source: &str) -> ImportManifest {
    let assets = import_text(source, "fixtures/import-fixture-a.mesh.json")
        .assets
        .expect("imports");
    let arts = artifacts::render_artifacts("import-fixture-a", &assets);
    manifest::build_manifest(
        "fixtures/import-fixture-a.mesh.json",
        source,
        IMPORTER_VERSION,
        1,
        &assets.static_mesh.asset,
        &arts,
    )
}

#[test]
fn manifest_records_source_fingerprint_and_artifact_hashes() {
    let m = manifest_for(BASE);
    assert_eq!(m.source_fingerprint.len(), 16);
    assert_eq!(m.importer_version, IMPORTER_VERSION);
    assert!(m
        .artifact_hash("import-fixture-a.staticmesh.json")
        .is_some());
    assert!(m.artifact_hash("import-fixture-a.catalog.json").is_some());
}

#[test]
fn unchanged_source_reimports_as_a_noop() {
    let prior = manifest_for(BASE);
    let next = manifest_for(BASE);
    assert_eq!(manifest::plan_reimport(&prior, &next), ReimportPlan::Noop);
    // And the lock sees no source drift.
    assert!(manifest::detect_source_drift(
        &prior.source_fingerprint,
        &next.source_fingerprint,
        &next.mesh_asset_id
    )
    .is_none());
}

#[test]
fn a_material_colour_change_is_a_visual_update_and_drifts_the_lock() {
    let prior = manifest_for(BASE);
    let recoloured = BASE.replace("[0.5, 0.5, 0.5, 1]", "[0.9, 0.1, 0.1, 1]");
    let next = manifest_for(&recoloured);
    assert!(matches!(
        manifest::plan_reimport(&prior, &next),
        ReimportPlan::VisualUpdate { .. }
    ));
    // The lock detects the source drift (it must not silently re-pin).
    let drift = manifest::detect_source_drift(
        &prior.source_fingerprint,
        &next.source_fingerprint,
        &next.mesh_asset_id,
    );
    assert!(drift.is_some());
}

#[test]
fn a_geometry_change_requires_a_structural_reload() {
    let prior = manifest_for(BASE);
    let moved = BASE.replace("[0, 0, 0, 1, 0, 0, 0, 1, 0]", "[0, 0, 0, 2, 0, 0, 0, 2, 0]");
    let next = manifest_for(&moved);
    assert!(matches!(
        manifest::plan_reimport(&prior, &next),
        ReimportPlan::StructuralReload { .. }
    ));
}
