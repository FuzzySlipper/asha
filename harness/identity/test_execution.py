#!/usr/bin/env python3
"""Negative and attribution tests for proof identity scheduling."""

from __future__ import annotations

import json
import pathlib
import tempfile
import unittest

import execution


class ProofExecutionTests(unittest.TestCase):
    def test_execution_identity_collision_is_rejected(self) -> None:
        with self.assertRaisesRegex(execution.ExecutionError, "identity collision"):
            execution.definition_index([{"id": "same"}, {"id": "same"}])

    def test_stale_cache_receipt_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            paths = execution.cache_paths(root, "sha256:current")
            paths["directory"].mkdir(parents=True)
            paths["stdout"].write_text("out", encoding="utf-8")
            paths["stderr"].write_text("err", encoding="utf-8")
            paths["receipt"].write_text(
                json.dumps({"fingerprint": "sha256:stale", "exitCode": 0}),
                encoding="utf-8",
            )
            self.assertFalse(execution.cache_valid(paths, "sha256:current"))

    def test_missing_provider_is_rejected(self) -> None:
        catalog = {"families": {"providers": [{"id": "known"}]}}
        with self.assertRaisesRegex(execution.ExecutionError, "missing provider"):
            execution.provider_digest(catalog, ["unknown"])

    def test_divergent_environment_changes_fingerprint(self) -> None:
        definition = {"id": "proof", "command": ["cargo", "test"], "providerIds": []}
        settings = {"environmentKeys": ["CI"], "environmentPrefixes": []}
        catalog = {"families": {"providers": []}}
        first, _ = execution.execution_fingerprint(
            definition, settings, catalog, {"CI": "first"}, {"cargo": "same"}, "sha256:inputs"
        )
        second, _ = execution.execution_fingerprint(
            definition, settings, catalog, {"CI": "second"}, {"cargo": "same"}, "sha256:inputs"
        )
        self.assertNotEqual(first, second)

    def test_reviewed_build_environment_changes_fingerprint(self) -> None:
        definition = {"id": "proof", "command": ["cargo", "test"], "providerIds": []}
        settings = execution.load_json(execution.DEFINITIONS)
        catalog = {"families": {"providers": []}}
        baseline_environment = {
            "CC": "gcc",
            "CFLAGS": "-O1",
            "NODE_ENV": "development",
            "TMPDIR": "/tmp/proof-one",
        }
        baseline, inputs = execution.execution_fingerprint(
            definition,
            settings,
            catalog,
            baseline_environment,
            {"cargo": "same"},
            "sha256:inputs",
        )
        for key, value in baseline_environment.items():
            self.assertEqual(inputs["environment"][key], value)

        for key, changed_value in (
            ("CC", "clang"),
            ("CFLAGS", "-O2"),
            ("NODE_ENV", "production"),
            ("TMPDIR", "/tmp/proof-two"),
        ):
            changed_environment = {**baseline_environment, key: changed_value}
            changed, _ = execution.execution_fingerprint(
                definition,
                settings,
                catalog,
                changed_environment,
                {"cargo": "same"},
                "sha256:inputs",
            )
            with self.subTest(key=key):
                self.assertNotEqual(baseline, changed)

    def test_command_input_toolchain_and_provider_changes_invalidate_fingerprint(self) -> None:
        definition = {"id": "proof", "command": ["cargo", "test"], "providerIds": ["provider"]}
        settings = {"environmentKeys": [], "environmentPrefixes": []}
        catalog = {"families": {"providers": [{"id": "provider", "sourceHash": "sha256:first"}]}}
        baseline, _ = execution.execution_fingerprint(
            definition, settings, catalog, {}, {"cargo": "first"}, "sha256:fixture-and-generated-contract"
        )
        mutations = [
            ({**definition, "command": ["cargo", "test", "--lib"]}, catalog, {"cargo": "first"}, "sha256:fixture-and-generated-contract"),
            (definition, catalog, {"cargo": "first"}, "sha256:changed-fixture-or-generated-contract"),
            (definition, catalog, {"cargo": "second"}, "sha256:fixture-and-generated-contract"),
            (definition, {"families": {"providers": [{"id": "provider", "sourceHash": "sha256:second"}]}}, {"cargo": "first"}, "sha256:fixture-and-generated-contract"),
        ]
        for changed_definition, changed_catalog, changed_toolchain, changed_inputs in mutations:
            fingerprint, _ = execution.execution_fingerprint(
                changed_definition,
                settings,
                changed_catalog,
                {},
                changed_toolchain,
                changed_inputs,
            )
            self.assertNotEqual(baseline, fingerprint)

    def test_shared_execution_retains_every_attribution(self) -> None:
        shared = {
            "fingerprint": "sha256:same",
            "fingerprintInputs": {"normalizedCommand": ["cargo", "test"]},
            "command": ["cargo", "test"],
            "executionIds": ["proof.one"],
            "artifactIds": ["evidence.one"],
            "attributions": [{"suiteId": "suite.one", "probeIds": ["probe.one"], "assertionIds": ["assertion.one"]}],
        }
        other = {
            **shared,
            "executionIds": ["proof.two"],
            "artifactIds": ["evidence.two"],
            "attributions": [{"suiteId": "suite.two", "probeIds": ["probe.two"], "assertionIds": ["assertion.two"]}],
        }
        grouped = execution.group_equivalent([shared, other])
        self.assertEqual(len(grouped), 1)
        self.assertEqual(grouped[0]["executionIds"], ["proof.one", "proof.two"])
        self.assertEqual(
            [item["suiteId"] for item in grouped[0]["attributions"]],
            ["suite.one", "suite.two"],
        )


if __name__ == "__main__":
    unittest.main()
