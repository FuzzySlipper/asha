//! Regenerator for the compacted-save bundle-section golden fixture.
//!
//! `cargo run -p rule-project-bundle --example dump_compacted_save` and redirect into
//! `harness/fixtures/project-bundle/compacted-save.txt`.

#[path = "../tests/support/render.rs"]
mod render;

fn main() {
    print!(
        "{}",
        render::render_compacted_save(&render::sample_compacted_save())
    );
}
