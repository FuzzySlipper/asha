//! Dump the deterministic imported artifacts + manifest for the canonical source
//! fixtures. The committed golden in `harness/fixtures/asset-import/imported.golden`
//! is this output; the `imported_artifacts_golden` test pins it. Regenerate with:
//!   cargo run -p asset-import --example dump_import > \
//!     harness/fixtures/asset-import/imported.golden

use asset_import::{artifacts, fixtures, import_text, manifest, IMPORTER_VERSION};

fn dump_one(label: &str, source_path: &str, source: &str, out: &mut String) {
    let outcome = import_text(source, source_path);
    let assets = outcome.assets.expect("fixture imports cleanly");
    let name = assets
        .static_mesh
        .asset
        .strip_prefix("mesh/")
        .unwrap_or(&assets.static_mesh.asset)
        .to_string();
    let arts = artifacts::render_artifacts(&name, &assets);

    out.push_str(&format!("=== {label} ===\n"));
    for art in &arts {
        out.push_str(&format!("--- {} ---\n", art.rel_path));
        out.push_str(&art.contents);
    }
    let m = manifest::build_manifest(
        source_path,
        source,
        IMPORTER_VERSION,
        1,
        &assets.static_mesh.asset,
        &arts,
    );
    out.push_str(&format!("--- {name}.import.json ---\n"));
    out.push_str(&m.render());
    out.push('\n');
}

fn main() {
    let mut out = String::new();
    dump_one(
        "triangle (textured, aabb collision)",
        "fixtures/import-fixture-a.mesh.json",
        fixtures::VALID_TRIANGLE,
        &mut out,
    );
    dump_one(
        "quad (two material slots)",
        "fixtures/import-fixture-b.mesh.json",
        fixtures::VALID_QUAD,
        &mut out,
    );
    print!("{out}");
}
