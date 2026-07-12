# Browser Named Input

Status: superseded by #5642. The five-key FPS collector described by earlier
versions of this document has been removed.

Browser keyboard and mouse integration now uses `BrowserInputHost` from
`@asha/runtime-bridge`. The host attaches the raw DOM listeners, normalizes
events into generated `RawInputSample` values, and submits them through an
initialized public `RuntimeSessionFacade`.

FPS camera code consumes `gameplay.*` resolver output through
`BrowserFpsResolvedActionConsumer`. Editor camera/tool code consumes `editor.*`
resolver output through `EditorResolvedInputConsumer`. Neither consumer owns a
key-code union or binding table.

Orbit and top-down interaction uses `ResolvedCameraNavigationConsumer` and the
high-priority `cameraNavigation` context. Pointer delta, wheel, and pan keys are
resolved as named actions; while this context is active it consumes the lower
FPS bindings, and returning to first person restores gameplay resolution. See
[Camera Modes, Navigation, and Transitions](camera-modes.md).

Menu and dialog contexts are pushed and popped through
`RuntimeSessionFacade.applyInputContextCommand`. Their Rust-owned priority and
consumption rules prevent lower gameplay delivery while preserving their own UI
bindings. `BrowserInputHost.readout()` exposes normalized samples, active
contexts, resolution evidence, consumer identity, and rejection/consumption
reason.

The default `Escape` bindings resolve `runtime.time.pause` from gameplay and
`runtime.time.resume` from the menu context. `ResolvedPauseContextConsumer`
combines those resolved actions with the public input-context and time-control
commands. This is downstream composition, not UI-owned pause authority.

Accepted receipts expose platform-free `RecordedInputAction` values.
`RuntimeSessionFacade.replayResolvedInputAction` validates and delivers those
semantic records directly, so replay does not synthesize DOM events or key
codes. See the named-input document for hash, context, and exactly-once rules.

See [Named Input Actions and Session Contexts](named-input-actions.md) for the
complete contracts, ownership, replay posture, and current non-claims.
