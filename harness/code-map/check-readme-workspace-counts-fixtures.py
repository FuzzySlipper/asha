#!/usr/bin/env python3
"""Negative fixtures for generated README workspace counts."""
from __future__ import annotations

import pathlib
import subprocess
import tempfile

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
CHECKER = REPO_ROOT / "harness" / "code-map" / "check-readme-workspace-counts.py"
MARKERS = "<!-- workspace-counts:start -->\nstale\n<!-- workspace-counts:end -->\n"


def write(path: pathlib.Path, contents: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(contents)


def run(root: pathlib.Path, mode: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(CHECKER), mode, "--repo-root", str(root)],
        cwd=REPO_ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


def assert_stale(result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode == 0:
        raise SystemExit(f"FAIL: {label} unexpectedly passed")
    if "README.md workspace counts are stale" not in result.stdout or "--write" not in result.stdout:
        raise SystemExit(f"FAIL: {label} lacked actionable stale-count output:\n{result.stdout}")
    print(f"README workspace-count negative fixture OK: {label}")


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="asha-readme-counts-") as temp_dir:
        root = pathlib.Path(temp_dir)
        write(root / "README.md", f"# Fixture\n\n{MARKERS}")
        write(
            root / "engine-rs" / "Cargo.toml",
            '[workspace]\nresolver = "2"\nmembers = ["crates/a"]\nexclude = ["crates/native"]\n',
        )
        package_manifest = '[package]\nname = "{}"\nversion = "0.1.0"\nedition = "2021"\n'
        write(root / "engine-rs" / "crates" / "a" / "Cargo.toml", package_manifest.format("a"))
        write(root / "engine-rs" / "crates" / "a" / "src" / "lib.rs", "")
        write(
            root / "engine-rs" / "crates" / "native" / "Cargo.toml",
            package_manifest.format("native"),
        )
        write(root / "engine-rs" / "crates" / "native" / "src" / "lib.rs", "")
        write(root / "ts" / "pnpm-workspace.yaml", "packages:\n  - 'packages/*'\n")
        write(root / "ts" / "package.json", '{"name":"fixture-root","private":true}\n')
        write(root / "ts" / "packages" / "a" / "package.json", '{"name":"@fixture/a"}\n')

        generated = run(root, "--write")
        if generated.returncode != 0:
            raise SystemExit(f"FAIL: fixture setup could not generate counts:\n{generated.stdout}")
        baseline = run(root, "--check")
        if baseline.returncode != 0:
            raise SystemExit(f"FAIL: generated fixture baseline failed:\n{baseline.stdout}")

        cargo_manifest = root / "engine-rs" / "Cargo.toml"
        cargo_manifest.write_text(cargo_manifest.read_text().replace(
            'members = ["crates/a"]',
            'members = ["crates/a", "crates/b"]',
        ))
        write(root / "engine-rs" / "crates" / "b" / "Cargo.toml", package_manifest.format("b"))
        write(root / "engine-rs" / "crates" / "b" / "src" / "lib.rs", "")
        assert_stale(run(root, "--check"), "added Cargo workspace member")

        refreshed = run(root, "--write")
        if refreshed.returncode != 0:
            raise SystemExit(f"FAIL: Cargo fixture refresh failed:\n{refreshed.stdout}")
        write(root / "ts" / "packages" / "b" / "package.json", '{"name":"@fixture/b"}\n')
        assert_stale(run(root, "--check"), "added pnpm workspace package")


if __name__ == "__main__":
    main()
