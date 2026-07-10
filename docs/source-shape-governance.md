# Source-Shape Governance

ASHA source-shape limits keep ownership cells inspectable and prevent large bridge, protocol,
authority, or shell files from becoming default stuffing points. The normal CI path checks both
current file sizes and changes to exemption baselines.

## Shrink-only defaults

- A file without an exemption must stay within the policy's global source-line limit.
- An exempt file may shrink without changing policy metadata.
- An unchanged exemption requires no metadata churn.
- Raising the global TypeScript source limit is forbidden. Split the source instead.
- Adding an exemption or increasing `maxLines` is temporary debt, not permission for continued
  growth.

## Audited TypeScript baseline changes

Every new TypeScript exemption and every increase to an existing `maxLines` must add or refresh a
`baselineChange` object on that exemption:

```json
{
  "baselineChange": {
    "changedAt": "2026-07-09",
    "changeTask": "#5505",
    "reason": "A reviewed temporary increase is required while the focused split proceeds.",
    "removalTask": "#5506",
    "previousMaxLines": 1810,
    "newMaxLines": 1820
  }
}
```

For a new exemption, `previousMaxLines` is `null`. `changeTask` identifies the work introducing the
temporary increase. `removalTask` must name a distinct scheduled task that will split the source or
remove the exemption. Git history preserves earlier change records; a later raise must refresh the
object with the exact new before/after limits.

Existing exemptions without `baselineChange` predate this policy. They remain valid only at or below
their recorded baseline. Do not manufacture historical metadata for them.

## Revision selection

`check-ts-source-shape-policy-diff.mjs` compares the current policy with:

1. `ASHA_SOURCE_SHAPE_BASE_REF` when CI supplies the push or pull-request base SHA;
2. `HEAD` when the local policy has uncommitted changes; or
3. `HEAD^` for a clean committed change.

The fixture suite can provide explicit policy files. A repository with no available base revision
skips only the revision audit; current source-shape validation still runs.

Run the normal gates with:

```bash
./harness/ci/check-ts.sh
./harness/ci/check-all.sh
```
