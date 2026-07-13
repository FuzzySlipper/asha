#!/usr/bin/env python3
"""Build the reviewed cross-harness identity catalog from owning manifests."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import sys
import tomllib
from typing import Any, Iterable

ROOT = pathlib.Path(__file__).resolve().parents[2]
CATALOG = ROOT / "harness/identity/catalog.json"
CONFORMANCE = ROOT / "harness/conformance/probe-inventory.json"
REACHABILITY = ROOT / "harness/reachability/manifest.json"
EXECUTIONS = ROOT / "harness/identity/executions.json"


class IdentityError(ValueError):
    """An identity is ambiguous or refers to an identity that does not exist."""


def load_json(path: pathlib.Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def entry_hash(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def indexed(records: Iterable[dict[str, Any]], family: str) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for record in records:
        identity = record.get("id")
        if not isinstance(identity, str) or not identity:
            raise IdentityError(f"{family} contains a missing or invalid id")
        if identity in result:
            raise IdentityError(f"{family} identity collision: {identity}")
        result[identity] = record
    return result


def public_surface_records() -> list[dict[str, Any]]:
    ts = load_json(ROOT / "harness/public-surface/ts-packages.json")
    rust = load_json(ROOT / "harness/public-surface/rust-crates.json")
    records = [
        {
            "id": item["package"],
            "kind": "typescript",
            "disposition": item.get("disposition", "preferred"),
            "sourceHash": entry_hash(item),
        }
        for item in ts["packages"]
    ]
    records.extend(
        {
            "id": item["crate"],
            "kind": "rust",
            "disposition": item["disposition"],
            "sourceHash": entry_hash(item),
        }
        for item in rust["crates"]
    )
    indexed(records, "publicSurfaces")
    return sorted(records, key=lambda item: item["id"])


def consumer_requirement_records() -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for path in sorted((ROOT / "harness/consumer-needs/manifests").glob("*.json")):
        document = load_json(path)
        for item in document["requirements"]:
            records.append({
                "id": item["id"],
                "consumerId": document["consumer"]["id"],
                "identity": item["identity"],
                "kind": item["kind"],
                "providerId": item.get("provider"),
                "requiredLevel": item["requiredLevel"],
                "source": path.relative_to(ROOT).as_posix(),
                "sourceHash": entry_hash(item),
            })
    indexed(records, "consumerRequirements")
    return sorted(records, key=lambda item: item["id"])


def operation_records() -> list[dict[str, Any]]:
    path = ROOT / "engine-rs/crates/bridge/runtime-bridge-api/bridge-manifest.toml"
    document = tomllib.loads(path.read_text(encoding="utf-8"))
    records = [
        {
            "id": item["name"],
            "surface": item["surface"],
            "quarantineReason": item.get("quarantine_reason"),
            "sourceHash": entry_hash(item),
        }
        for item in document["operation"]
    ]
    indexed(records, "operations")
    return sorted(records, key=lambda item: item["id"])


def conformance_records() -> tuple[
    list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]]
]:
    document = load_json(CONFORMANCE)
    suites = [
        {
            "id": item["id"],
            "executionId": item["executionId"],
            "executionClass": item["executionClass"],
            "sourceHash": entry_hash(item),
        }
        for item in document["suites"]
    ]
    probes: list[dict[str, Any]] = []
    assertions: list[dict[str, Any]] = []
    artifacts: list[dict[str, Any]] = []
    for probe in document["semanticProbes"]:
        probes.append({
            "id": probe["id"],
            "suiteId": probe["suite"],
            "sourceHash": entry_hash(probe),
        })
        for evidence in probe["evidence"]:
            assertions.append({
                "id": evidence["assertionId"],
                "probeId": probe["id"],
                "evidenceArtifactId": evidence["artifactId"],
            })
            artifacts.append({
                "id": evidence["artifactId"],
                "kind": "sourceAssertion",
                "path": evidence["path"],
                "token": evidence["token"],
                "sourceHash": entry_hash({"path": evidence["path"], "token": evidence["token"]}),
            })
    indexed(suites, "suites")
    indexed(probes, "probes")
    indexed(assertions, "assertions")
    indexed(artifacts, "evidenceArtifacts")
    return (
        sorted(suites, key=lambda item: item["id"]),
        sorted(probes, key=lambda item: item["id"]),
        sorted(assertions, key=lambda item: item["id"]),
        sorted(artifacts, key=lambda item: item["id"]),
    )


def execution_records() -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    document = load_json(EXECUTIONS)
    executions = document["executions"]
    indexed(executions, "executions")
    artifacts = [
        {
            "id": item["artifactId"],
            "kind": "proofExecution",
            "executionId": item["id"],
        }
        for item in executions
    ]
    indexed(artifacts, "executionEvidenceArtifacts")
    return (
        sorted(
            ({"id": item["id"], "artifactId": item["artifactId"], "sourceHash": entry_hash(item)} for item in executions),
            key=lambda item: item["id"],
        ),
        sorted(artifacts, key=lambda item: item["id"]),
    )


def build_catalog() -> dict[str, Any]:
    reachability = load_json(REACHABILITY)
    requirements = consumer_requirement_records()
    requirement_ids = {item["id"] for item in requirements}
    surfaces = public_surface_records()
    surface_ids = {item["id"] for item in surfaces}
    capabilities: list[dict[str, Any]] = []
    provider_sources: dict[str, list[dict[str, Any]]] = {
        item["id"]: [{"publicSurfaceId": item["id"]}] for item in surfaces
    }
    for item in reachability["capabilities"]:
        surface_id = item.get("publicSurface", {}).get("identity")
        provider_id = item.get("providerIdentity", surface_id)
        need_id = item.get("consumerNeed")
        if need_id is not None and need_id not in requirement_ids:
            raise IdentityError(f"capability {item['id']} references missing consumer requirement {need_id}")
        if surface_id not in surface_ids:
            raise IdentityError(f"capability {item['id']} references missing public surface {surface_id}")
        capabilities.append({
            "id": item["id"],
            "consumerRequirementId": need_id,
            "providerId": provider_id,
            "publicSurfaceId": surface_id,
            "sourceHash": entry_hash(item),
        })
        provider_sources.setdefault(provider_id, []).append({
            "capabilityId": item["id"],
            "evidence": item["provider"],
        })
    indexed(capabilities, "capabilities")
    for item in requirements:
        provider_id = item["providerId"]
        if provider_id is not None:
            provider_sources.setdefault(provider_id, []).append({
                "consumerRequirementId": item["id"],
            })
    providers = [
        {
            "id": identity,
            "references": sorted(sources, key=entry_hash),
            "sourceHash": entry_hash(sorted(sources, key=entry_hash)),
        }
        for identity, sources in provider_sources.items()
    ]
    indexed(providers, "providers")
    suites, probes, assertions, source_artifacts = conformance_records()
    executions, execution_artifacts = execution_records()
    execution_ids = {item["id"] for item in executions}
    for suite in suites:
        if suite["executionId"] not in execution_ids:
            raise IdentityError(
                f"suite {suite['id']} references missing execution {suite['executionId']}"
            )
    families = {
        "assertions": assertions,
        "capabilities": sorted(capabilities, key=lambda item: item["id"]),
        "consumerRequirements": requirements,
        "evidenceArtifacts": sorted(source_artifacts + execution_artifacts, key=lambda item: item["id"]),
        "executions": executions,
        "operations": operation_records(),
        "probes": probes,
        "providers": sorted(providers, key=lambda item: item["id"]),
        "publicSurfaces": surfaces,
        "suites": suites,
    }
    payload = {"schemaVersion": 1, "families": families}
    payload["catalogHash"] = entry_hash(families)
    return payload


def encoded(value: Any) -> str:
    return json.dumps(value, indent=2, sort_keys=False) + "\n"


def validate_catalog() -> tuple[bool, str]:
    try:
        generated = build_catalog()
    except (IdentityError, KeyError, TypeError, json.JSONDecodeError) as error:
        return False, f"identity catalog: {error}"
    if not CATALOG.is_file():
        return False, "identity catalog is missing; run catalog.py --write"
    if CATALOG.read_text(encoding="utf-8") != encoded(generated):
        return False, "identity catalog is stale; run catalog.py --write and review the identity diff"
    return True, (
        "Harness identities: OK "
        f"({len(generated['families']['operations'])} operations, "
        f"{len(generated['families']['capabilities'])} capabilities, "
        f"{len(generated['families']['consumerRequirements'])} consumer requirements, "
        f"{len(generated['families']['providers'])} providers, "
        f"{len(generated['families']['assertions'])} assertions)"
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        CATALOG.write_text(encoded(build_catalog()), encoding="utf-8")
    valid, message = validate_catalog()
    print(message, file=sys.stdout if valid else sys.stderr)
    return 0 if valid else 1


if __name__ == "__main__":
    raise SystemExit(main())
