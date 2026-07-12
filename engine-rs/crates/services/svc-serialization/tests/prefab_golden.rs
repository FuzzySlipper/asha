//! Golden-fixture drift, atomic load, and classified rejection for prefab registries.

use std::path::PathBuf;

use svc_serialization::{
    encode_prefab_registry, load_prefab_registry, PrefabDiagnosticCode, PrefabRegistryLoadError,
};

#[path = "support/prefab_fixtures.rs"]
mod prefab_fixtures;

fn dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .find(|ancestor| ancestor.join("engine-rs").is_dir() && ancestor.join("harness").is_dir())
        .expect("repo root")
        .join("harness/fixtures/project-bundle")
}

#[test]
fn valid_registry_encoding_matches_committed_golden() {
    let committed = std::fs::read_to_string(dir().join("prefab-registry.valid.json"))
        .expect("read valid prefab registry");
    assert_eq!(
        encode_prefab_registry(&prefab_fixtures::valid_registry()),
        committed,
        "prefab registry encoding drifted; regenerate with \
         `cargo run -p svc-serialization --example dump_prefab_registry`"
    );
}

#[test]
fn committed_valid_registry_loads_and_is_a_fixed_point() {
    let committed = std::fs::read_to_string(dir().join("prefab-registry.valid.json"))
        .expect("read valid prefab registry");
    let accepted = load_prefab_registry(&committed, &prefab_fixtures::context())
        .expect("valid golden loads atomically");
    assert_eq!(encode_prefab_registry(&accepted), committed);
}

#[test]
fn committed_invalid_registry_is_classified_and_never_accepted() {
    let committed =
        std::fs::read_to_string(dir().join("prefab-registry.invalid-alias-removal.json"))
            .expect("read invalid prefab registry");
    let error = load_prefab_registry(&committed, &prefab_fixtures::context()).unwrap_err();
    let PrefabRegistryLoadError::Validation(report) = error else {
        panic!("invalid semantic fixture must reach validation");
    };
    let codes = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(codes.contains(&PrefabDiagnosticCode::UnsafePartRemoval));
    assert!(codes.contains(&PrefabDiagnosticCode::DeletedRoleReferenced));
}
