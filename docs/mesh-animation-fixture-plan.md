# Mesh Animation Fixture Plan

Status: task #5287 resource inventory and fixture selection for the mesh-animation campaign.

## Selected Source

Use the Kenney Animated Characters Retro resources from `/home/stash/mesh-resources`.
The top-level stash directory is equivalent to the nested
`kenney_animated-characters-retro` copy, and it has the shortest stable paths for
the initial import proof.

Chosen source files:

| Role | Source path | Size | SHA-256 |
|---|---|---:|---|
| Bind-pose/skinned model | `/home/stash/mesh-resources/Model/characterMedium.fbx` | 167212 bytes | `18835fef534eede635b081ee7fe647d01a885550a591d2e6bf071010906167d8` |
| Idle animation source | `/home/stash/mesh-resources/Animations/idle.fbx` | 608188 bytes | `c8a24e0294376ee5a195c56752a13310e1c0b5f8588a4db50e094120e3e4cc74` |
| Run animation source | `/home/stash/mesh-resources/Animations/run.fbx` | 572732 bytes | `e635461fc8dace85ec67a7f7941e949a7c3f108b51ae4d2da1557e6e01749df8` |
| Jump animation source | `/home/stash/mesh-resources/Animations/jump.fbx` | 562796 bytes | `b88429077a7a1af5d3f55f43cfd8ce0f7441b4f6f7bb15a8070d7ed15d275f74` |
| Initial skin texture | `/home/stash/mesh-resources/Skins/humanMaleA.png` | 35497 bytes | `1590e08cea37f5aecbacabb40a57c176e389e9a95d5b2a4de00086604ef23e1c` |
| License | `/home/stash/mesh-resources/License.txt` | 692 bytes | `eaa916e20df30c26f18a752290f93ab0e5d95c3dd1057e6887d11aa4acc0e74b` |

License note: the source package is Kenney Animated Characters Retro 1.1,
distributed under CC0. Credit to Kenney is welcome but not required by the
license text.

## Clip Inventory

The stash provides three separate FBX animation source files:

| Stable ASHA clip id | Source file | Intended proof use |
|---|---|---|
| `idle` | `Animations/idle.fbx` | Default or rest-loop comparison clip. |
| `run` | `Animations/run.fbx` | Primary non-default proof clip. |
| `jump` | `Animations/jump.fbx` | Secondary named-clip coverage once playback switching exists. |

The FBX files are binary Kaydara FBX 7400 files. A direct string scan confirms
animation stack/layer records in each animation file, but the current checkout
does not have a committed FBX or glTF inspection tool. Task #5290 must convert
and validate the runtime GLB fixture, then record the final exported clip names.
The steady-state ASHA ids should remain lowercase `idle`, `run`, and `jump`,
even if the generated GLB stores display names such as `Idle`, `Run`, or `Jump`.

## Runtime Fixture Target

Task #5290 should prepare a small committed or reproducibly generated GLB fixture
from the selected source files. Use these target locations unless #5290 discovers
a technical blocker:

```text
harness/fixtures/mesh-animation/kenney-retro-character-medium.glb
harness/fixtures/mesh-animation/kenney-retro-character-medium.manifest.json
harness/fixtures/mesh-animation/LICENSE.Kenney-Animated-Characters-Retro.txt
```

Recommended manifest fields:

- `assetId`: `mesh-animation/kenney-retro-character-medium`
- `sourcePackage`: `Kenney Animated Characters Retro 1.1`
- `sourceLicense`: `CC0`
- `sourceHashes`: the SHA-256 values in this document
- `runtimeFormat`: `glb`
- `clipIds`: `idle`, `run`, `jump`
- `primaryProofClipId`: `run`
- `scaleHint`: start with `1.0`; adjust only if the converted asset is not visible
  in the standard ASHA scene scale.
- `boundsHint`: record after GLB validation.
- `texture`: `humanMaleA.png` baked or embedded by the import step.

Do not make runtime demos load from `/home/stash`. The stash paths are source
material only. Browser/runtime proof surfaces should load the committed fixture
or a generated artifact produced by a checked-in import command.

## Conversion And Validation Plan

Preferred public runtime format is glTF/GLB. FBX should stay an import/source
format for this proof.

Task #5290 should add the narrowest practical conversion path, for example:

```text
harness/assets/mesh-animation/prepare-kenney-retro-character.{sh,js}
harness/assets/mesh-animation/inspect-animation-fixture.{sh,js}
```

The preparation command must:

- merge `characterMedium.fbx` with the `idle`, `run`, and `jump` animation FBXs;
- preserve or normalize stable clip ids;
- include or bake the selected skin texture;
- write the GLB and manifest target paths above;
- fail closed if any source hash differs from this document.

The inspection command must report:

- available clip ids/names and durations;
- mesh/skinned-mesh count;
- skeleton/bone count when available;
- approximate bounds;
- content hash for the generated GLB.

Validation acceptance for #5290:

- `run` is present as a named clip and is not the implicit/default autoplay clip;
- `idle` is present so later proofs can distinguish commanded clip selection;
- the generated fixture can be loaded without absolute `/home/stash` paths;
- the license text is preserved beside the fixture or referenced by manifest.

## Downstream Use

Later #5286 child tasks should use this chosen asset plan as their fixture target:

- #5288 defines typed render/protocol vocabulary around stable asset ids and clip
  ids, not arbitrary URLs.
- #5289 teaches the Three.js backend to load the GLB and play a command-selected
  named clip.
- #5291 bridges Rust-owned render/runtime state into projection-only animation
  intent.
- #5292 exposes diagnostic readback proving the commanded clip and frame progress.
- #5293/#5294 use the committed GLB fixture for human-facing proof.

Animation mixer playback remains projection-only. It must not determine health,
hits, lifecycle, collision authority, replay outcomes, or any other gameplay
state.
