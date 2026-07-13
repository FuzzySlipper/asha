# Harness identities and proof execution

This directory joins harness records without merging their authorities:

- consumer-needs decides what a downstream role requires;
- reachability decides whether a requirement has a public provider path;
- conformance decides whether a real assertion exercises that path;
- proof execution decides whether the declared command passed for the current inputs.

`catalog.json` is a committed, generated cross-reference. It assigns stable IDs to operations, requirements, capabilities, providers, public surfaces, suites, probes, assertions, executions, and evidence artifacts. The owning manifests still author their distinct decisions. Run:

```bash
python3 harness/identity/catalog.py --write
```

when an owning identity changes, then review the generated catalog diff. Normal gates run `catalog.py` without `--write` and reject a stale catalog. Bridge operation records are generated from `bridge-manifest.toml`; the catalog therefore follows the current 71-operation inventory (68 stable and 3 quarantined) rather than a hard-coded historical count.

`executions.json` owns normalized command arrays and their relevant input/provider sets. `execution.py` hashes the command, selected environment, recursive input contents, generated contracts, provider records, and toolchain versions. Successful receipts and stdout/stderr logs live under ignored `harness/smoke-out/proof-execution/`. Suites with the same fingerprint share one process while every suite, probe, assertion, and evidence artifact remains visible in the receipt and execution report.

Run the identity and negative gates with:

```bash
./harness/ci/check-harness-identities.sh
```
