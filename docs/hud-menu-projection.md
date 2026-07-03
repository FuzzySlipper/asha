# HUD Menu Projection

Status: task #4043 reusable UI projection surface.

Public import path:

```ts
import {
  buildHudProjection,
  hudControlToIntent,
  type HudProjection,
  type HudMenuIntent,
} from '@asha/ui-dom';
```

The HUD projection is a pure rusty-view-style model: data in, render-agnostic
control descriptors and typed intents out. It is suitable for Angular/Studio/demo
bindings because no state or authority is hidden in DOM components.

`buildHudProjection()` projects:

- player health: current, max, dead, ratio, accessible label
- status lines
- runtime non-claim text
- menu controls for resume, restart, options, and exit

`hudControlToIntent()` returns typed proposals only:

- `runtime.restart_session_intent`
- `ui.open_options_intent`
- `ui.exit_to_menu_intent`
- `ui.resume_intent`

The restart intent is a UI proposal. Runtime/session code still validates and
executes restarts through the runtime facade.

Non-claims:

- No gameplay authority.
- No restart execution.
- No options or exit implementation.
- No DOM framework requirement.
- No arbitrary JSON payloads.
