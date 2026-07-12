#!/usr/bin/env python3
"""Execute every declared real conformance suite without shell interpolation."""

from __future__ import annotations

import json
import pathlib
import shlex
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
INVENTORY = ROOT / "harness/conformance/probe-inventory.json"


def main() -> int:
    document = json.loads(INVENTORY.read_text(encoding="utf-8"))
    for suite in document["suites"]:
        command = shlex.split(suite["command"])
        print(f"==> conformance {suite['id']}: {' '.join(command)}", flush=True)
        completed = subprocess.run(command, cwd=ROOT, check=False)
        if completed.returncode != 0:
            print(f"conformance suite {suite['id']} failed", file=sys.stderr)
            return completed.returncode
    print(f"Real conformance probes passed ({len(document['suites'])} suites).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
