# Gameplay module configuration and ProjectBundle bindings

Gameplay configuration is durable authored input to a statically linked Rust
module. It is not live gameplay state, a callback registration, or a mutable
registry. At Session construction, Rust validates the complete binding registry
against the already-closed gameplay fabric, resolves stable targets, and then
initializes the existing `GameplayModuleStateStore` in one atomic batch.

## Stored contract

`GameplayModuleBindingRegistry` is generated from Rust into
`@asha/contracts/gameExtension`. It contains:

- hash-bound canonical configuration bytes plus module/provider/version,
  configuration-schema, and codec identity;
- bindings to a module-owned state schema, declared read contracts, declared
  output contracts, and an activation target;
- optional prefab-instance layers that replace configuration or enablement; and
- a content hash over the complete sorted registry.

Targets are `Session`, stored `EntityDefinition` stable identity, prefab stable
identity, or `PrefabPartReference { prefab, role }`. The last form is deliberately
separate from `PrefabDefinition`; no display name, hierarchy path, runtime entity
id, or private prefab-registry lookup becomes durable identity.

Downstream Rust authoring uses the public `asha-gameplay-module-sdk` exports
`GameplayModuleBindingRegistryBuilder`, `GameplayModuleConfiguration`, and
`GameplayModuleBindingTarget`. TypeScript receives the corresponding generated
projection contracts but cannot register executable behavior.

## Construction and authority

`GameplayBoundProjectBundleSession::activate` consumes a successful
`ProjectBundleLoadResult`, a `GameplayStaticComposition`, the binding registry,
and an explicit EntityDefinition-to-entity index. Before module state exists it
checks:

- exact compiled module, provider, version, SDK, contract, artifact, and source
  evidence through the closed registry;
- configuration schema and codec identity;
- state ownership and every requested read/output contract;
- stable target resolution and active target eligibility; and
- unique `(state schema, Session/entity/prefab-instance scope)` ownership.

Only after all target and contract checks succeed does the state store decode
configuration and initialize every facet atomically. Thereafter canonical module
state is live authority; changing the authored configuration does not mutate a
running Session.

Prefab bindings fan out over matching instantiated prefabs. A whole-prefab
binding owns a stable prefab-instance state scope. A stable-part binding resolves
the role through `PrefabInstanceAuthority` and owns the resolved entity facet.
Per-instance layers are applied only after proving that the instance belongs to
the binding's prefab.

## Receipts, save, reload, and migration

Activation returns typed generated readouts and a receipt carrying the authored
binding hash, closed gameplay-registry digest, resolved scopes, effective
configuration ids, activation status, provenance hashes, and initialized module
state hash.

`compose_gameplay_session_snapshot` writes the durable
`session/gameplay-modules.snapshot.json` artifact. It binds the gameplay state
snapshot to the normal ProjectBundle Session artifact, including prefab
role/override metadata. Reload re-resolves the current authored bindings, checks
the activation evidence and compiled registry, restores through registered typed
state adapters, and rejects authority or snapshot drift before returning a live
Session. State-schema upgrades continue through the existing explicit
`GameplayModuleStateMigration` path; no old provider is installed into the live
registry.

Proofs live in `rule-project-bundle/tests/gameplay_bindings.rs`, the generated
contract round-trip tests, and the downstream public-facade fixture. The
`gameplayBindingSchema` consumer need now reaches the delivery proof level.

See [Gameplay fabric growth recipes](gameplay-fabric-growth-recipes.md) for the
downstream authoring sequence and the distinction between module-local state
and shared owner-routed mutations.
