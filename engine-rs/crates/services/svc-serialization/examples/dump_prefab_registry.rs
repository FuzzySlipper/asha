//! Regenerator for the valid prefab-registry golden fixture.
//!
//! `cargo run -p svc-serialization --example dump_prefab_registry` and redirect
//! into `harness/fixtures/project-bundle/prefab-registry.valid.json`.

use svc_serialization::encode_prefab_registry;

#[path = "../tests/support/prefab_fixtures.rs"]
mod prefab_fixtures;

fn main() {
    print!(
        "{}",
        encode_prefab_registry(&prefab_fixtures::valid_registry())
    );
}
