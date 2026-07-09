# Mesh Animation Fixture

This fixture is the stable runtime asset for the ASHA mesh-animation campaign.
It is generated from Kenney Animated Characters Retro 1.1 source files under
`/home/stash/mesh-resources`, but runtime/demo code must load the committed GLB
from this directory instead of loading stash paths directly.

Files:

- `kenney-retro-character-medium.glb`: skinned GLB with embedded `humanMaleA`
  skin texture and named clips `idle`, `run`, and `jump`.
- `kenney-retro-character-medium.manifest.json`: asset id, source hashes, clip
  list, bounds hint, content hash, and conversion notes.
- `LICENSE.Kenney-Animated-Characters-Retro.txt`: upstream CC0 license text.

Commands:

```bash
node harness/assets/mesh-animation/inspect-animation-fixture.js
node harness/assets/mesh-animation/prepare-kenney-retro-character.js
```

The prep command validates source hashes before conversion. The inspector is
dependency-free and fails closed if the committed fixture loses the `run` proof
clip, contains external URI dependencies, or drifts from its manifest hash.
