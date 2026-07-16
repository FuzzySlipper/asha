# Shared execution identities and receipts

This directory deduplicates expensive engine-owned gate commands. It is not a
delivery or conformance catalog.

`executions.json` owns normalized command arrays, stable receipt IDs, bounded
claim attribution, and relevant inputs/providers. `execution.py` hashes the
command, selected environment, recursive input contents, generated contracts,
provider identities, toolchain versions, and exact contributing repository
revisions. A changed command, input, toolchain, provider identity, or revision
makes an older receipt stale.

Successful receipts and stdout/stderr logs live under ignored
`harness/smoke-out/execution-receipts/`. Commands with the same fingerprint
share one process while each consuming gate remains attributed to the receipt.
Nothing here claims that Demo or Studio behavior was delivered.

Run the identity and scheduler tests with:

```bash
./harness/ci/check-harness-identities.sh
```
