#!/usr/bin/env python3
"""Prove generated atlas evidence counts ignore ambient worktree files."""

from __future__ import annotations

import importlib.util
import pathlib
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
ATLAS_PATH = ROOT / "harness" / "code-map" / "check-agent-code-atlas.py"


def load_atlas_module():
    specification = importlib.util.spec_from_file_location("agent_code_atlas", ATLAS_PATH)
    if specification is None or specification.loader is None:
        raise RuntimeError("could not load Agent Code Atlas module")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def main() -> None:
    atlas = load_atlas_module()
    baseline = atlas.render_evidence_inventory()
    fixtures = ROOT / "harness" / "fixtures"
    existing_group = fixtures / "gameplay-module-sdk"

    with tempfile.TemporaryDirectory(
        prefix=".agent-code-atlas-untracked-", dir=existing_group
    ) as temporary:
        pathlib.Path(temporary, "ambient.txt").write_text("not committed\n")
        if atlas.render_evidence_inventory() != baseline:
            raise SystemExit("ambient file changed committed evidence counts")

    with tempfile.TemporaryDirectory(
        prefix=".agent-code-atlas-untracked-group-", dir=fixtures
    ) as temporary:
        pathlib.Path(temporary, "ambient.txt").write_text("not committed\n")
        if atlas.render_evidence_inventory() != baseline:
            raise SystemExit("ambient group changed committed evidence groups")

    print("Agent Code Atlas fixtures: OK (ambient files and groups ignored)")


if __name__ == "__main__":
    main()
