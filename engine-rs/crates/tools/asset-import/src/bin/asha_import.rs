//! `asha-import` — the offline asset-import CLI (#2386).
//!
//! Usage:
//!   asha-import <source.mesh.json> --out <dir> [--dry-run]
//!
//! Imports a documented source-mesh file into ASHA-native static-mesh + catalog +
//! manifest artifacts under `<dir>`. `--dry-run` reports what would be generated and
//! how a reimport would be classified, writing nothing. The structured report is
//! printed to stdout; the exit code is 0 on success, 1 on import error or bad usage.
//!
//! Non-goals: no runtime import, no DCC pipeline, no product asset content.

use std::path::PathBuf;
use std::process::ExitCode;

use asset_import::cli::{run, Mode};

fn usage() -> String {
    "usage: asha-import <source.mesh.json> --out <dir> [--dry-run]".to_string()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut source: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut mode = Mode::Write;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => mode = Mode::DryRun,
            "--out" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out = Some(PathBuf::from(p)),
                    None => {
                        eprintln!("error: --out requires a directory\n{}", usage());
                        return ExitCode::FAILURE;
                    }
                }
            }
            other if !other.starts_with("--") && source.is_none() => {
                source = Some(PathBuf::from(other));
            }
            other => {
                eprintln!("error: unexpected argument `{other}`\n{}", usage());
                return ExitCode::FAILURE;
            }
        }
        i += 1;
    }

    let (Some(source), Some(out)) = (source, out) else {
        eprintln!(
            "error: a source file and --out <dir> are required\n{}",
            usage()
        );
        return ExitCode::FAILURE;
    };

    match run(&source, &out, mode) {
        Ok(plan) => {
            print!("{}", plan.report);
            if plan.has_errors {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
