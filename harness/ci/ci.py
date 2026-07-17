#!/usr/bin/env python3
"""Select and time ASHA CI gates without creating a second gate inventory."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import time
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[2]
REPORT_ROOT = ROOT / "harness/smoke-out/ci"

GUARDRAIL_POLICY_PATHS = {
    ".github/workflows/offline-ci.yml",
    "harness/ci/guardrail-policy.json",
    "harness/ci/guardrail_policy.py",
}

SELECTOR_CATEGORIES = (
    "unknown",
    "cross-cutting",
    "guardrail-policy",
    "docs",
    "harness-policy",
    "harness-contract",
    "rust",
    "typescript",
    "contract",
    "bridge",
    "native",
    "replay",
    "render",
)

GATES: dict[str, dict[str, Any]] = {
    "guardrail-policy": {
        "command": ["python3", "harness/ci/guardrail_policy.py", "--self-test"],
        "claims": ["guardrail registry and CI entrypoints agree"],
    },
    "rust": {
        "command": ["env", "ASHA_GAMEPLAY_RUNTIME_HOST_GATE_OWNS_TESTS=1", "harness/ci/check-rust.sh"],
        "claims": ["Rust formatting, compilation, clippy, and workspace tests"],
    },
    "typescript": {
        "command": ["harness/ci/check-ts.sh"],
        "claims": ["TypeScript build, typecheck, tests, lint, and package boundaries"],
    },
    "contracts": {
        "command": ["harness/ci/check-contracts.sh"],
        "claims": ["generated Rust-to-TypeScript border parity"],
    },
    "depgraph": {
        "command": ["harness/ci/check-depgraph.sh"],
        "claims": [
            "authority lanes, dependency edges, committed paths, and public package roots remain valid; source-shape and inventory pressure is advisory"
        ],
    },
    "no-den-coupling": {
        "command": ["harness/ci/check-no-den-coupling.sh"],
        "claims": ["engine remains independent of Den runtime code"],
    },
    "vocabulary": {
        "command": ["harness/ci/check-vocabulary.sh"],
        "claims": ["ECRP vocabulary and Rust authority naming remain legible"],
    },
    "identities": {
        "command": ["harness/ci/check-harness-identities.sh"],
        "claims": ["shared execution fingerprints, receipts, and artifact identity"],
    },
    "bridge": {
        "command": ["harness/ci/check-bridge.sh"],
        "claims": ["strict public wire and bridge boundary"],
    },
    "gameplay-conformance": {
        "command": ["harness/ci/check-gameplay-module-conformance.sh"],
        "claims": ["engine-owned gameplay provider behavior through public Rust seams"],
    },
    "gameplay-sdk": {
        "command": ["harness/ci/check-gameplay-module-sdk.sh"],
        "claims": ["public gameplay SDK fixture and scaffold"],
    },
    "replays": {
        "command": ["harness/ci/check-replays.sh"],
        "claims": ["replay, snapshot, and atomicity fixtures"],
    },
    "render-goldens": {
        "command": ["harness/ci/check-render-goldens.sh"],
        "claims": ["render projection goldens"],
    },
    "native": {
        "command": ["harness/ci/check-native.sh"],
        "claims": ["native addon, strict wire, runtime bridge, and browser-host integration"],
    },
}

ADVISORY_GATES: dict[str, dict[str, str]] = {
    "vocabulary": {
        "owner": "Architecture stewardship",
        "nextAction": "promote one precise rule only when a representative consequential API or authority ambiguity exists",
    },
}

FULL_ORDER = [
    "rust",
    "typescript",
    "contracts",
    "depgraph",
    "no-den-coupling",
    "vocabulary",
    "identities",
    "bridge",
    "gameplay-conformance",
    "gameplay-sdk",
    "replays",
    "render-goldens",
    "native",
]
FAST_ALWAYS = ["depgraph", "no-den-coupling", "vocabulary"]
FAST_ORDER = [
    "guardrail-policy",
    "depgraph",
    "no-den-coupling",
    "vocabulary",
    "contracts",
    "rust",
    "typescript",
    "identities",
    "bridge",
    "gameplay-conformance",
    "gameplay-sdk",
    "replays",
    "render-goldens",
    "native",
]


def stable_hash(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def gate_runtime_policy(gate: str) -> dict[str, str]:
    advisory = ADVISORY_GATES.get(gate)
    if advisory is not None:
        return {"failureMode": "warning", **advisory}
    return {
        "failureMode": "blocking",
        "owner": "Owning gate maintainer",
        "nextAction": "repair the named consequential failure before accepting the change",
    }


def classify_path(path: str) -> set[str]:
    normalized = pathlib.PurePosixPath(path).as_posix()
    categories: set[str] = set()
    if normalized == "<unknown>":
        return {"unknown"}
    if normalized.startswith(("docs/", "governance/lanes/")) or normalized in {
        "README.md",
        "AGENTS.md",
        "agents-project.md",
    }:
        categories.add("docs")
    if normalized.startswith((".github/", "harness/ci/")):
        categories.add("cross-cutting")
    if normalized in GUARDRAIL_POLICY_PATHS or normalized.startswith("harness/ci/"):
        categories.add("guardrail-policy")
    if normalized.startswith(("governance/", "harness/depgraph/", "harness/code-map/", "harness/lint/")):
        categories.add("harness-policy")
    if normalized.startswith((
        "harness/identity/",
        "harness/public-surface/",
    )):
        categories.add("harness-contract")
    if normalized == "harness/public-surface/check-public-rust-distribution.py":
        categories.add("bridge")
    if normalized.startswith("engine-rs/") or normalized in {"Cargo.toml", "engine-rs/Cargo.lock"}:
        categories.add("rust")
    if normalized.startswith("public-rust/"):
        categories.update(("rust", "harness-contract"))
    if normalized.startswith("ts/") or normalized in {"package.json", "pnpm-lock.yaml"}:
        categories.add("typescript")
    if normalized.startswith("engine-rs/crates/protocol/") or normalized.startswith(
        "ts/packages/contracts/"
    ):
        categories.add("contract")
    if normalized.startswith(("engine-rs/crates/bridge/", "harness/bridge/")):
        categories.add("bridge")
    if normalized.startswith("engine-rs/crates/bridge/native-bridge/") or normalized == "harness/ci/check-native.sh":
        categories.add("native")
    if "replay" in normalized or "snapshot" in normalized:
        categories.add("replay")
    if normalized.startswith(("engine-rs/crates/render/", "ts/packages/renderer-", "ts/packages/render-projection/")):
        categories.add("render")
    if normalized.startswith("harness/fixtures/"):
        categories.add("unknown")
    if not categories:
        categories.add("unknown")
    return categories


def select_fast(changed_files: list[str]) -> tuple[list[str], set[str], bool]:
    categories = set().union(*(classify_path(path) for path in changed_files)) if changed_files else set()
    if categories & {"unknown", "cross-cutting"}:
        selected = [*FULL_ORDER]
        if "guardrail-policy" in categories:
            selected.insert(0, "guardrail-policy")
        return selected, categories, True
    selected = set(FAST_ALWAYS)
    if "guardrail-policy" in categories:
        selected.add("guardrail-policy")
    if "rust" in categories:
        selected.add("rust")
    if "typescript" in categories:
        selected.add("typescript")
    if "contract" in categories:
        selected.update(("contracts", "rust", "typescript", "bridge"))
    if "harness-policy" in categories:
        selected.add("identities")
    if "harness-contract" in categories:
        selected.add("identities")
    if "bridge" in categories:
        selected.add("bridge")
    if "native" in categories:
        selected.add("native")
    if "replay" in categories:
        selected.add("replays")
    if "render" in categories:
        selected.add("render-goldens")
    ordered = [gate for gate in FAST_ORDER if gate in selected]
    return ordered, categories, False


def git_lines(*arguments: str) -> tuple[list[str], bool]:
    completed = subprocess.run(
        ["git", *arguments], cwd=ROOT, check=False, text=True, capture_output=True
    )
    return [line for line in completed.stdout.splitlines() if line], completed.returncode == 0


def resolve_commit(ref: str) -> str:
    if not ref or ref.startswith("-"):
        raise ValueError(f"invalid git commit ref: {ref!r}")
    completed = subprocess.run(
        ["git", "rev-parse", "--verify", "--quiet", f"{ref}^{{commit}}"],
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    resolved = completed.stdout.strip()
    if completed.returncode != 0 or len(resolved) != 40:
        raise ValueError(f"git commit ref does not resolve: {ref}")
    return resolved


def clean_commit_command(
    worktree: pathlib.Path,
    tier: str,
    base_commit: str | None,
    plan_only: bool,
    output: pathlib.Path | None,
    inject_failure: str | None,
) -> list[str]:
    script = worktree / "harness/ci" / ("check-fast.sh" if tier == "fast" else "check-all.sh")
    command = [str(script)]
    if tier == "fast" and base_commit:
        command.extend(("--base-ref", base_commit))
    if plan_only:
        command.append("--plan")
    if output:
        command.extend(("--output", str(output)))
    if inject_failure:
        command.extend(("--inject-failure", inject_failure))
    return command


def run_clean_commit(
    parser: argparse.ArgumentParser,
    tier: str,
    commit_ref: str,
    base_ref: str | None,
    plan_only: bool,
    output: pathlib.Path | None,
    inject_failure: str | None,
) -> int:
    try:
        commit = resolve_commit(commit_ref)
        base_commit = resolve_commit(base_ref or f"{commit}^")
    except ValueError as error:
        parser.error(str(error))

    selected_output = output
    if selected_output and not selected_output.is_absolute():
        selected_output = (pathlib.Path.cwd() / selected_output).resolve()
    if not plan_only and selected_output is None:
        selected_output = REPORT_ROOT / f"{tier}-commit-{commit[:12]}.json"

    print(
        f"==> Validating exact commit {commit} from clean detached worktree "
        f"(base {base_commit}, tier {tier})",
        file=sys.stderr,
        flush=True,
    )
    with tempfile.TemporaryDirectory(prefix="asha-ci-clean-") as temporary:
        worktree = pathlib.Path(temporary) / "worktree"
        added = False
        try:
            subprocess.run(
                ["git", "worktree", "add", "--detach", str(worktree), commit],
                cwd=ROOT,
                check=True,
                stdout=sys.stderr,
            )
            added = True
            command = clean_commit_command(
                worktree,
                tier,
                base_commit,
                plan_only,
                selected_output,
                inject_failure,
            )
            environment = os.environ.copy()
            environment.update({
                "ASHA_CI_BASE_REF": base_commit,
                "ASHA_SOURCE_SHAPE_BASE_REF": base_commit,
                "ASHA_CI_VALIDATION_MODE": "clean-commit",
                "ASHA_CI_VALIDATION_COMMIT": commit,
                "ASHA_CI_VALIDATION_BASE_COMMIT": base_commit,
            })
            validation_target = {
                "mode": "clean-commit",
                "commit": commit,
                "baseCommit": base_commit,
            }
            completed = subprocess.run(
                command,
                cwd=worktree,
                env=environment,
                check=False,
                text=plan_only,
                capture_output=plan_only,
            )
            if plan_only:
                if completed.stderr:
                    print(completed.stderr, file=sys.stderr, end="")
                if completed.stdout:
                    plan = json.loads(completed.stdout)
                    plan["validationTarget"] = validation_target
                    print(json.dumps(plan, indent=2))
            elif selected_output and selected_output.is_file():
                report = json.loads(selected_output.read_text(encoding="utf-8"))
                report["validationTarget"] = validation_target
                selected_output.write_text(
                    json.dumps(report, indent=2) + "\n",
                    encoding="utf-8",
                )
            return completed.returncode
        finally:
            if added:
                removed = subprocess.run(
                    ["git", "worktree", "remove", "--force", str(worktree)],
                    cwd=ROOT,
                    check=False,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.PIPE,
                    text=True,
                )
                if removed.returncode != 0:
                    print(
                        f"WARN: could not unregister temporary CI worktree: {removed.stderr.strip()}",
                        file=sys.stderr,
                    )
                    subprocess.run(
                        ["git", "worktree", "prune"],
                        cwd=ROOT,
                        check=False,
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL,
                    )


def detect_changed_files(base_ref: str | None) -> list[str]:
    changed: set[str] = set()
    reliable = True
    if base_ref and set(base_ref) != {"0"}:
        lines, ok = git_lines("diff", "--name-only", f"{base_ref}...HEAD")
        changed.update(lines)
        reliable &= ok
    for arguments in (
        ("diff", "--name-only"),
        ("diff", "--cached", "--name-only"),
        ("ls-files", "--others", "--exclude-standard"),
    ):
        lines, ok = git_lines(*arguments)
        changed.update(lines)
        reliable &= ok
    if not reliable:
        changed.add("<unknown>")
    return sorted(changed)


def gate_command(
    gate: str,
    tier: str,
    categories: set[str],
    expanded_to_full: bool,
) -> list[str]:
    command = list(GATES[gate]["command"])
    if (
        tier == "fast"
        and not expanded_to_full
        and gate in {"depgraph", "typescript"}
        and "harness-policy" not in categories
    ):
        command = ["env", "ASHA_HARNESS_SELF_TESTS=0", *command]
    return command


def plan_document(tier: str, changed_files: list[str]) -> dict[str, Any]:
    if tier == "full":
        selected, categories, expanded = list(FULL_ORDER), set(), False
    else:
        selected, categories, expanded = select_fast(changed_files)
    gates = []
    for gate in selected:
        command = gate_command(gate, tier, categories, expanded)
        policy = gate_runtime_policy(gate)
        gates.append({
            "id": gate,
            "normalizedCommand": command,
            "commandFingerprint": stable_hash(command),
            "semanticClaimConsumers": GATES[gate]["claims"],
            "failureMode": policy["failureMode"],
            "owner": policy["owner"],
            "nextAction": policy["nextAction"],
        })
    plan = {
        "schemaVersion": 1,
        "tier": tier,
        "changedFiles": changed_files,
        "changeClasses": sorted(categories),
        "expandedToFull": expanded,
        "gates": gates,
    }
    validation_commit = os.environ.get("ASHA_CI_VALIDATION_COMMIT")
    if validation_commit:
        plan["validationTarget"] = {
            "mode": os.environ.get("ASHA_CI_VALIDATION_MODE", "clean-commit"),
            "commit": validation_commit,
            "baseCommit": os.environ.get("ASHA_CI_VALIDATION_BASE_COMMIT"),
        }
    return plan


def run_plan(plan: dict[str, Any], output: pathlib.Path, inject_failure: str | None) -> int:
    run_started = time.monotonic()
    execution_event_log = REPORT_ROOT / "execution-events.jsonl"
    execution_event_log.parent.mkdir(parents=True, exist_ok=True)
    execution_event_log.unlink(missing_ok=True)
    os.environ["ASHA_EXECUTION_EVENT_LOG"] = str(execution_event_log)
    results = []
    for gate in plan["gates"]:
        command = gate["normalizedCommand"]
        if inject_failure == gate["id"]:
            command = ["bash", "-c", "exit 86"]
        print(f"==> CI gate {gate['id']}: {' '.join(command)}", flush=True)
        gate_started = time.monotonic()
        completed = subprocess.run(command, cwd=ROOT, check=False)
        outcome = "passed"
        if completed.returncode != 0:
            outcome = "warning" if gate["failureMode"] == "warning" else "failed"
        results.append({
            **gate,
            "exitCode": completed.returncode,
            "outcome": outcome,
            "wallTimeMs": round((time.monotonic() - gate_started) * 1000),
        })
        if completed.returncode != 0:
            if gate["failureMode"] == "warning":
                print(
                    f"WARN: advisory gate {gate['id']} failed; owner={gate['owner']}; "
                    f"next={gate['nextAction']}",
                    file=sys.stderr,
                    flush=True,
                )
                continue
            break
    fingerprints = [item["commandFingerprint"] for item in results]
    execution_events = []
    if execution_event_log.is_file():
        execution_events = [
            json.loads(line)
            for line in execution_event_log.read_text(encoding="utf-8").splitlines()
            if line
        ]
    observed_shared_executions = [
        execution
        for event in execution_events
        for execution in event.get("executions", [])
    ]
    actual_by_fingerprint: dict[str, int] = {}
    for execution in observed_shared_executions:
        if not execution.get("cacheHit"):
            fingerprint = execution["fingerprint"]
            actual_by_fingerprint[fingerprint] = actual_by_fingerprint.get(fingerprint, 0) + 1
    duplicate_actual_fingerprints = sorted(
        fingerprint for fingerprint, count in actual_by_fingerprint.items() if count > 1
    )
    execution_fingerprints = [item["fingerprint"] for item in observed_shared_executions]
    execution_summary = {
        "schedulerCallCount": len(execution_events),
        "observedFingerprintCount": len(execution_fingerprints),
        "uniqueFingerprintCount": len(set(execution_fingerprints)),
        "repeatedFingerprintRequestCount": len(execution_fingerprints) - len(set(execution_fingerprints)),
        "actualExecutionCount": sum(not item.get("cacheHit") for item in observed_shared_executions),
        "receiptReuseCount": sum(bool(item.get("cacheHit")) for item in observed_shared_executions),
        "duplicateActualFingerprints": duplicate_actual_fingerprints,
    }
    wall_time_ms = round((time.monotonic() - run_started) * 1000)
    report = {
        **plan,
        "valid": (
            len(results) == len(plan["gates"])
            and all(
                item["exitCode"] == 0 or item["failureMode"] == "warning"
                for item in results
            )
            and not duplicate_actual_fingerprints
        ),
        "summary": {
            "selectedGateCount": len(plan["gates"]),
            "completedGateCount": len(results),
            "uniqueCommandCount": len(set(fingerprints)),
            "repeatedCommandCount": len(fingerprints) - len(set(fingerprints)),
            "advisoryWarningCount": sum(
                item["exitCode"] != 0 and item["failureMode"] == "warning"
                for item in results
            ),
            "blockingFailureCount": sum(
                item["exitCode"] != 0 and item["failureMode"] == "blocking"
                for item in results
            ),
            "runnerWallTimeMs": wall_time_ms,
            "runnerMinutes": round(wall_time_ms / 60_000, 3),
            "sharedExecutions": execution_summary,
        },
        "results": results,
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    try:
        output_label = output.relative_to(ROOT)
    except ValueError:
        output_label = output
    print(f"CI timing report: {output_label}")
    if report["valid"]:
        return 0
    return next(
        (item["exitCode"] for item in results if item["exitCode"] != 0),
        1,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tier", choices=("fast", "full"), required=True)
    parser.add_argument("--base-ref", default=os.environ.get("ASHA_CI_BASE_REF"))
    parser.add_argument("--changed-file", action="append", default=[])
    parser.add_argument(
        "--clean-commit",
        nargs="?",
        const="HEAD",
        help="run this tier at an exact commit in a temporary clean detached worktree",
    )
    parser.add_argument("--plan", action="store_true")
    parser.add_argument("--output", type=pathlib.Path)
    parser.add_argument("--inject-failure", choices=tuple(GATES))
    args = parser.parse_args()
    if args.inject_failure and os.environ.get("ASHA_CI_ALLOW_FAILURE_INJECTION") != "1":
        parser.error("failure injection requires ASHA_CI_ALLOW_FAILURE_INJECTION=1")
    if args.clean_commit:
        if args.changed_file:
            parser.error("--clean-commit cannot be combined with --changed-file")
        return run_clean_commit(
            parser,
            args.tier,
            args.clean_commit,
            args.base_ref,
            args.plan,
            args.output,
            args.inject_failure,
        )
    changed_files = args.changed_file or (
        detect_changed_files(args.base_ref) if args.tier == "fast" else []
    )
    plan = plan_document(args.tier, changed_files)
    if args.plan:
        print(json.dumps(plan, indent=2))
        return 0
    output = args.output or REPORT_ROOT / f"{args.tier}-latest.json"
    return run_plan(plan, output, args.inject_failure)


if __name__ == "__main__":
    raise SystemExit(main())
