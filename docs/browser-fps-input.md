# Browser FPS Input

Status: task #4404 upstream browser FPS input surface for `asha-demo`,
Studio, and renderer-host canvas wiring.

Public import path:

```ts
import {
  BrowserFpsInputCollector,
} from '@asha/runtime-bridge';
import { createMockRuntimeSession } from '@asha/runtime-bridge/reference';
```

Typed runtime command emitted per drain:

```ts
{
  kind: 'runtime.apply_first_person_camera_input',
  envelope: FirstPersonCameraInputEnvelope
}
```

The envelope is accepted by `RuntimeSessionFacade.applyFirstPersonCameraInput`.
Primary fire press/release is emitted as typed runtime action intent proposals:

```ts
{
  kind: 'runtime.propose_runtime_action_intent',
  envelope: RuntimeActionIntentEnvelope
}
```

The envelope is accepted by `RuntimeSessionFacade.submitRuntimeActionIntent`;
the reference RuntimeSession returns typed combat/fire/health readout evidence
for primary-fire press intents.
Surfaces that do not own a RuntimeSession camera handle, such as
`@asha/renderer-host`, use the same collector through `drainInputFrame()`:

```ts
{
  tick: number,
  input: {
    moveForward: number,
    moveRight: number,
    moveUp: number,
    yawDeltaDegrees: number,
    pitchDeltaDegrees: number,
    dtSeconds: number,
    moveSpeedUnitsPerSecond: number
  }
}
```

This runtime-neutral frame is the durable browser/standalone input lane.
Renderer hosts may adapt DOM events into it and apply the resulting camera pose
or forward it to a movement authority, but they should not keep separate WASD,
mouse-look, or primary-fire state machines.
The collector also emits typed shell intents:

- `{ kind: 'request_pointer_lock', reason: 'primary_button' | 'programmatic' }`
- `{ kind: 'release_pointer_lock', reason: 'escape_key' | 'programmatic' }`

Shell state is explicit:

- `active` accepts keyboard, pointer-lock, mouse-look, and primary-fire input.
- `disabled` emits zero movement/look and no pointer/fire intents.
- `paused` emits zero movement/look and no pointer/fire intents.

Input mapping:

- `KeyW` / `KeyS` map to `moveForward` `1` / `-1`.
- `KeyD` / `KeyA` map to `moveRight` `1` / `-1`.
- Mouse movement is accumulated only while pointer lock is active.
- `yawDeltaDegrees = movementX * mouseSensitivityDegreesPerPixel`.
- `pitchDeltaDegrees = -movementY * mouseSensitivityDegreesPerPixel`.
- `Escape` emits pointer-lock release intent and records `releaseRequestedByEscape`.

Non-claims:

- No gameplay movement, collision, or physics.
- No authority mutation from browser input.
- Primary fire is a typed proposal/readout path, not local browser authority.
- No gameplay movement, collision, or physics authority; Rust/runtime movement
  surfaces still validate the submitted camera/action envelopes.
