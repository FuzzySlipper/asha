#!/usr/bin/env python3
"""Derive and validate the real conformance-probe checklist from public catalogs."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import pathlib
import re
import sys
import tempfile
import tomllib
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "harness/conformance/probe-inventory.json"
REPORT = ROOT / "harness/conformance/probe-results.json"
REAL_EXECUTION_CLASSES = {
    "compiledDownstreamConsumer",
    "compiledRustAuthority",
    "nativeTransport",
    "publicConsumer",
}
REQUIRED_CLAIMS = {
    "actualModuleInvocation",
    "atomicRejection",
    "configuredProjectBundleBootstrap",
    "eventBoundIdentity",
    "eventIdentity",
    "fieldSelection",
    "publicProjectionReadout",
    "quota",
    "selectorResolution",
    "stableOrdering",
    "stablePrefabPartResolution",
    "typedViewDelivery",
}


def load_json(path: pathlib.Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def relative(path: pathlib.Path) -> str:
    return path.relative_to(ROOT).as_posix()


def digest(path: pathlib.Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def entry_digest(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def add_gap(gaps: list[dict[str, str]], identity: str, code: str, path: str, message: str) -> None:
    gaps.append({"identity": identity, "code": code, "path": path, "message": message})


def evidence_token(
    gaps: list[dict[str, str]], identity: str, evidence: Any, path: str
) -> None:
    if not isinstance(evidence, dict):
        add_gap(gaps, identity, "invalid_evidence", path, "evidence must be an object")
        return
    source = evidence.get("path")
    token = evidence.get("token")
    if not isinstance(source, str) or not isinstance(token, str) or not source or not token:
        add_gap(gaps, identity, "invalid_evidence", path, "evidence path and token are required")
        return
    file_path = ROOT / source
    if not file_path.is_file():
        add_gap(gaps, identity, "missing_evidence_path", f"{path}.path", f"missing {source}")
    elif token not in file_path.read_text(encoding="utf-8"):
        add_gap(gaps, identity, "missing_evidence_token", f"{path}.token", f"missing {token!r} in {source}")


def bridge_operations() -> tuple[list[str], set[str]]:
    path = ROOT / "engine-rs/crates/bridge/runtime-bridge-api/bridge-manifest.toml"
    document = tomllib.loads(path.read_text(encoding="utf-8"))
    stable = sorted(item["name"] for item in document["operation"] if item["surface"] == "stable")
    native_source = (ROOT / "ts/packages/runtime-bridge/src/native.ts").read_text(encoding="utf-8")
    start = native_source.index("NATIVE_WIRED_OPERATIONS")
    end = native_source.index("function nativeUnimplemented", start)
    wired = set(re.findall(r"'([a-z][a-z0-9_]+)'", native_source[start:end]))
    return stable, wired


def reachability_capabilities() -> set[str]:
    document = load_json(ROOT / "harness/reachability/manifest.json")
    return {item["id"] for item in document["capabilities"]}


def public_surfaces() -> set[str]:
    ts = load_json(ROOT / "harness/public-surface/ts-packages.json")
    rust = load_json(ROOT / "harness/public-surface/rust-crates.json")
    return {item["package"] for item in ts["packages"]} | {item["crate"] for item in rust["crates"]}


def delivery_requirements() -> tuple[set[str], set[str]]:
    needs: set[str] = set()
    surfaces = public_surfaces()
    referenced_surfaces: set[str] = set()
    for path in sorted((ROOT / "harness/consumer-needs/manifests").glob("*.json")):
        for item in load_json(path)["requirements"]:
            if item["requiredLevel"] != "delivery":
                continue
            needs.add(item["id"])
            for candidate in (item.get("identity"), item.get("provider")):
                if candidate in surfaces:
                    referenced_surfaces.add(candidate)
    reachability = load_json(ROOT / "harness/reachability/manifest.json")
    for capability in reachability["capabilities"]:
        surface = capability.get("publicSurface", {}).get("identity")
        if surface in surfaces:
            referenced_surfaces.add(surface)
    return needs, referenced_surfaces


def required_catalog_entry_hashes() -> dict[str, dict[str, str]]:
    reachability = load_json(ROOT / "harness/reachability/manifest.json")
    capability_hashes = {
        item["id"]: entry_digest(item) for item in reachability["capabilities"]
    }
    need_hashes: dict[str, str] = {}
    for path in sorted((ROOT / "harness/consumer-needs/manifests").glob("*.json")):
        for item in load_json(path)["requirements"]:
            if item["requiredLevel"] == "delivery":
                need_hashes[item["id"]] = entry_digest(item)
    required_needs, required_surfaces = delivery_requirements()
    surface_entries: dict[str, Any] = {}
    ts = load_json(ROOT / "harness/public-surface/ts-packages.json")
    rust = load_json(ROOT / "harness/public-surface/rust-crates.json")
    surface_entries.update({item["package"]: item for item in ts["packages"]})
    surface_entries.update({item["crate"]: item for item in rust["crates"]})
    assert set(need_hashes) == required_needs
    return {
        "reachabilityCapabilities": dict(sorted(capability_hashes.items())),
        "deliveryRequirements": dict(sorted(need_hashes.items())),
        "publicSurfaces": {
            identity: entry_digest(surface_entries[identity])
            for identity in sorted(required_surfaces)
        },
    }


def validate(manifest_path: pathlib.Path) -> dict[str, Any]:
    document = load_json(manifest_path)
    gaps: list[dict[str, str]] = []
    if document.get("schemaVersion") != 1:
        add_gap(gaps, "manifest", "unsupported_schema", "schemaVersion", "schemaVersion must be 1")

    expected_hashes = document.get("catalogEntryHashes")
    actual_hashes = required_catalog_entry_hashes()
    if not isinstance(expected_hashes, dict):
        add_gap(gaps, "manifest", "missing_catalog_entry_hashes", "catalogEntryHashes", "reviewed catalog entry hashes are required")
    else:
        for family, actual_entries in actual_hashes.items():
            expected_entries = expected_hashes.get(family)
            if not isinstance(expected_entries, dict):
                add_gap(gaps, family, "missing_catalog_hash_family", f"catalogEntryHashes.{family}", "catalog hash family is required")
                continue
            for identity in sorted(set(actual_entries) | set(expected_entries)):
                if identity not in actual_entries:
                    add_gap(gaps, identity, "stale_catalog_entry_hash", f"catalogEntryHashes.{family}", "hash remains for an entry no longer requiring a probe")
                elif identity not in expected_entries:
                    add_gap(gaps, identity, "unreviewed_catalog_entry", f"catalogEntryHashes.{family}", "new catalog entry requires a reviewed semantic probe and hash")
                elif expected_entries[identity] != actual_entries[identity]:
                    add_gap(gaps, identity, "catalog_entry_changed", f"catalogEntryHashes.{family}.{identity}", "fields, selectors, quotas, provider, or delivery evidence changed; review the real probe")

    suites = document.get("suites", [])
    suite_ids = [item.get("id") for item in suites if isinstance(item, dict)]
    if suite_ids != sorted(suite_ids) or len(suite_ids) != len(set(suite_ids)):
        add_gap(gaps, "manifest", "noncanonical_suites", "suites", "suite ids must be sorted and unique")
    suite_map = {item["id"]: item for item in suites if isinstance(item, dict) and isinstance(item.get("id"), str)}
    for index, suite in enumerate(suites):
        identity = suite.get("id", f"suites[{index}]") if isinstance(suite, dict) else f"suites[{index}]"
        if not isinstance(suite, dict):
            add_gap(gaps, identity, "invalid_suite", f"suites[{index}]", "suite must be an object")
            continue
        if suite.get("executionClass") not in REAL_EXECUTION_CLASSES:
            add_gap(gaps, identity, "mock_or_schema_suite", "executionClass", "suite must execute real compiled, native, or public-consumer code")
        if not isinstance(suite.get("command"), str) or not suite["command"].strip():
            add_gap(gaps, identity, "missing_suite_command", "command", "suite command is required")

    corpora = document.get("testCorpora", [])
    corpus_text: dict[str, list[str]] = {}
    for index, corpus in enumerate(corpora):
        identity = corpus.get("id", f"testCorpora[{index}]") if isinstance(corpus, dict) else f"testCorpora[{index}]"
        if not isinstance(corpus, dict):
            add_gap(gaps, identity, "invalid_corpus", f"testCorpora[{index}]", "corpus must be an object")
            continue
        if corpus.get("executionClass") not in {"compiledRustAuthority", "nativeTransport"}:
            add_gap(gaps, identity, "non_real_operation_corpus", "executionClass", "operation evidence must be compiled Rust authority or native transport")
        texts: list[str] = []
        for source in corpus.get("paths", []):
            path = ROOT / source
            if not path.is_file():
                add_gap(gaps, identity, "missing_corpus_path", "paths", f"missing {source}")
            else:
                texts.append(path.read_text(encoding="utf-8"))
        corpus_text[identity] = texts

    stable, native_wired = bridge_operations()
    native_export_text = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted((ROOT / "engine-rs/crates/bridge/native-bridge/src").glob("*.rs"))
    )
    exemptions = document.get("temporaryOperationExemptions", [])
    exemption_names = [item.get("operation") for item in exemptions if isinstance(item, dict)]
    if exemption_names != sorted(exemption_names) or len(exemption_names) != len(set(exemption_names)):
        add_gap(gaps, "manifest", "noncanonical_operation_exemptions", "temporaryOperationExemptions", "operation exemptions must be sorted and unique")
    exemption_map = {item["operation"]: item for item in exemptions if isinstance(item, dict) and isinstance(item.get("operation"), str)}
    unknown_exemptions = sorted(set(exemption_map) - set(stable))
    for operation in unknown_exemptions:
        add_gap(gaps, operation, "unknown_operation_exemption", "temporaryOperationExemptions", "exemption is not a stable operation")

    operation_results: list[dict[str, Any]] = []
    for operation in stable:
        evidence_corpora = sorted(
            corpus_id
            for corpus_id, texts in corpus_text.items()
            if any(operation in text for text in texts)
        )
        exemption = exemption_map.get(operation)
        if operation in native_wired:
            if exemption is not None:
                add_gap(gaps, operation, "stale_operation_exemption", "temporaryOperationExemptions", "native-wired operation must use its real probe")
            if not evidence_corpora:
                add_gap(gaps, operation, "stable_operation_without_real_probe", "testCorpora", "native-wired stable operation lacks compiled semantic evidence")
            if re.search(rf"#\[napi\]\s*pub fn\s+{re.escape(operation)}\s*\(", native_export_text) is None:
                add_gap(gaps, operation, "native_provider_not_exported", "engine-rs/crates/bridge/native-bridge/src", "NATIVE_WIRED_OPERATIONS entry has no concrete #[napi] export")
            status = "probed"
        else:
            if exemption is None:
                add_gap(gaps, operation, "stable_operation_without_real_probe", "temporaryOperationExemptions", "unwired stable operation needs an explicit temporary exemption")
                status = "gap"
            else:
                for field in ("owner", "reason", "exitCriteria"):
                    value = exemption.get(field)
                    if not isinstance(value, str) or len(value.strip()) < (40 if field != "owner" else 1):
                        add_gap(gaps, operation, "invalid_operation_exemption", field, f"specific {field} is required")
                evidence_token(gaps, operation, exemption.get("evidence"), "evidence")
                status = "temporaryExemption"
        operation_results.append({
            "operation": operation,
            "status": status,
            "nativeWired": operation in native_wired,
            "evidenceCorpora": evidence_corpora,
        })

    probes = document.get("semanticProbes", [])
    probe_ids = [item.get("id") for item in probes if isinstance(item, dict)]
    if probe_ids != sorted(probe_ids) or len(probe_ids) != len(set(probe_ids)):
        add_gap(gaps, "manifest", "noncanonical_semantic_probes", "semanticProbes", "probe ids must be sorted and unique")
    covered_capabilities: set[str] = set()
    covered_needs: set[str] = set()
    covered_surfaces: set[str] = set()
    covered_claims: set[str] = set()
    probe_results: list[dict[str, Any]] = []
    for index, probe in enumerate(probes):
        identity = probe.get("id", f"semanticProbes[{index}]") if isinstance(probe, dict) else f"semanticProbes[{index}]"
        before = len(gaps)
        if not isinstance(probe, dict):
            add_gap(gaps, identity, "invalid_probe", f"semanticProbes[{index}]", "probe must be an object")
            continue
        suite = suite_map.get(probe.get("suite"))
        if suite is None:
            add_gap(gaps, identity, "unknown_probe_suite", "suite", "probe must reference a declared suite")
        elif suite.get("executionClass") not in REAL_EXECUTION_CLASSES:
            add_gap(gaps, identity, "mock_only_probe", "suite", "mock-only and schema-only suites cannot satisfy conformance")
        evidence = probe.get("evidence", [])
        if not isinstance(evidence, list) or not evidence:
            add_gap(gaps, identity, "missing_probe_evidence", "evidence", "at least one semantic assertion is required")
        else:
            for evidence_index, item in enumerate(evidence):
                evidence_token(gaps, identity, item, f"evidence[{evidence_index}]")
        covered_capabilities.update(probe.get("capabilities", []))
        covered_needs.update(probe.get("consumerNeeds", []))
        covered_surfaces.update(probe.get("publicSurfaces", []))
        covered_claims.update(probe.get("claims", []))
        probe_results.append({"id": identity, "suite": probe.get("suite"), "passed": len(gaps) == before})

    required_capabilities = reachability_capabilities()
    required_needs, required_surfaces = delivery_requirements()
    requirements = [
        ("capability", required_capabilities, covered_capabilities),
        ("consumer_need", required_needs, covered_needs),
        ("public_surface", required_surfaces, covered_surfaces),
        ("semantic_claim", REQUIRED_CLAIMS, covered_claims),
    ]
    for kind, required, covered in requirements:
        for missing in sorted(required - covered):
            add_gap(gaps, missing, f"unprobed_{kind}", "semanticProbes", f"required {kind.replace('_', ' ')} has no real semantic probe")
        for extra in sorted(covered - required):
            add_gap(gaps, extra, f"unknown_{kind}", "semanticProbes", f"probe claims unknown {kind.replace('_', ' ')}")

    gaps.sort(key=lambda item: (item["identity"], item["code"], item["path"], item["message"]))
    catalog_paths = [
        ROOT / "engine-rs/crates/bridge/runtime-bridge-api/bridge-manifest.toml",
        ROOT / "harness/consumer-needs/validation-report.json",
        ROOT / "harness/public-surface/rust-crates.json",
        ROOT / "harness/public-surface/ts-packages.json",
        ROOT / "harness/reachability/validation-report.json",
        ROOT / "ts/packages/runtime-bridge/src/native.ts",
    ]
    return {
        "schemaVersion": 1,
        "valid": not gaps,
        "inventory": relative(manifest_path),
        "inventoryHash": digest(manifest_path),
        "catalogs": [{"path": relative(path), "hash": digest(path)} for path in catalog_paths],
        "summary": {
            "stableOperationCount": len(stable),
            "realOperationProbeCount": sum(item["status"] == "probed" for item in operation_results),
            "temporaryOperationExemptionCount": sum(item["status"] == "temporaryExemption" for item in operation_results),
            "semanticProbeCount": len(probe_results),
            "deliveryRequirementCount": len(required_needs),
            "publicSurfaceCount": len(required_surfaces),
        },
        "operations": operation_results,
        "semanticProbes": probe_results,
        "gaps": gaps,
    }


def encoded(report: dict[str, Any]) -> str:
    return json.dumps(report, indent=2) + "\n"


def apply_fixture_mutation(document: dict[str, Any], fixture: dict[str, Any]) -> None:
    action = fixture["action"]
    identity = fixture["identity"]
    if action == "removeOperationExemption":
        document["temporaryOperationExemptions"] = [
            item for item in document["temporaryOperationExemptions"]
            if item["operation"] != identity
        ]
    elif action == "makeSuiteMockOnly":
        next(item for item in document["suites"] if item["id"] == identity)["executionClass"] = "schemaOnly"
    elif action == "removeCapabilityCoverage":
        for probe in document["semanticProbes"]:
            probe["capabilities"] = [item for item in probe["capabilities"] if item != identity]
    elif action == "removeConsumerNeedCoverage":
        for probe in document["semanticProbes"]:
            probe["consumerNeeds"] = [item for item in probe["consumerNeeds"] if item != identity]
    elif action == "removePublicSurfaceCoverage":
        for probe in document["semanticProbes"]:
            probe["publicSurfaces"] = [item for item in probe["publicSurfaces"] if item != identity]
    elif action == "removeSemanticClaim":
        for probe in document["semanticProbes"]:
            probe["claims"] = [item for item in probe["claims"] if item != identity]
    elif action == "changeCatalogEntryHash":
        family, entry = identity.split("/", 1)
        document["catalogEntryHashes"][family][entry] = "sha256:stale"
    elif action == "breakEvidenceToken":
        probe = next(item for item in document["semanticProbes"] if item["id"] == identity)
        probe["evidence"][0]["token"] = "missing-conformance-token"
    else:
        raise ValueError(f"unknown fixture action {action!r}")


def check_fixtures() -> int:
    base = load_json(MANIFEST)
    fixture_dir = ROOT / "harness/conformance/fixtures"
    failures: list[str] = []
    fixtures = sorted(fixture_dir.glob("*.json"))
    for fixture_path in fixtures:
        fixture = load_json(fixture_path)
        document = copy.deepcopy(base)
        apply_fixture_mutation(document, fixture)
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".json", dir=MANIFEST.parent, delete=False, encoding="utf-8"
        ) as temporary:
            json.dump(document, temporary)
            temporary_path = pathlib.Path(temporary.name)
        try:
            report = validate(temporary_path)
        finally:
            temporary_path.unlink(missing_ok=True)
        codes = {item["code"] for item in report["gaps"]}
        if fixture["expectedCode"] not in codes:
            failures.append(
                f"{fixture_path.name}: expected {fixture['expectedCode']}, got {sorted(codes)}"
            )
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"conformance fixtures: OK ({len(fixtures)} negative fixtures)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=pathlib.Path, default=MANIFEST)
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--check-fixtures", action="store_true")
    args = parser.parse_args()
    if args.check_fixtures:
        return check_fixtures()
    report = validate(args.manifest.resolve())
    if args.write_report and args.manifest.resolve() == MANIFEST:
        REPORT.write_text(encoded(report), encoding="utf-8")
    elif args.manifest.resolve() == MANIFEST:
        if not REPORT.is_file() or REPORT.read_text(encoding="utf-8") != encoded(report):
            print("conformance: probe-results.json is stale; run validate.py --write-report", file=sys.stderr)
            return 1
    if not report["valid"]:
        print(encoded(report), file=sys.stderr)
        return 1
    summary = report["summary"]
    print(
        "Conformance inventory: OK "
        f"({summary['realOperationProbeCount']} real stable operations, "
        f"{summary['temporaryOperationExemptionCount']} temporary exemptions, "
        f"{summary['semanticProbeCount']} semantic probes)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
