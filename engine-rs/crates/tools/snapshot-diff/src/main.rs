//! `snapshot-diff` — structured replay/snapshot-checkpoint comparison.
//!
//! The current committed ASHA snapshot artifact is replay metadata: each replay
//! records commands, outcomes, post-step state hashes, and `core-snapshot`
//! checkpoint metadata. This tool compares decoded replay records instead of raw
//! text so formatting noise does not hide authority drift.
//!
//! Commands:
//!   snapshot-diff replay <expected.replay> <actual.replay> [--json]
//!   snapshot-diff --help
//!
//! Exit codes: 0 = match, 1 = semantic mismatch, 2 = malformed/read error,
//! 3 = usage error.

use std::io::Write;
use std::process::ExitCode;

use sim_replay::{decode, ReplayHash, ReplayRecord, SnapshotMeta, StepOutcome};

const USAGE: &str = "\
snapshot-diff — compare ASHA authority replay/snapshot checkpoint artifacts

USAGE:
    snapshot-diff replay <expected.replay> <actual.replay> [--json]
    snapshot-diff --help

COMMANDS:
    replay    Decode both replay artifacts and compare their semantic record:
              format version, initial hash, command steps, accepted/rejected
              outcomes, post-step hashes, and snapshot checkpoint metadata.

OPTIONS:
    --json    Print mismatch reports as one compact JSON object to stdout.

EXIT CODES:
    0 match
    1 semantic mismatch
    2 malformed artifact or read error
    3 usage error
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = run(&args, &mut std::io::stdout(), &mut std::io::stderr());
    ExitCode::from(code)
}

fn run<O: Write, E: Write>(args: &[String], out: &mut O, err: &mut E) -> u8 {
    match args.first().map(String::as_str) {
        None | Some("--help") | Some("-h") | Some("help") => {
            let _ = write!(out, "{USAGE}");
            if args.is_empty() {
                3
            } else {
                0
            }
        }
        Some("replay") => cmd_replay(&args[1..], out, err),
        Some(other) => {
            let _ = writeln!(err, "error: unknown command '{other}'\n");
            let _ = write!(err, "{USAGE}");
            3
        }
    }
}

fn cmd_replay<O: Write, E: Write>(args: &[String], out: &mut O, err: &mut E) -> u8 {
    let parsed = match ReplayArgs::parse(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            return 3;
        }
    };

    let expected = match read_replay(parsed.expected) {
        Ok(record) => record,
        Err(message) => {
            let _ = writeln!(err, "error: expected artifact: {message}");
            return 2;
        }
    };
    let actual = match read_replay(parsed.actual) {
        Ok(record) => record,
        Err(message) => {
            let _ = writeln!(err, "error: actual artifact: {message}");
            return 2;
        }
    };

    match first_mismatch(&expected, &actual) {
        None => {
            let _ = writeln!(
                out,
                "ok: replay artifacts match ({} steps, {} checkpoints, final_hash={})",
                expected.steps.len(),
                expected.snapshots.len(),
                format_hash(expected.latest_hash())
            );
            0
        }
        Some(mismatch) => {
            if parsed.json {
                let _ = writeln!(out, "{}", mismatch.to_json());
            } else {
                let _ = writeln!(err, "replay mismatch: {}", mismatch.path);
                let _ = writeln!(err, "  expected: {}", mismatch.expected);
                let _ = writeln!(err, "  actual:   {}", mismatch.actual);
                let _ = writeln!(err, "  detail:   {}", mismatch.detail);
            }
            1
        }
    }
}

struct ReplayArgs<'a> {
    expected: &'a str,
    actual: &'a str,
    json: bool,
}

impl<'a> ReplayArgs<'a> {
    fn parse(args: &'a [String]) -> Result<Self, &'static str> {
        let mut paths = Vec::new();
        let mut json = false;
        for arg in args {
            if arg == "--json" {
                json = true;
            } else if arg.starts_with("--") {
                return Err("unknown option for `replay`");
            } else {
                paths.push(arg.as_str());
            }
        }

        match paths.as_slice() {
            [expected, actual] => Ok(Self {
                expected,
                actual,
                json,
            }),
            _ => Err("`replay` requires <expected.replay> <actual.replay>"),
        }
    }
}

fn read_replay(path: &str) -> Result<ReplayRecord, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    decode(&text).map_err(|e| format!("malformed {path}: {e}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mismatch {
    path: String,
    expected: String,
    actual: String,
    detail: String,
}

impl Mismatch {
    fn new(
        path: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            expected: expected.into(),
            actual: actual.into(),
            detail: detail.into(),
        }
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"status\":\"mismatch\",\"path\":\"{}\",\"expected\":\"{}\",\"actual\":\"{}\",\"detail\":\"{}\"}}",
            escape_json(&self.path),
            escape_json(&self.expected),
            escape_json(&self.actual),
            escape_json(&self.detail)
        )
    }
}

fn first_mismatch(expected: &ReplayRecord, actual: &ReplayRecord) -> Option<Mismatch> {
    if expected.format_version != actual.format_version {
        return Some(Mismatch::new(
            "format_version",
            expected.format_version.to_string(),
            actual.format_version.to_string(),
            "replay format versions differ",
        ));
    }

    if expected.initial_hash != actual.initial_hash {
        return Some(Mismatch::new(
            "initial_hash",
            format_hash(expected.initial_hash),
            format_hash(actual.initial_hash),
            "initial authority state hash differs",
        ));
    }

    for (position, (left, right)) in expected.steps.iter().zip(actual.steps.iter()).enumerate() {
        let step_path = format!("steps[{position}]");
        if left.index != right.index {
            return Some(Mismatch::new(
                format!("{step_path}.index"),
                left.index.raw().to_string(),
                right.index.raw().to_string(),
                "step index differs",
            ));
        }
        if left.command != right.command {
            return Some(Mismatch::new(
                format!("{step_path}.command"),
                format!("{:?}", left.command),
                format!("{:?}", right.command),
                "proposed command differs",
            ));
        }
        if left.outcome != right.outcome {
            return Some(Mismatch::new(
                format!("{step_path}.outcome"),
                outcome_summary(&left.outcome),
                outcome_summary(&right.outcome),
                "authority outcome differs",
            ));
        }
        if left.post_hash != right.post_hash {
            return Some(Mismatch::new(
                format!("{step_path}.post_hash"),
                format_hash(left.post_hash),
                format_hash(right.post_hash),
                "post-step authority hash differs",
            ));
        }
    }

    if expected.steps.len() != actual.steps.len() {
        return Some(Mismatch::new(
            "steps.len",
            expected.steps.len().to_string(),
            actual.steps.len().to_string(),
            "replay step count differs after all shared steps matched",
        ));
    }

    for (position, (left, right)) in expected
        .snapshots
        .iter()
        .zip(actual.snapshots.iter())
        .enumerate()
    {
        if let Some(mismatch) = compare_snapshot_meta(position, left, right) {
            return Some(mismatch);
        }
    }

    if expected.snapshots.len() != actual.snapshots.len() {
        return Some(Mismatch::new(
            "snapshots.len",
            expected.snapshots.len().to_string(),
            actual.snapshots.len().to_string(),
            "snapshot checkpoint count differs after all shared checkpoints matched",
        ));
    }

    None
}

fn compare_snapshot_meta(
    position: usize,
    expected: &SnapshotMeta,
    actual: &SnapshotMeta,
) -> Option<Mismatch> {
    let path = format!("snapshots[{position}]");
    if expected.step != actual.step {
        return Some(Mismatch::new(
            format!("{path}.step"),
            expected.step.raw().to_string(),
            actual.step.raw().to_string(),
            "snapshot checkpoint step differs",
        ));
    }
    if expected.hash != actual.hash {
        return Some(Mismatch::new(
            format!("{path}.hash"),
            format_hash(expected.hash),
            format_hash(actual.hash),
            "snapshot checkpoint state hash differs",
        ));
    }
    if expected.snapshot_version != actual.snapshot_version {
        return Some(Mismatch::new(
            format!("{path}.snapshot_version"),
            expected.snapshot_version.to_string(),
            actual.snapshot_version.to_string(),
            "snapshot payload version differs",
        ));
    }
    None
}

fn outcome_summary(outcome: &StepOutcome) -> String {
    match outcome {
        StepOutcome::Accepted { events } => format!("accepted({} events)", events.len()),
        StepOutcome::Rejected { summary } => format!("rejected({summary})"),
    }
}

fn format_hash(hash: ReplayHash) -> String {
    format!("{:016x}", hash.raw())
}

fn escape_json(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .expect("repo root")
            .to_path_buf()
    }

    fn golden() -> String {
        repo_root()
            .join("harness/goldens/replays/tagged-entity-run.replay")
            .to_string_lossy()
            .into_owned()
    }

    fn run_str(args: &[&str]) -> (u8, String, String) {
        let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run(&owned, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn temp_file(name: &str, contents: &str) -> String {
        let path = std::env::temp_dir().join(format!(
            "snapshot-diff-{}-{name}.replay",
            std::process::id()
        ));
        std::fs::write(&path, contents).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn identical_replay_artifacts_match() {
        let path = golden();
        let (code, out, err) = run_str(&["replay", &path, &path]);
        assert_eq!(code, 0);
        assert!(err.is_empty());
        assert!(out.contains("ok: replay artifacts match"));
        assert!(out.contains("3 checkpoints"));
    }

    #[test]
    fn malformed_input_exits_two() {
        let bad = temp_file("bad", "this is not a replay\n");
        let path = golden();
        let (code, out, err) = run_str(&["replay", &path, &bad]);
        std::fs::remove_file(&bad).ok();

        assert_eq!(code, 2);
        assert!(out.is_empty());
        assert!(err.contains("actual artifact: malformed"));
    }

    #[test]
    fn semantic_mismatch_reports_first_post_hash_drift() {
        let original = std::fs::read_to_string(golden()).unwrap();
        let tampered = original.replacen("post 9245ad62d9fc0fab", "post 0000000000000000", 1);
        let changed = temp_file("changed", &tampered);
        let path = golden();

        let (code, out, err) = run_str(&["replay", &path, &changed]);
        std::fs::remove_file(&changed).ok();

        assert_eq!(code, 1);
        assert!(out.is_empty());
        assert!(err.contains("replay mismatch: steps[0].post_hash"));
        assert!(err.contains("expected: 9245ad62d9fc0fab"));
        assert!(err.contains("actual:   0000000000000000"));
    }

    #[test]
    fn json_mismatch_is_machine_readable() {
        let original = std::fs::read_to_string(golden()).unwrap();
        let tampered = original.replacen(
            "snapshot 1 f2378409885f38f5 1",
            "snapshot 1 0000000000000000 1",
            1,
        );
        let changed = temp_file("json", &tampered);
        let path = golden();

        let (code, out, err) = run_str(&["replay", &path, &changed, "--json"]);
        std::fs::remove_file(&changed).ok();

        assert_eq!(code, 1);
        assert!(err.is_empty());
        assert!(out.starts_with("{\"status\":\"mismatch\""));
        assert!(out.contains("\"path\":\"snapshots[0].hash\""));
    }

    #[test]
    fn usage_errors_are_distinct() {
        let (code, _out, err) = run_str(&["replay", "only-one-path"]);
        assert_eq!(code, 3);
        assert!(err.contains("requires <expected.replay> <actual.replay>"));
    }
}
