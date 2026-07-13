#!/usr/bin/env python3
"""Negative tests for bridge manifest generation and exact wiring inventories."""

from __future__ import annotations

import copy
import importlib.util
import pathlib
import sys
import tomllib

REPO = pathlib.Path(__file__).resolve().parents[2]
MANIFEST = REPO / "engine-rs/crates/bridge/runtime-bridge-api/bridge-manifest.toml"
sys.dont_write_bytecode = True


def load_module(name: str, path: pathlib.Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def expect_invalid(emitter, manifest, operations, capabilities, expected_fragment: str) -> None:
    try:
        emitter.validate_model(manifest, operations, capabilities)
    except ValueError as cause:
        if expected_fragment not in str(cause):
            raise AssertionError(
                f"expected validation error containing {expected_fragment!r}, got {cause!r}"
            ) from cause
        return
    raise AssertionError(f"expected validation failure containing {expected_fragment!r}")


def main() -> int:
    emitter = load_module("bridge_emit", REPO / "harness/codegen/bridge-emit.py")
    validator = load_module(
        "bridge_manifest_validator", REPO / "harness/bridge/validate-manifest.py"
    )
    with MANIFEST.open("rb") as source:
        document = tomllib.load(source)
    manifest = document["manifest"]
    operations = document["operation"]
    capabilities = document["capability"]

    missing = copy.deepcopy(capabilities)
    removed = missing[0]["operations"].pop()
    expect_invalid(emitter, manifest, operations, missing, f"missing=['{removed}']")

    duplicate = copy.deepcopy(capabilities)
    duplicated = duplicate[0]["operations"][0]
    duplicate[1]["operations"].append(duplicated)
    expect_invalid(emitter, manifest, operations, duplicate, f"duplicate=['{duplicated}']")

    duplicate_operation = copy.deepcopy(operations)
    duplicate_operation.append(copy.deepcopy(operations[0]))
    expect_invalid(emitter, manifest, duplicate_operation, capabilities, "must be unique")

    missing_errors = copy.deepcopy(manifest)
    missing_errors["error_families"] = []
    expect_invalid(emitter, missing_errors, operations, capabilities, "non-empty unique list")

    inventory_errors: list[str] = []
    validator.validate_exact_inventory(
        inventory_errors, "fixture bindings", {"alpha", "beta"}, ["alpha", "alpha", "gamma"]
    )
    if not any("duplicate" in error for error in inventory_errors):
        raise AssertionError("duplicate binding did not fail exact-inventory validation")
    if not any("beta" in error and "missing" in error for error in inventory_errors):
        raise AssertionError("unwired binding did not fail exact-inventory validation")
    if not any("gamma" in error and "non-manifest" in error for error in inventory_errors):
        raise AssertionError("mismatched binding did not fail exact-inventory validation")

    print("Bridge codegen negative fixtures: OK (missing, duplicate, mismatch, unwired)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
