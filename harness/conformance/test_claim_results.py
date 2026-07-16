#!/usr/bin/env python3
"""Behavioral tests for honest conformance claim and execution states."""

from __future__ import annotations

import pathlib
import tempfile
import unittest

import claim_results


def plan(repository_revision: str = "a" * 40) -> dict:
    inputs = {
        "normalizedCommand": ["test"],
        "inputDigest": "sha256:inputs",
        "providerDigest": "sha256:provider",
        "toolchain": {"test": "one"},
        "environment": {},
        "repositoryRevisions": {".": repository_revision},
    }
    return {
        "executionIds": ["execution.one"],
        "fingerprint": "sha256:current",
        "fingerprintInputs": inputs,
    }


def receipt(**updates: object) -> dict:
    expected = plan()
    result = {
        **expected,
        "exitCode": 0,
        "attributions": [{"suiteId": "suite.one"}],
    }
    result.update(updates)
    return result


class ClaimResultTests(unittest.TestCase):
    def test_structural_evidence_without_execution_is_not_run(self) -> None:
        outcome = claim_results.suite_execution_outcome(
            "suite.one", "execution.one", None, None, None
        )
        self.assertEqual(outcome["state"], "not_run")

    def test_missing_consumer_checkout_is_unavailable(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            availability = claim_results.unavailable_executions(
                [{"id": "execution.one", "inputs": ["../asha-demo/test.mjs"]}],
                pathlib.Path(temporary),
            )
        self.assertEqual(availability["execution.one"]["state"], "unavailable")

    def test_fresh_success_and_failure_are_distinct(self) -> None:
        success = claim_results.suite_execution_outcome(
            "suite.one", "execution.one", plan(), receipt(), None
        )
        failure = claim_results.suite_execution_outcome(
            "suite.one", "execution.one", plan(), receipt(exitCode=7), None
        )
        self.assertEqual(success["state"], "passed")
        self.assertEqual(failure["state"], "failed")

    def test_wrong_fingerprint_input_or_repository_revision_is_stale(self) -> None:
        wrong_fingerprint = receipt(fingerprint="sha256:old")
        wrong_input = receipt(fingerprintInputs={
            **plan()["fingerprintInputs"],
            "inputDigest": "sha256:old-inputs",
        })
        wrong_revision = receipt(fingerprintInputs={
            **plan()["fingerprintInputs"],
            "repositoryRevisions": {".": "b" * 40},
        })
        for candidate in (wrong_fingerprint, wrong_input, wrong_revision):
            with self.subTest(candidate=candidate):
                outcome = claim_results.suite_execution_outcome(
                    "suite.one", "execution.one", plan(), candidate, None
                )
                self.assertEqual(outcome["state"], "stale")

    def test_receipt_without_suite_attribution_cannot_pass(self) -> None:
        outcome = claim_results.suite_execution_outcome(
            "suite.one", "execution.one", plan(), receipt(attributions=[]), None
        )
        self.assertEqual(outcome["state"], "failed")


if __name__ == "__main__":
    unittest.main()
