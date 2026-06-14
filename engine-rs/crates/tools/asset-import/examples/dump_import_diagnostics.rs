//! Dump the deterministic classified diagnostics for representative bad/edge import
//! scenarios. The committed golden in
//! `harness/fixtures/asset-import/diagnostics.golden` is this output; the
//! `import_diagnostics_golden` test pins it. Regenerate with:
//!   cargo run -p asset-import --example dump_import_diagnostics > \
//!     harness/fixtures/asset-import/diagnostics.golden

use asset_import::{fingerprint, fixtures, import_text, manifest, parse_source, ImportContext};

fn section(label: &str, out: &mut String) {
    out.push_str(&format!("=== {label} ===\n"));
}

fn dump_diagnostics(diags: &[asset_import::ImportDiagnostic], out: &mut String) {
    if diags.is_empty() {
        out.push_str("(none)\n");
    }
    for d in diags {
        out.push_str(&d.render());
        out.push('\n');
    }
}

fn main() {
    let mut out = String::new();

    section("unsupported feature", &mut out);
    dump_diagnostics(
        &import_text(
            fixtures::UNSUPPORTED_FEATURE,
            "fixtures/unsupported.mesh.json",
        )
        .diagnostics,
        &mut out,
    );

    section("non-triangle topology", &mut out);
    dump_diagnostics(
        &import_text(fixtures::BAD_TOPOLOGY, "fixtures/bad-topology.mesh.json").diagnostics,
        &mut out,
    );

    section("missing external texture", &mut out);
    let parsed = parse_source(
        fixtures::VALID_TRIANGLE,
        "fixtures/import-fixture-a.mesh.json",
    )
    .mesh
    .expect("triangle parses");
    // The context resolves no textures, so the referenced texture is reported missing.
    let outcome = asset_import::import_with_context(
        &parsed,
        &ImportContext::with_textures(Vec::<String>::new()),
    );
    dump_diagnostics(&outcome.diagnostics, &mut out);

    section("source fingerprint drift vs asset lock", &mut out);
    let locked = fingerprint::fingerprint_hex(fixtures::VALID_TRIANGLE.as_bytes());
    let changed_source = fixtures::VALID_TRIANGLE.replace("0.8", "0.9");
    let current = fingerprint::fingerprint_hex(changed_source.as_bytes());
    match manifest::detect_source_drift(&locked, &current, "mesh/import-fixture-a") {
        Some(d) => out.push_str(&format!("{}\n", d.render())),
        None => out.push_str("(no drift)\n"),
    }

    print!("{out}");
}
