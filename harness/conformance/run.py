#!/usr/bin/env python3
"""Execute the declared conformance suites through the shared proof scheduler."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "harness/identity"))

from execution import ExecutionError, make_plan, run_plan  # noqa: E402
from claim_results import build_claim_report, unavailable_executions  # noqa: E402
from validate import MANIFEST, validate  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--workspace-parent",
        type=pathlib.Path,
        default=ROOT.parent,
        help="parent directory used to resolve optional sibling consumer checkouts",
    )
    args = parser.parse_args()
    definitions_path = ROOT / "harness/identity/executions.json"
    definitions = json.loads(definitions_path.read_text(encoding="utf-8"))["executions"]
    workspace_parent = args.workspace_parent.resolve()
    unavailable = unavailable_executions(definitions, workspace_parent)
    available_ids = [
        definition["id"] for definition in definitions
        if definition["id"] not in unavailable
    ]
    try:
        plan = make_plan(available_ids)
        exit_code, execution_report = run_plan(plan)
    except ExecutionError as error:
        print(f"conformance execution: {error}", file=sys.stderr)
        return 1
    report = build_claim_report(
        json.loads(MANIFEST.read_text(encoding="utf-8")),
        validate(MANIFEST),
        definitions,
        plan,
        execution_report,
        unavailable,
    )
    report["workspaceParent"] = workspace_parent.as_posix()
    report_path = ROOT / "harness/smoke-out/proof-execution/conformance-report.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    states = report["summary"]["executionStates"]
    consumer = next(
        item["state"] for item in report["claimVerdicts"]
        if item["claimClass"] == "consumerProductAcceptance"
    )
    print(
        f"Conformance execution claims: {states['passed']} passed, "
        f"{states['failed']} failed, {states['stale']} stale, "
        f"{states['not_run']} not run, {states['unavailable']} unavailable; "
        f"consumer acceptance {consumer}; "
        f"report {report_path.relative_to(ROOT)})."
    )
    if exit_code != 0:
        return exit_code
    return 0 if report["valid"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
