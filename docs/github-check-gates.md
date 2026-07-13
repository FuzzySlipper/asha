# GitHub Check Gates

ASHA runs the same repo gate locally and in GitHub Actions:

```bash
./harness/ci/check-all.sh
```

The workflow is `.github/workflows/offline-ci.yml`. It runs automatically on
pushes to `main`, pull requests, and manual dispatch.

`check-all.sh` owns the semantic gate inventory. In particular,
`check-gameplay-runtime-host.sh` is the single execution owner for direct host
tests and its targeted one-cell provider lifecycle proof. The earlier Rust
workspace pass excludes that crate only under `check-all.sh`; standalone
`check-rust.sh` remains complete. The runtime-host gate records bounded local
evidence under `harness/smoke-out/gameplay-runtime-host/`.

For Den Review GitHub check gates, use:

```json
{
  "project_id": "asha",
  "task_id": "<den-task-id>",
  "repository": "FuzzySlipper/asha-engine",
  "commit_sha": "<full-40-character-sha>",
  "ref": "main",
  "required_checks": ["Verify ASHA"],
  "requested_by": "<agent-name>"
}
```

Agents should register the exact pushed commit SHA after a task commit is
pushed. The Den service records pass, fail, timeout, or superseded evidence on
the task thread; GitHub Actions remains the runner.

`Verify ASHA` is the GitHub Actions job/check-run name from
`.github/workflows/offline-ci.yml`. Do not use the workflow file name as the
required check.
