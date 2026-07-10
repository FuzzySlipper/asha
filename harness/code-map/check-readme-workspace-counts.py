#!/usr/bin/env python3
"""Generate or validate the README workspace inventory counts."""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from typing import NoReturn

DEFAULT_REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
START_MARKER = "<!-- workspace-counts:start -->"
END_MARKER = "<!-- workspace-counts:end -->"


def fail(message: str) -> NoReturn:
    print(f"FAIL: {message}")
    raise SystemExit(1)


def run_json(command: list[str], cwd: pathlib.Path) -> object:
    completed = subprocess.run(
        command,
        cwd=cwd,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if completed.returncode != 0:
        fail(
            f"workspace metadata command failed ({' '.join(command)}):\n"
            f"{completed.stderr.strip()}"
        )
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        fail(f"workspace metadata command returned invalid JSON: {error}")


@dataclass(frozen=True)
class WorkspaceCounts:
    cargo_members: int
    cargo_excluded: int
    pnpm_packages: int

    @property
    def governed_rust_crates(self) -> int:
        return self.cargo_members + self.cargo_excluded


def cargo_counts(repo_root: pathlib.Path) -> tuple[int, int]:
    engine_root = repo_root / "engine-rs"
    manifest_path = engine_root / "Cargo.toml"
    metadata = run_json(
        [
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            str(manifest_path),
        ],
        cwd=repo_root,
    )
    if not isinstance(metadata, dict) or not isinstance(metadata.get("workspace_members"), list):
        fail("cargo metadata omitted workspace_members")
    member_count = len(metadata["workspace_members"])

    workspace_manifest = tomllib.loads(manifest_path.read_text())
    workspace = workspace_manifest.get("workspace")
    if not isinstance(workspace, dict):
        fail("engine-rs/Cargo.toml is missing [workspace]")
    excluded = workspace.get("exclude", [])
    if not isinstance(excluded, list) or not all(isinstance(path, str) for path in excluded):
        fail("engine-rs/Cargo.toml workspace.exclude must be a string array")
    for relative_path in excluded:
        excluded_manifest = engine_root / relative_path / "Cargo.toml"
        if not excluded_manifest.is_file():
            fail(f"excluded explicit-build crate is missing Cargo.toml: engine-rs/{relative_path}")
    return member_count, len(excluded)


def pnpm_package_count(repo_root: pathlib.Path) -> int:
    ts_root = (repo_root / "ts").resolve()
    listing = run_json(
        ["pnpm", "--dir", str(ts_root), "list", "-r", "--depth", "-1", "--json"],
        cwd=repo_root,
    )
    if not isinstance(listing, list):
        fail("pnpm recursive list did not return an array")

    package_paths: set[pathlib.Path] = set()
    root_seen = False
    for record in listing:
        if not isinstance(record, dict) or not isinstance(record.get("path"), str):
            fail("pnpm recursive list contains a record without a path")
        package_path = pathlib.Path(record["path"]).resolve()
        if package_path == ts_root:
            root_seen = True
            continue
        try:
            package_path.relative_to(ts_root)
        except ValueError:
            fail(f"pnpm listed a package outside ts/: {package_path}")
        package_paths.add(package_path)
    if not root_seen:
        fail("pnpm recursive list omitted the ts workspace root")
    return len(package_paths)


def derive_counts(repo_root: pathlib.Path) -> WorkspaceCounts:
    cargo_members, cargo_excluded = cargo_counts(repo_root)
    return WorkspaceCounts(
        cargo_members=cargo_members,
        cargo_excluded=cargo_excluded,
        pnpm_packages=pnpm_package_count(repo_root),
    )


def plural(count: int, singular: str, plural_form: str | None = None) -> str:
    return singular if count == 1 else (plural_form or f"{singular}s")


def generated_block(counts: WorkspaceCounts) -> str:
    return "\n".join(
        [
            START_MARKER,
            (
                "Workspace inventory: "
                f"**{counts.cargo_members} default Cargo workspace "
                f"{plural(counts.cargo_members, 'member')}, "
                f"{counts.cargo_excluded} explicit-build excluded "
                f"{plural(counts.cargo_excluded, 'crate')} "
                f"({counts.governed_rust_crates} governed Rust crates total), and "
                f"{counts.pnpm_packages} pnpm workspace "
                f"{plural(counts.pnpm_packages, 'package')} "
                "(workspace root excluded).**"
            ),
            END_MARKER,
        ]
    )


def replace_block(readme: str, expected: str) -> str:
    pattern = re.compile(
        rf"{re.escape(START_MARKER)}.*?{re.escape(END_MARKER)}",
        re.DOTALL,
    )
    matches = list(pattern.finditer(readme))
    if len(matches) != 1:
        fail(
            "README.md must contain exactly one workspace count marker block; "
            f"found {len(matches)}"
        )
    return pattern.sub(expected, readme, count=1)


def main() -> None:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="validate README counts")
    mode.add_argument("--write", action="store_true", help="update README counts")
    parser.add_argument("--repo-root", type=pathlib.Path, default=DEFAULT_REPO_ROOT)
    args = parser.parse_args()

    repo_root = args.repo_root.resolve()
    readme_path = repo_root / "README.md"
    if not readme_path.is_file():
        fail(f"README.md is missing under {repo_root}")
    counts = derive_counts(repo_root)
    expected_block = generated_block(counts)
    current = readme_path.read_text()
    updated = replace_block(current, expected_block)

    if args.write:
        readme_path.write_text(updated)
        print(
            "updated README workspace counts: "
            f"{counts.cargo_members}+{counts.cargo_excluded} Rust, "
            f"{counts.pnpm_packages} TypeScript"
        )
        return
    if current != updated:
        fail(
            "README.md workspace counts are stale; run "
            "python3 harness/code-map/check-readme-workspace-counts.py --write"
        )
    print(
        "README workspace counts: OK "
        f"({counts.cargo_members} default + {counts.cargo_excluded} excluded Rust; "
        f"{counts.pnpm_packages} TypeScript)"
    )


if __name__ == "__main__":
    main()
