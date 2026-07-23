---
status: current
audience: [consumer, engine, studio]
tags: [typescript, behavior, project-content, gameplay-fabric, state-machine]
supersedes: []
see-also: [../../project-content-authoring.md, ../authority/gameplay-module-sdk.md, ../authority/gameplay-fabric-growth-recipes.md]
---

# Authored behavior packages

Authored behavior packages let TypeScript express a small gameplay possibility
space while Rust remains authoritative over what the declarations mean and how
they execute. TypeScript produces immutable ProjectContent. It is never called
by the runtime and cannot register callbacks or mutate runtime state.

The first vocabulary supports Demo-scale switch and door behavior:

- a signal from a prefab-part interaction or a trigger entry;
- conditions over an owner-bound state machine;
- states that describe a relative translation and collision activation;
- a legal state transition; and
- one immediate transition with an optional bounded, scheduled follow-up.

## TypeScript authoring

The helpers in `@asha/game-workspace` create readable data and normalize it
deterministically before it reaches Rust:

```ts
import {
  authoredBehavior as behavior,
  createAshaAuthoredBehaviorDocument,
} from '@asha/game-workspace';

const doorEntity = behavior.sceneEntity('scene.main/door');
const door = behavior.stateMachine(
  'main-door',
  'scene.main/door',
  'closed',
  [behavior.state('closed'), behavior.state('open')],
  [
    behavior.transition('open-door', 'closed', 'open'),
    behavior.transition('close-door', 'open', 'closed'),
  ],
);

export const doorBehavior = createAshaAuthoredBehaviorDocument(
  'behavior.main-door',
  {
    packageId: 'demo.main-door',
    stateMachines: [door],
    behaviors: [
      behavior.behavior(
        'open-then-close',
        behavior.prefabPartInteracted(
          behavior.prefabPart('scene.main/switch', 'interaction/switch'),
        ),
        [behavior.whenState('main-door', 'closed')],
        [
          behavior.step('open', [
            behavior.transitionState('main-door', 'open-door'),
            behavior.setRelativeTranslation(doorEntity, [0, 3, 0]),
            behavior.setCapabilityActive(doorEntity, 'collision', false),
          ]),
          behavior.afterTicks('close', 'open', 120, [
            behavior.transitionState('main-door', 'close-door'),
            behavior.setRelativeTranslation(doorEntity, [0, 0, 0]),
            behavior.setCapabilityActive(doorEntity, 'collision', true),
          ]),
        ],
      ),
    ],
  },
  {
    sourceModule: '@demo/content',
    sourcePath: 'src/content/main-door.ts',
  },
);
```

The result is a `behaviorPackage` document. A project stores its canonical JSON
under the manifest's `behaviorPackages` source root and saves it through the
ordinary ProjectContent authoring/write path. The compiler freezes every nested
value, rejects executable or ambient objects, sorts declarations canonically,
and records SDK, vocabulary, source-module, source-path, and source-hash
provenance.

## Authority and execution

The package is a first-class ProjectContent kind. It does not become a synthetic
Gameplay Module, provider configuration, binding, callback, or Fabric proposal.

1. ProjectContent validation resolves the package against the complete
   Engine-owned project set.
2. Rust rejects unknown versions, missing owners or references, illegal
   transitions, cycles, executable values, and declarations over the published
   budgets.
3. Project admission lowers stable scene and prefab references into a private,
   canonical numeric plan. No downstream runtime IDs are stored in the authored
   source or exposed as an authoring requirement.
4. Accepted signals select the compiled behavior. Rust evaluates its closed
   predicates and applies an atomic group of typed state-machine, transform, and
   capability verbs through their existing authority owners.
5. A delayed follow-up becomes an ordinary Gameplay Action Scheduler action.
   Symbolic state, accepted facts, pending work, and the compiled program
   identity participate in snapshot, restore, and replay validation.
6. Accepted EntityStore changes use the ordinary RuntimeSession authority
   commit and render-projection path. There is no behavior-specific bridge or
   renderer call.

The direct authority verbs are also the narrow adapter seam for owners that
already participate in Gameplay Fabric. This keeps one implementation of each
mutation while preserving two intentional consumption levels: compact authored
data for ordinary gameplay and the explicit Guard/Transform/React fabric for
advanced cross-module integration.

The initial state uses the scene instance's authored transform as its base.
Every other state translation is relative to that base, so behavior does not
silently replace scene placement semantics.

## Runtime composition

The closed vocabulary is built into canonical project admission and the runtime
host. Consumers do not register a provider or maintain a parallel gameplay
configuration merely to use an authored package. The same project-authoring
surface that reports other ProjectContent diagnostics reports package schema,
vocabulary, budget, and reference failures before runtime activation.

Studio can therefore construct a package from the generated contract, navigate
valid scene and prefab references, and repair rejected content without
inventing runtime IDs or module wiring. A game that needs custom runtime code
can still statically compose Gameplay Modules and use Fabric beside these
packages; neither system impersonates the other.

## Why this is not another scripting runtime

The v1 package is intentionally not:

- a universal node graph or arbitrary expression language;
- runtime TypeScript, `eval`, a callback registry, or arbitrary JSON invoke;
- an arbitrary property/path mutation API;
- branching, looping, parallel execution, or an unbounded sequence;
- a second state store, scheduler, persistence format, or replay system; or
- a durable relationship model.

The switch-to-door link is compile-time package wiring, so v1 does not install
it in `rule-relationship`. A relationship that must be queried, changed, or
persisted independently belongs in that existing rule. Likewise, the existing
state-machine rule is already the bounded process owner; wrapping it in a
second process abstraction would add no authority.

Future expressive families should grow by adding closed generated contracts,
Rust validation and ownership, private compilation, and typed TypeScript
helpers. They should not grow through an escape hatch or force ordinary content
through synthetic module ceremony. The downstream `asha-rpg` language helped
motivate this shape but remains unchanged and is not an Engine dependency.
