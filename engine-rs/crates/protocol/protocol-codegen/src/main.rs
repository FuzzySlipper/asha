//! Command-line entrypoint for the ASHA protocol contract generator.
//!
//! Usage:
//!   protocol-codegen            Write generated contracts into the repo.
//!   protocol-codegen --check    Verify committed contracts are up to date
//!                               (exit 1 with a source-pointing message on drift).
//!
//! `--check` is what `harness/ci/check-contracts.sh` invokes so a stale or
//! hand-edited generated file fails CI with a precise pointer rather than a
//! generic diff.

use std::path::Path;
use std::process::ExitCode;

use protocol_codegen::{check_against, generated_files, repo_root};

fn main() -> ExitCode {
    let check = std::env::args().skip(1).any(|a| a == "--check");
    let root = repo_root();

    if check {
        run_check(&root)
    } else {
        run_generate(&root)
    }
}

fn run_generate(root: &Path) -> ExitCode {
    for file in generated_files() {
        let path = root.join(&file.rel_path);
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("error: could not create {}: {e}", parent.display());
                return ExitCode::FAILURE;
            }
        }
        if let Err(e) = std::fs::write(&path, &file.contents) {
            eprintln!("error: could not write {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        println!("wrote {}", file.rel_path);
    }
    ExitCode::SUCCESS
}

fn run_check(root: &Path) -> ExitCode {
    let drifts = check_against(root);
    if drifts.is_empty() {
        println!("protocol contracts are up to date.");
        ExitCode::SUCCESS
    } else {
        eprintln!("protocol contract drift detected:");
        for drift in &drifts {
            eprintln!("  - {}", drift.describe());
        }
        ExitCode::FAILURE
    }
}
