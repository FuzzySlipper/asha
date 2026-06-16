#!/usr/bin/env python3
"""Validate the Tier 1 public engine facade metadata for downstream consumers.

This is intentionally metadata-only: it prevents a consumer-facing package/crate
from drifting into an unlabeled internal surface, while the depgraph and bridge
checks enforce import/escape-hatch rules.
"""
from __future__ import annotations

import json
import pathlib
import sys
import tomllib
from typing import Any, cast

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]


def fail(message: str) -> None:
    print(f"FAIL: {message}")
    sys.exit(1)


def read_json(path: pathlib.Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def package(name: str) -> dict[str, Any]:
    return read_json(REPO_ROOT / "ts" / "packages" / name / "package.json")


def require_root_only_export(pkg_name: str, pkg: dict[str, Any]) -> None:
    exports_value = pkg.get("exports")
    if not isinstance(exports_value, dict):
        fail(f"{pkg_name} must define package exports")
    exports: dict[str, Any] = cast(dict[str, Any], exports_value)
    if sorted(exports.keys()) != ["."]:
        fail(f"{pkg_name} must expose only the root export; got {sorted(exports.keys())}")


TIER1_TS = {
    "contracts": {
        "expected_name": "@asha/contracts",
        "role": "generated-contracts",
        "may_import_native": False,
    },
    "runtime-bridge": {
        "expected_name": "@asha/runtime-bridge",
        "role": "runtime-facade",
        "may_import_native": True,
    },
}


def check_ts_package(dir_name: str, spec: dict[str, Any]) -> None:
    pkg = package(dir_name)
    if pkg.get("name") != spec["expected_name"]:
        fail(f"ts/packages/{dir_name} package name drifted: {pkg.get('name')}")
    require_root_only_export(spec["expected_name"], pkg)

    asha = pkg.get("asha")
    if not isinstance(asha, dict):
        fail(f"{spec['expected_name']} must declare an 'asha' public-surface metadata block")
    public_surface = asha.get("publicSurface")
    if not isinstance(public_surface, dict):
        fail(f"{spec['expected_name']} must declare asha.publicSurface metadata")
    if public_surface.get("tier") != 1:
        fail(f"{spec['expected_name']} must be marked Tier 1 public surface")
    if public_surface.get("role") != spec["role"]:
        fail(
            f"{spec['expected_name']} role must be {spec['role']!r}, "
            f"got {public_surface.get('role')!r}"
        )
    if "asha-demo" not in public_surface.get("allowedConsumers", []):
        fail(f"{spec['expected_name']} must explicitly allow the asha-demo boundary consumer")
    if public_surface.get("rootExportOnly") is not True:
        fail(f"{spec['expected_name']} must declare rootExportOnly=true")
    if public_surface.get("nativeTransportAccess") is not spec["may_import_native"]:
        fail(f"{spec['expected_name']} nativeTransportAccess metadata drifted")


def check_native_bridge_internal() -> None:
    pkg = package("native-bridge")
    require_root_only_export("@asha/native-bridge", pkg)
    asha = pkg.get("asha")
    if not isinstance(asha, dict) or asha.get("publicSurface") is not False:
        fail("@asha/native-bridge must declare asha.publicSurface=false (raw transport is internal)")
    if asha.get("importedOnlyBy") != ["@asha/runtime-bridge"]:
        fail("@asha/native-bridge must declare importedOnlyBy ['@asha/runtime-bridge']")


def check_runtime_bridge_api_crate() -> None:
    cargo_path = REPO_ROOT / "engine-rs" / "crates" / "bridge" / "runtime-bridge-api" / "Cargo.toml"
    cargo = tomllib.loads(cargo_path.read_text())
    package_meta = cargo.get("package", {}).get("metadata", {}).get("asha", {})
    public_surface = package_meta.get("public-surface")
    if not isinstance(public_surface, dict):
        fail("runtime-bridge-api Cargo.toml must declare package.metadata.asha.public-surface")
    if public_surface.get("tier") != 1:
        fail("runtime-bridge-api must be marked Tier 1 public bridge boundary")
    if public_surface.get("role") != "rust-bridge-boundary":
        fail("runtime-bridge-api public-surface role must be rust-bridge-boundary")
    if public_surface.get("direct-consumer-import") is not False:
        fail("runtime-bridge-api must document that asha-demo v1 should not import it directly")


for dir_name, spec in TIER1_TS.items():
    check_ts_package(dir_name, spec)
check_native_bridge_internal()
check_runtime_bridge_api_crate()
print("Public engine boundary metadata: OK")
