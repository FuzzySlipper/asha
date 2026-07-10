// Browser-safe package-root condition for @asha/runtime-bridge.
//
// Browser consumers still import `@asha/runtime-bridge`; package.json selects
// this entry under the `browser` condition so Vite/Webpack do not evaluate the
// native transport module or its Node-only dependency chain.
export { MANIFEST_OPERATIONS } from './generated/operations.js';
export { decodeRenderDiff, decodeRenderFrameDiff, RenderDecodeError, RenderDiffStream, FrameMemory, } from './render-decode.js';
export { RuntimeBridgeError, frameCursor } from './bridge.js';
export * from './browser-fps-input.js';
export * from './native-runtime-provider.js';
export * from './playable-encounter-tick.js';
export * from './playable-loop-state.js';
export { createRuntimeSessionFacade, } from './runtime-session-adapter.js';
//# sourceMappingURL=browser.js.map