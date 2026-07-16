#!/usr/bin/env python3
"""Reconcile the compact guardrail registry with ASHA CI entrypoints."""

from __future__ import annotations

import argparse
import copy
import json
import pathlib
import sys
from typing import Any

import ci

ROOT = pathlib.Path(__file__).resolve().parents[2]
POLICY_PATH = ROOT / "harness/ci/guardrail-policy.json"
WORKFLOW_PATH = ROOT / ".github/workflows/offline-ci.yml"

GATE_DISPOSITIONS = {"blocking", "advisory", "change-triggered", "scheduled/campaign-close"}
DECISION_DISPOSITIONS = GATE_DISPOSITIONS | {"consumer-owned", "merged", "retired"}
FAILURE_MODES = {"blocking", "warning", "external"}
REQUIRED_FIELDS = {
    "owner",
    "claim",
    "disposition",
    "failureMode",
    "trigger",
    "failSafe",
    "cost",
    "escalation",
}
BLOCKING_FIELDS = {"failureClass", "representativeRegression"}
EXPECTED_DECISIONS = {
    "source-shape-pressure",
    "generated-inventory-freshness",
    "validator-negative-fixtures",
    "computed-validation-reports",
    "consumer-live-acceptance",
    "duplicate-execution-wrappers",
}
SELECTOR_EXAMPLES = {
    "unknown": "unclassified/new-root.file",
    "cross-cutting": "harness/ci/check-all.sh",
    "guardrail-policy": "harness/ci/guardrail-policy.json",
    "docs": "docs/runtime-session-facade.md",
    "harness-policy": "governance/ownership.toml",
    "harness-contract": "harness/identity/execution.py",
    "rust": "engine-rs/crates/state/core-scene/src/lib.rs",
    "typescript": "ts/packages/ui-dom/src/index.ts",
    "contract": "engine-rs/crates/protocol/protocol-scene/src/lib.rs",
    "bridge": "engine-rs/crates/bridge/runtime-bridge-api/src/lib.rs",
    "native": "engine-rs/crates/bridge/native-bridge/src/lib.rs",
    "replay": "engine-rs/crates/sim/sim-replay/src/lib.rs",
    "render": "engine-rs/crates/render/render-bridge/src/lib.rs",
}


def load_policy() -> dict[str, Any]:
    return json.loads(POLICY_PATH.read_text(encoding="utf-8"))


def non_empty_text(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def validate_entry(
    entry_id: str,
    entry: Any,
    allowed_dispositions: set[str],
) -> list[str]:
    errors: list[str] = []
    if not isinstance(entry, dict):
        return [f"{entry_id}: entry must be an object"]
    missing = sorted(REQUIRED_FIELDS - set(entry))
    if missing:
        errors.append(f"{entry_id}: missing fields {missing}")
    for field in sorted(REQUIRED_FIELDS & set(entry)):
        if not non_empty_text(entry[field]):
            errors.append(f"{entry_id}: {field} must be non-empty text")
    if entry.get("disposition") not in allowed_dispositions:
        errors.append(f"{entry_id}: invalid disposition {entry.get('disposition')!r}")
    if entry.get("failureMode") not in FAILURE_MODES:
        errors.append(f"{entry_id}: invalid failureMode {entry.get('failureMode')!r}")
    if entry.get("failureMode") == "blocking":
        for field in sorted(BLOCKING_FIELDS):
            if not non_empty_text(entry.get(field)):
                errors.append(f"{entry_id}: blocking entry requires {field}")
    return errors


def validate_document(document: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(document, dict):
        return ["policy root must be an object"]
    if document.get("schemaVersion") != 1:
        errors.append("schemaVersion must be 1")
    if not non_empty_text(document.get("computedResults")):
        errors.append("computedResults must state the artifact posture")
    requirements = document.get("newBlockingGateRequirements")
    if not isinstance(requirements, list) or not all(non_empty_text(item) for item in requirements):
        errors.append("newBlockingGateRequirements must be a non-empty text list")

    gates = document.get("gates")
    if not isinstance(gates, dict):
        return [*errors, "gates must be an object"]
    expected_gates = set(ci.GATES)
    if set(gates) != expected_gates:
        errors.append(
            f"gate registry mismatch: missing={sorted(expected_gates - set(gates))} "
            f"extra={sorted(set(gates) - expected_gates)}"
        )
    for gate_id, entry in gates.items():
        errors.extend(validate_entry(f"gate {gate_id}", entry, GATE_DISPOSITIONS))
        configured = ci.GATES.get(gate_id)
        if configured is not None and entry.get("claim") not in configured["claims"]:
            errors.append(f"gate {gate_id}: claim does not match ci.py")
        if configured is not None:
            runtime_failure_mode = ci.gate_runtime_policy(gate_id)["failureMode"]
            if entry.get("failureMode") != runtime_failure_mode:
                errors.append(
                    f"gate {gate_id}: failureMode does not match ci.py runtime posture"
                )

    if set(ci.FULL_ORDER) != (expected_gates - {"guardrail-policy"}):
        errors.append("FULL_ORDER must contain every runtime gate except change-triggered policy validation")
    if not set(ci.FAST_ALWAYS).issubset(expected_gates):
        errors.append("FAST_ALWAYS references an unknown gate")
    if "guardrail-policy" not in ci.FAST_ORDER:
        errors.append("guardrail-policy must be orderable on policy changes")

    selectors = document.get("selectorCategories")
    if not isinstance(selectors, dict):
        errors.append("selectorCategories must be an object")
    else:
        expected_selectors = set(ci.SELECTOR_CATEGORIES)
        if set(selectors) != expected_selectors:
            errors.append(
                f"selector category mismatch: missing={sorted(expected_selectors - set(selectors))} "
                f"extra={sorted(set(selectors) - expected_selectors)}"
            )
        for selector_id, entry in selectors.items():
            if not isinstance(entry, dict):
                errors.append(f"selector {selector_id}: entry must be an object")
                continue
            for field in ("disposition", "gates", "fallback"):
                if field not in entry:
                    errors.append(f"selector {selector_id}: missing {field}")
            if entry.get("disposition") not in GATE_DISPOSITIONS:
                errors.append(f"selector {selector_id}: invalid disposition")
            if not non_empty_text(entry.get("fallback")):
                errors.append(f"selector {selector_id}: fallback must be non-empty text")
            gate_refs = entry.get("gates")
            if gate_refs != "full":
                if not isinstance(gate_refs, list) or not gate_refs:
                    errors.append(f"selector {selector_id}: gates must be 'full' or a non-empty list")
                elif not set(gate_refs).issubset(expected_gates):
                    errors.append(f"selector {selector_id}: references unknown gates")
            example = SELECTOR_EXAMPLES.get(selector_id)
            if example is not None:
                selected, categories, _expanded = ci.select_fast([example])
                if selector_id not in categories:
                    errors.append(f"selector {selector_id}: example is no longer classified")
                expected_refs = set(ci.FULL_ORDER) if gate_refs == "full" else set(gate_refs or [])
                if not expected_refs.issubset(selected):
                    errors.append(f"selector {selector_id}: registry gates do not match selection")

    decisions = document.get("explicitDecisions")
    if not isinstance(decisions, dict):
        errors.append("explicitDecisions must be an object")
    else:
        if set(decisions) != EXPECTED_DECISIONS:
            errors.append(
                f"explicit decision mismatch: missing={sorted(EXPECTED_DECISIONS - set(decisions))} "
                f"extra={sorted(set(decisions) - EXPECTED_DECISIONS)}"
            )
        for decision_id, entry in decisions.items():
            errors.extend(validate_entry(f"decision {decision_id}", entry, DECISION_DISPOSITIONS))

    workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
    if "./harness/ci/check-fast.sh" not in workflow:
        errors.append("GitHub workflow does not invoke check-fast.sh")
    if "./harness/ci/check-all.sh" not in workflow:
        errors.append("GitHub workflow does not invoke check-all.sh")
    if ci.GATES.get("native", {}).get("command") != ["harness/ci/check-native.sh"]:
        errors.append("native gate is not reconciled to check-native.sh")
    return errors


def run_self_tests(document: dict[str, Any]) -> list[str]:
    failures: list[str] = []

    missing_gate = copy.deepcopy(document)
    del missing_gate["gates"]["native"]
    if not any("gate registry mismatch" in error for error in validate_document(missing_gate)):
        failures.append("self-test: missing gate was accepted")

    incomplete_blocker = copy.deepcopy(document)
    incomplete_blocker["gates"]["bridge"].pop("failureClass")
    if not any("blocking entry requires failureClass" in error for error in validate_document(incomplete_blocker)):
        failures.append("self-test: incomplete blocking claim was accepted")

    unknown_selector = copy.deepcopy(document)
    unknown_selector["selectorCategories"]["ambient-magic"] = {
        "disposition": "change-triggered",
        "gates": ["rust"],
        "fallback": "none",
    }
    if not any("selector category mismatch" in error for error in validate_document(unknown_selector)):
        failures.append("self-test: unknown selector category was accepted")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    document = load_policy()
    errors = validate_document(document)
    if args.self_test:
        errors.extend(run_self_tests(document))
    if errors:
        for error in errors:
            print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print(
        "Guardrail policy: OK "
        f"({len(document['gates'])} gates, "
        f"{len(document['selectorCategories'])} selector categories, "
        f"{len(document['explicitDecisions'])} explicit dispositions)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
