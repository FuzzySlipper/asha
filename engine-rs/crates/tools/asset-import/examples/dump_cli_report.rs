//! Dump the deterministic CLI dry-run report for the canonical fixture. The
//! committed golden in `harness/fixtures/asset-import/cli-report.golden` is this
//! output; the `cli_report_golden` test pins it. Regenerate with:
//!   cargo run -p asset-import --example dump_cli_report > \
//!     harness/fixtures/asset-import/cli-report.golden

use asset_import::cli::{plan, Mode};
use asset_import::fixtures;

fn main() {
    let first = plan(
        "import-fixture-a.mesh.json",
        fixtures::VALID_TRIANGLE,
        &Mode::DryRun,
        None,
    );
    print!("{}", first.report);
}
