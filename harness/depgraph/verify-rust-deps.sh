#!/usr/bin/env bash
# Verifies that no Rust crate in the workspace depends on a crate listed
# under may_not_depend_on in governance/ownership.toml.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python3 - "$REPO_ROOT" <<'PYEOF'
import sys, tomllib, pathlib, re

repo = pathlib.Path(sys.argv[1])
ownership_path = repo / "governance" / "ownership.toml"
engine_rs = repo / "engine-rs"

with open(ownership_path, "rb") as f:
    ownership = tomllib.load(f)

crates = ownership.get("crate", {})
workspace_toml = engine_rs / "Cargo.toml"
with open(workspace_toml, "rb") as f:
    workspace = tomllib.load(f)

failures = []

for rel_path in workspace.get("workspace", {}).get("members", []):
    crate_path = engine_rs / rel_path
    ownership_key = f"engine-rs/{rel_path}"
    crate_meta = crates.get(ownership_key, {})
    forbidden = crate_meta.get("may_not_depend_on", [])
    if not forbidden:
        continue

    cargo_toml = crate_path / "Cargo.toml"
    if not cargo_toml.exists():
        continue
    with open(cargo_toml, "rb") as f:
        crate_cfg = tomllib.load(f)

    actual_deps = set(crate_cfg.get("dependencies", {}).keys())
    for fd in forbidden:
        fd_norm = fd.replace("-", "_")
        for dep in actual_deps:
            if dep.replace("-", "_") == fd_norm:
                failures.append(f"FAIL: {ownership_key} depends on forbidden crate '{fd}'")

if failures:
    for msg in failures:
        print(msg)
    sys.exit(1)
else:
    print("Rust dep graph: OK")
PYEOF
