//! Inspectable offline-import CLI core (#2386).
//!
//! The CLI wraps the importer so agents and humans can run, inspect, and verify
//! imports consistently. It separates a **pure planning** stage (deterministic,
//! filesystem-free — used for the golden report and dry-run) from the **fs-driven**
//! run that reads the source and writes artifacts. Writes go to temp files that are
//! atomically renamed into place, so a failed import never leaves partial or corrupt
//! output. Output paths are deterministic and the report is stable and diffable.
//!
//! Non-goals (explicit): no runtime import, no broad DCC pipeline, no product asset
//! content. Diagnostics/artifacts are Den-agnostic plain text/JSON.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::artifacts::{render_artifacts, GeneratedArtifact};
use crate::import::ImportedAssets;
use crate::manifest::{build_manifest, plan_reimport, ImportManifest, ReimportPlan};
use crate::{import_text, ImportOutcome, IMPORTER_VERSION};

/// What the CLI was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Report what would be generated/changed; write nothing.
    DryRun,
    /// Write the artifacts (via temp + atomic rename).
    Write,
}

/// A planned import: the artifacts to write, the manifest, and a deterministic
/// human/agent-legible report. Filesystem-free, so it is fully testable + goldenable.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportPlan {
    pub report: String,
    pub artifacts: Vec<GeneratedArtifact>,
    pub manifest: Option<ImportManifest>,
    pub has_errors: bool,
}

fn base_name(assets: &ImportedAssets) -> String {
    assets
        .static_mesh
        .asset
        .strip_prefix("mesh/")
        .unwrap_or(&assets.static_mesh.asset)
        .to_string()
}

/// The deterministic output file names for a given imported base name.
pub fn output_paths(name: &str) -> Vec<String> {
    vec![
        format!("{name}.catalog.json"),
        format!("{name}.staticmesh.json"),
        format!("{name}.import.json"),
    ]
}

/// Plan an import from source text (no filesystem). `source_label` is the stable
/// path shown in the report and recorded in the manifest. `prior` is the previously
/// written manifest, if any, used to classify the reimport.
pub fn plan(
    source_label: &str,
    source_text: &str,
    mode: &Mode,
    prior: Option<&ImportManifest>,
) -> ImportPlan {
    let outcome: ImportOutcome = import_text(source_text, source_label);
    let mut report = String::new();
    report.push_str(&format!("asha-import: {}\n", source_label));
    report.push_str(&format!(
        "mode: {}\n",
        match mode {
            Mode::DryRun => "dry-run",
            Mode::Write => "write",
        }
    ));

    // Diagnostics are always reported, in order, classified.
    report.push_str(&format!("diagnostics: {}\n", outcome.diagnostics.len()));
    for d in &outcome.diagnostics {
        report.push_str(&format!("  {}\n", d.render()));
    }

    let Some(assets) = outcome.assets else {
        report.push_str("result: FAILED — no artifacts produced\n");
        return ImportPlan {
            report,
            artifacts: Vec::new(),
            manifest: None,
            has_errors: true,
        };
    };

    let name = base_name(&assets);
    let artifacts = render_artifacts(&name, &assets);
    let manifest = build_manifest(
        source_label,
        source_text,
        IMPORTER_VERSION,
        1,
        &assets.static_mesh.asset,
        &artifacts,
    );

    report.push_str(&format!("asset: {}\n", assets.static_mesh.asset));
    report.push_str(&format!(
        "sourceFingerprint: {}\n",
        manifest.source_fingerprint
    ));
    report.push_str(&format!("importerVersion: {}\n", manifest.importer_version));

    // Reimport classification against any prior manifest.
    let plan_label = match prior {
        Some(prior_manifest) => describe_reimport(&plan_reimport(prior_manifest, &manifest)),
        None => "firstImport".to_string(),
    };
    report.push_str(&format!("reimportPlan: {plan_label}\n"));

    report.push_str("artifacts:\n");
    for art in &artifacts {
        report.push_str(&format!(
            "  {} ({})\n",
            art.rel_path,
            crate::fingerprint::fingerprint_hex(art.contents.as_bytes())
        ));
    }
    report.push_str(&format!(
        "  {name}.import.json ({})\n",
        crate::fingerprint::fingerprint_hex(manifest.render().as_bytes())
    ));

    report.push_str(&format!(
        "result: {}\n",
        match mode {
            Mode::DryRun => "OK (dry-run, nothing written)",
            Mode::Write => "OK (written)",
        }
    ));

    ImportPlan {
        report,
        artifacts,
        manifest: Some(manifest),
        has_errors: false,
    }
}

fn describe_reimport(plan: &ReimportPlan) -> String {
    match plan {
        ReimportPlan::Noop => "noop (unchanged)".to_string(),
        ReimportPlan::VisualUpdate { changed } => {
            format!("visualUpdate ({} changed)", changed.len())
        }
        ReimportPlan::StructuralReload { reason, .. } => format!("structuralReload ({reason})"),
    }
}

/// The full set of files a plan would write (artifacts + manifest), by name.
fn plan_files(name: &str, plan: &ImportPlan) -> Vec<GeneratedArtifact> {
    let mut files = plan.artifacts.clone();
    if let Some(manifest) = &plan.manifest {
        files.push(GeneratedArtifact {
            rel_path: format!("{name}.import.json"),
            contents: manifest.render(),
        });
    }
    files
}

/// Read the prior manifest's raw bytes from the output dir, if present, to classify
/// a reimport. We only need its fingerprints, so a minimal re-parse is used.
fn read_prior_manifest(out_dir: &Path, name: &str) -> Option<ImportManifest> {
    let path = out_dir.join(format!("{name}.import.json"));
    let text = fs::read_to_string(path).ok()?;
    crate::manifest::parse_manifest(&text)
}

/// Run an import end to end against the filesystem. Reads `source_path`, plans, and
/// (in write mode) writes every artifact to a `.tmp` sibling then atomically renames
/// it into place — so a mid-write failure leaves the prior output intact.
pub fn run(source_path: &Path, out_dir: &Path, mode: Mode) -> io::Result<ImportPlan> {
    let source_text = fs::read_to_string(source_path)?;
    let source_label = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("source.mesh.json")
        .to_string();

    // Determine the base name (and thus the prior manifest) without committing.
    let provisional = import_text(&source_text, &source_label);
    let prior = provisional
        .assets
        .as_ref()
        .map(base_name)
        .and_then(|name| read_prior_manifest(out_dir, &name));

    let plan = plan(&source_label, &source_text, &mode, prior.as_ref());

    if plan.has_errors || mode == Mode::DryRun {
        return Ok(plan);
    }

    // Write mode: stage to temp files, then rename into place (atomic per file).
    let name = provisional
        .assets
        .as_ref()
        .map(base_name)
        .expect("assets present on success");
    fs::create_dir_all(out_dir)?;
    let files = plan_files(&name, &plan);

    let mut staged: Vec<(PathBuf, PathBuf)> = Vec::new();
    for file in &files {
        let final_path = out_dir.join(&file.rel_path);
        let tmp_path = out_dir.join(format!("{}.tmp", file.rel_path));
        fs::write(&tmp_path, &file.contents)?;
        staged.push((tmp_path, final_path));
    }
    // All temp files written successfully — now swap them in.
    for (tmp, final_path) in staged {
        fs::rename(tmp, final_path)?;
    }

    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn plan_is_deterministic_and_filesystem_free() {
        let a = plan(
            "import-fixture-a.mesh.json",
            fixtures::VALID_TRIANGLE,
            &Mode::DryRun,
            None,
        );
        let b = plan(
            "import-fixture-a.mesh.json",
            fixtures::VALID_TRIANGLE,
            &Mode::DryRun,
            None,
        );
        assert_eq!(a, b);
        assert!(!a.has_errors);
        assert!(a.report.contains("reimportPlan: firstImport"));
    }

    #[test]
    fn a_failed_import_plans_no_artifacts() {
        let p = plan("bad.mesh.json", fixtures::BAD_TOPOLOGY, &Mode::DryRun, None);
        assert!(p.has_errors);
        assert!(p.artifacts.is_empty());
        assert!(p.report.contains("result: FAILED"));
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("asha-import-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn write_mode_emits_the_deterministic_files() {
        let dir = temp_dir("write");
        let src = dir.join("import-fixture-a.mesh.json");
        fs::write(&src, fixtures::VALID_TRIANGLE).unwrap();
        let out = dir.join("out");

        let plan = run(&src, &out, Mode::Write).unwrap();
        assert!(!plan.has_errors);
        for name in output_paths("import-fixture-a") {
            assert!(out.join(&name).exists(), "missing {name}");
        }
        // No leftover temp files.
        let leftovers: Vec<_> = fs::read_dir(&out)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "staging temp files were not swapped");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dry_run_writes_nothing() {
        let dir = temp_dir("dryrun");
        let src = dir.join("import-fixture-a.mesh.json");
        fs::write(&src, fixtures::VALID_TRIANGLE).unwrap();
        let out = dir.join("out");

        run(&src, &out, Mode::DryRun).unwrap();
        assert!(!out.exists() || fs::read_dir(&out).unwrap().next().is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reimport_of_unchanged_source_reports_noop() {
        let dir = temp_dir("noop");
        let src = dir.join("import-fixture-a.mesh.json");
        fs::write(&src, fixtures::VALID_TRIANGLE).unwrap();
        let out = dir.join("out");

        run(&src, &out, Mode::Write).unwrap();
        let second = run(&src, &out, Mode::DryRun).unwrap();
        assert!(second.report.contains("reimportPlan: noop"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_failed_import_writes_no_files() {
        let dir = temp_dir("fail");
        let src = dir.join("bad.mesh.json");
        fs::write(&src, fixtures::BAD_TOPOLOGY).unwrap();
        let out = dir.join("out");

        let plan = run(&src, &out, Mode::Write).unwrap();
        assert!(plan.has_errors);
        assert!(!out.exists() || fs::read_dir(&out).unwrap().next().is_none());

        let _ = fs::remove_dir_all(&dir);
    }
}
