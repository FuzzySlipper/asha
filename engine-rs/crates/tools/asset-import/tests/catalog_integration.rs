//! Integration: the imported catalog passes Rust-owned catalog validation, and the
//! imported static mesh passes border validation (#2384). Catalog validation stays
//! Rust-owned — the importer only produces descriptors the validator accepts.

use asset_import::{fixtures, import_text};
use core_catalog::validate;

#[test]
fn imported_catalog_passes_rust_catalog_validation() {
    let outcome = import_text(fixtures::VALID_QUAD, "fixtures/import-fixture-b.mesh.json");
    let assets = outcome.assets.expect("fixture imports cleanly");
    let report = validate(&assets.catalog);
    assert!(
        report.errors.is_empty(),
        "imported catalog should validate, got {:?}",
        report.errors
    );
}

#[test]
fn imported_static_mesh_passes_border_validation() {
    let outcome = import_text(
        fixtures::VALID_TRIANGLE,
        "fixtures/import-fixture-a.mesh.json",
    );
    let assets = outcome.assets.expect("fixture imports cleanly");
    assert!(assets.static_mesh.validate().is_ok());
    // Provenance proves the runtime consumes ASHA-native output, not glTF.
    assert_eq!(
        assets.static_mesh.payload.provenance,
        protocol_render::MeshProvenance::StaticAsset
    );
}
