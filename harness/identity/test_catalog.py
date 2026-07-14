#!/usr/bin/env python3
"""Collision tests for the merged cross-harness identity catalog."""

from __future__ import annotations

import unittest

import catalog


class IdentityCatalogTests(unittest.TestCase):
    def test_evidence_artifact_collision_across_layers_is_rejected(self) -> None:
        source_artifact = {
            "id": "evidence.shared",
            "kind": "sourceAssertion",
        }
        execution_artifact = {
            "id": "evidence.shared",
            "kind": "proofExecution",
        }

        with self.assertRaisesRegex(
            catalog.IdentityError, "evidenceArtifacts identity collision"
        ):
            catalog.merged_evidence_artifacts(
                [source_artifact], [execution_artifact]
            )


if __name__ == "__main__":
    unittest.main()
