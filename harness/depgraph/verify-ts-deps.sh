#!/usr/bin/env bash
# Verifies that each TypeScript package's internal @asha/* imports are listed
# under may_import in governance/ownership.toml, and that every package has an
# ownership entry unless explicitly exempted.
set -euo pipefail

REPO_ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

python3 - "$REPO_ROOT" <<'PYEOF'
import sys, tomllib, pathlib, json, re

repo = pathlib.Path(sys.argv[1])
ownership_path = repo / "governance" / "ownership.toml"
ts_packages = repo / "ts" / "packages"

with open(ownership_path, "rb") as f:
    ownership = tomllib.load(f)

packages = ownership.get("package", {})
failures = []
ownership_exempt = set(ownership.get("ownership_exempt", {}).get("packages", []))

actual_packages: dict[str, tuple[str, pathlib.Path]] = {}
for pkg_dir in sorted(ts_packages.iterdir()):
    if not pkg_dir.is_dir():
        continue
    pkg_json = pkg_dir / "package.json"
    if not pkg_json.exists():
        continue
    data = json.loads(pkg_json.read_text())
    package_name = data.get("name")
    if package_name:
        actual_packages[f"ts/packages/{pkg_dir.name}"] = (package_name, pkg_dir)


def package_name_for_key(ownership_key: str) -> str:
    package_dir_name = ownership_key.rsplit("/", 1)[-1]
    return f"@asha/{package_dir_name}"


known_asha_packages = {name for name, _pkg_dir in actual_packages.values()}
known_asha_packages.update(package_name_for_key(key) for key in packages)


def collect_source_imports(pkg_dir: pathlib.Path, package_name: str) -> set[str]:
    imports_found: set[str] = set()
    src_dir = pkg_dir / "src"
    if not src_dir.exists():
        return imports_found

    import_re = re.compile(
        r"(?:from\s+|import\s+(?:type\s+)?|import\s*\(\s*)"
        r"[\"'](@asha/[a-z0-9-]+)(?:/[^\"']*)?[\"']"
    )
    for src_file in src_dir.rglob("*.ts"):
        text = src_file.read_text()
        for match in import_re.finditer(text):
            imported_package = match.group(1)
            if imported_package != package_name and imported_package in known_asha_packages:
                imports_found.add(imported_package)
    return imports_found


def collect_manifest_imports(pkg_dir: pathlib.Path, package_name: str) -> set[str]:
    imports_found: set[str] = set()
    pkg_json = pkg_dir / "package.json"
    if not pkg_json.exists():
        return imports_found
    data = json.loads(pkg_json.read_text())
    for section in ("dependencies", "devDependencies", "peerDependencies"):
        for dep in data.get(section, {}):
            if dep.startswith("@asha/") and dep != package_name:
                imports_found.add(dep)
    return imports_found


for ownership_key, (package_name, pkg_dir) in actual_packages.items():
    if ownership_key not in packages and ownership_key not in ownership_exempt:
        failures.append(f"FAIL: {ownership_key} has no ownership entry in governance/ownership.toml")
        continue

    pkg_meta = packages.get(ownership_key, {})
    pkg_lane = pkg_meta.get("lane", "?")
    allowed = set(pkg_meta.get("may_import", []))
    forbidden = set(pkg_meta.get("may_not_import", []))
    imports_found = collect_source_imports(pkg_dir, package_name)
    imports_found.update(collect_manifest_imports(pkg_dir, package_name))

    for dep in sorted(allowed & forbidden):
        failures.append(f"FAIL: {ownership_key} lists '{dep}' in both may_import and may_not_import.")

    for dep in sorted(imports_found):
        target_short = dep.split("/", 1)[-1]
        target_lane = packages.get(f"ts/packages/{target_short}", {}).get("lane", "?")
        if dep in forbidden:
            failures.append(
                f"FAIL: {ownership_key} (lane {pkg_lane}) imports forbidden package "
                f"'{dep}' (lane {target_lane}).\n"
                f"      Route this through the contract border or move the dependency "
                f"into a {target_lane} package — do not relax the boundary."
            )
            continue
        if dep not in allowed:
            failures.append(
                f"FAIL: {ownership_key} (lane {pkg_lane}) imports unlisted internal "
                f"package '{dep}' (lane {target_lane}).\n"
                f"      Add it to governance/ownership.toml may_import only if this is an "
                f"approved package boundary; otherwise route through the existing public API."
            )

if failures:
    for msg in failures:
        print(msg)
    sys.exit(1)
else:
    print("TypeScript dep graph: OK")
PYEOF
