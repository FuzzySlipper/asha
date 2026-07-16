# Consumer needs manifests

Consumer needs are role-scoped, versioned records of what downstream code
actually consumes. They complement the engine-owned public-surface manifests:
the public manifests say what a role may import, while needs manifests say what
that consumer expects to exist and how strongly it must be proved.

The schema is `harness/consumer-needs/schema.json`. Initial observed migrations
live under `harness/consumer-needs/manifests/` for the current `asha-demo` browser
game and the compiled pulse gameplay-module fixture.

## Proof ladder

Every requirement names one required proof level and carries four separate
evidence lists:

1. `type` — the package, crate, generated type, or contract shape exists.
2. `provider` — a named provider advertises that it can supply the need.
3. `selector` — requested fields, selectors, quotas, ordering, or target roles
   are supported by that provider.
4. `delivery` — the owning consumer executed the real user/product path and
   delivered the need.

Higher levels require evidence for every earlier level. A generated type alone
therefore cannot satisfy a provider or delivery claim. Engine-local provider
tests may satisfy provider/selector behavior, but they cannot satisfy Studio or
Game Project delivery. Missing consumer execution is `not_run` or `unavailable`,
never passed.

## Guardrails

- TypeScript package and Rust crate requirements are checked against independent
  role allowlists in `harness/public-surface/`.
- Rust facade symbol requirements must appear in the facade's declared exports.
- Gameplay reads require explicit fields, selector capabilities, a positive
  `maxItems` quota, and a stable ordering statement.
- Prefab-part requirements use `{ prefab, role }`; display labels, hierarchy
  scans, and private registry access are not representable selectors.
- Gameplay binding requirements can name configuration fields and target scope;
  the committed pulse requirement reaches delivery through generated contracts,
  public builder compilation, and ProjectBundle activation/save/reload proof.
- Lists and requirements are sorted so equivalent manifests are byte-stable and
  validation reports are deterministic.
- Requirements formerly labelled as delivery from only engine-local smoke or
  provider tests are downgraded to their honest provider/selector level. The
  obsolete `asha-demo.conformance` self-proof requirement was removed rather
  than preserved as report ceremony.

Run `./harness/ci/check-consumer-needs.sh`. The committed machine-readable result
is `harness/consumer-needs/validation-report.json`.

`docs/public-capability-reachability.md` describes the next join: selected stable
requirements are connected to generated contracts, real providers, public
surfaces, selectors/fields, bootstrap adapters, and delivery evidence. Its
machine-readable report is separate so a needs declaration cannot manufacture
provider evidence merely by naming it.
