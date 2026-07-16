#!/usr/bin/env python3
"""Join structural conformance with fresh execution receipts without merging authorities."""

from __future__ import annotations

import pathlib
from typing import Any

EXECUTION_STATES = ("passed", "failed", "not_run", "unavailable", "stale")
CLAIM_CLASSES = (
    "structuralDeclaration",
    "providerIntegrationExecution",
    "consumerProductAcceptance",
)
EXTERNAL_CONSUMER_ROOTS = {"asha-demo", "asha-studio", "asha-testing"}


def external_execution_roots(definition: dict[str, Any]) -> list[str]:
    roots = {
        parts[1]
        for source in definition.get("inputs", [])
        if len(parts := pathlib.PurePosixPath(source).parts) >= 2
        and parts[0] == ".."
        and parts[1] in EXTERNAL_CONSUMER_ROOTS
    }
    return sorted(roots)


def unavailable_executions(
    definitions: list[dict[str, Any]], workspace_parent: pathlib.Path
) -> dict[str, dict[str, str]]:
    unavailable: dict[str, dict[str, str]] = {}
    for definition in definitions:
        missing = [
            root for root in external_execution_roots(definition)
            if not (workspace_parent / root).is_dir()
        ]
        if missing:
            unavailable[definition["id"]] = {
                "state": "unavailable",
                "reasonCode": "consumer_checkout_unavailable",
                "message": f"missing consumer checkout(s): {', '.join(missing)}",
            }
    return unavailable


def execution_claim_class(definition: dict[str, Any]) -> str:
    if external_execution_roots(definition):
        return "consumerProductAcceptance"
    return "providerIntegrationExecution"


def _index_plans(plan: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    return {
        identity: item
        for item in plan
        for identity in item.get("executionIds", [])
    }


def _index_receipts(report: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {
        identity: item
        for item in report.get("executions", [])
        for identity in item.get("executionIds", [])
    }


def suite_execution_outcome(
    suite_id: str,
    execution_id: str,
    expected_plan: dict[str, Any] | None,
    receipt: dict[str, Any] | None,
    unavailable: dict[str, str] | None,
) -> dict[str, Any]:
    if unavailable is not None:
        return dict(unavailable)
    if expected_plan is None or receipt is None:
        return {
            "state": "not_run",
            "reasonCode": "execution_receipt_absent",
            "message": "no fresh execution receipt was supplied",
        }
    if (
        receipt.get("fingerprint") != expected_plan.get("fingerprint")
        or receipt.get("fingerprintInputs") != expected_plan.get("fingerprintInputs")
    ):
        return {
            "state": "stale",
            "reasonCode": "execution_fingerprint_mismatch",
            "message": "receipt command, inputs, toolchain, provider, or repository revision is stale",
        }
    if receipt.get("exitCode") != 0:
        return {
            "state": "failed",
            "reasonCode": "execution_failed",
            "message": f"execution {execution_id} returned {receipt.get('exitCode')!r}",
        }
    attribution = next(
        (
            item for item in receipt.get("attributions", [])
            if item.get("suiteId") == suite_id
        ),
        None,
    )
    if attribution is None:
        return {
            "state": "failed",
            "reasonCode": "suite_attribution_absent",
            "message": "fresh receipt did not attribute the executed suite",
        }
    return {
        "state": "passed",
        "reasonCode": "fresh_execution_passed",
        "message": "fresh fingerprint- and repository-bound execution passed",
        "fingerprint": receipt["fingerprint"],
        "repositoryRevisions": receipt["fingerprintInputs"].get(
            "repositoryRevisions", {}
        ),
    }


def aggregate_state(states: list[str]) -> str:
    if not states:
        return "not_run"
    for state in ("failed", "stale", "unavailable", "not_run"):
        if state in states:
            return state
    return "passed"


def build_claim_report(
    manifest: dict[str, Any],
    structural_report: dict[str, Any],
    definitions: list[dict[str, Any]],
    plan: list[dict[str, Any]],
    execution_report: dict[str, Any],
    unavailable: dict[str, dict[str, str]],
) -> dict[str, Any]:
    definition_map = {item["id"]: item for item in definitions}
    planned = _index_plans(plan)
    receipts = _index_receipts(execution_report)
    structural_probe_map = {
        item["id"]: item for item in structural_report.get("semanticProbes", [])
    }

    suite_outcomes: list[dict[str, Any]] = []
    for suite in manifest.get("suites", []):
        execution_id = suite["executionId"]
        definition = definition_map[execution_id]
        outcome = suite_execution_outcome(
            suite["id"],
            execution_id,
            planned.get(execution_id),
            receipts.get(execution_id),
            unavailable.get(execution_id),
        )
        suite_outcomes.append({
            "suite": suite["id"],
            "executionId": execution_id,
            "claimClass": execution_claim_class(definition),
            **outcome,
        })

    suite_map = {item["suite"]: item for item in suite_outcomes}
    probe_outcomes: list[dict[str, Any]] = []
    for probe in manifest.get("semanticProbes", []):
        structural = structural_probe_map.get(probe["id"], {})
        execution = suite_map[probe["suite"]]
        structural_state = structural.get("structuralState", "failed")
        state = "failed" if structural_state != "passed" else execution["state"]
        probe_outcomes.append({
            "id": probe["id"],
            "suite": probe["suite"],
            "executionId": execution["executionId"],
            "claimClass": execution["claimClass"],
            "structuralState": structural_state,
            "executionState": execution["state"],
            "state": state,
        })

    provider_states = [
        item["state"] for item in suite_outcomes
        if item["claimClass"] == "providerIntegrationExecution"
    ]
    consumer_states = [
        item["state"] for item in suite_outcomes
        if item["claimClass"] == "consumerProductAcceptance"
    ]
    verdicts = [
        {
            "claimClass": "structuralDeclaration",
            "state": "passed" if structural_report.get("valid") else "failed",
        },
        {
            "claimClass": "providerIntegrationExecution",
            "state": aggregate_state(provider_states),
        },
        {
            "claimClass": "consumerProductAcceptance",
            "state": aggregate_state(consumer_states),
        },
    ]
    state_counts = {
        state: sum(item["state"] == state for item in suite_outcomes)
        for state in EXECUTION_STATES
    }
    blocking_states = {
        item["state"] for item in suite_outcomes if item["state"] in {"failed", "stale"}
    }
    return {
        "schemaVersion": 1,
        "valid": structural_report.get("valid") is True and not blocking_states,
        "claimVerdicts": verdicts,
        "summary": {
            "suiteCount": len(suite_outcomes),
            "executionStates": state_counts,
        },
        "suites": suite_outcomes,
        "semanticProbes": probe_outcomes,
        "executionReceipts": execution_report.get("executions", []),
        "structuralReport": {
            "inventoryHash": structural_report.get("inventoryHash"),
            "valid": structural_report.get("valid"),
        },
    }
