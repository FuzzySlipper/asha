# Project content authoring

ASHA exposes one Rust-owned workspace-authoring boundary for durable project
content that is not a `SceneDocument` or environment materialization artifact.
The boundary is intentionally a closed document union, not a property-path API
or JSON value bus.

## Stored document kinds

`ProjectContentDocument` admits these reusable categories:

- `entityDefinition` — the existing typed entity definition and capabilities;
- `assetCatalog` — stored asset, material, version, and dependency records;
- `prefabRegistry` — prefab definitions, stable named variants, roles, and
  typed overrides;
- `gameplayConfiguration` — provider-selected typed configuration values,
  bindings, stable scene-instance overrides, and trigger definitions; and
- `presentationCatalog` — renderer-neutral resources and animation, audio,
  particle, or overlay cue records.

Scene nodes remain in the existing scene-document codec. Project-content
requests carry the actual validated scenes as reference context so prefab
placements, trigger volumes, and per-instance overrides are checked against
the stored hierarchy rather than a caller-created index.

## Provider-owned configuration

Gameplay providers export immutable `ProjectConfigurationSchema` descriptors
through `ProjectContentReferenceContext.configurationSchemas`. Studio may use
their labels, bounds, value kinds, and reference kinds to construct an editor,
but project documents cannot edit those descriptors. Stored gameplay content
contains only the selected schema id and typed field values.

Rust admits the provider schema only through the supported canonical typed
codec, verifies provider ownership, required fields, value types and bounds,
and resolves typed references against the project content set. Provider and
product-specific combat or weapon vocabulary does not enter the generic
document contract.

## Public workflow

The workspace-authoring facade exposes:

1. `decodeProjectContent`, which strictly decodes source text, rejects unknown
   fields, resolves cross-document references, and returns canonical files,
   content identities, and field metadata;
2. `encodeProjectContent`, which performs the same validation over typed
   documents before canonical encoding; and
3. `applyProjectContentAuthoring`, which applies one typed upsert or delete.

An authoring request is bound to the workspace id, workspace generation,
working revision, and current project-content set hash. A stale request cannot
invoke the edit or create a save candidate. An accepted edit increments the
Rust workspace revision and registers its returned set hash as the only hash a
trusted host may confirm as stored.

File selection and persistence are trusted-host responsibilities. The host
writes the returned `canonicalFiles` only after Rust acceptance, then calls the
ordinary workspace stored-confirmation operation with the accepted set hash.
Browser code never accepts an edit or promotes a file itself.

## Non-claims

This surface does not start a `RuntimeSession`, register runtime callbacks,
expose arbitrary mutation paths, or make presentation code authoritative. It
also does not materialize procedural environments; that remains a separate
recipe-to-artifact workflow. Runtime composition consumes these canonical
documents later through ProjectBundle loading.
