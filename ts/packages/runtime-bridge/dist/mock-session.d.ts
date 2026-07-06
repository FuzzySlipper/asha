import { type RuntimeSessionFacade } from './runtime-session.js';
import type { RuntimeBridge } from './bridge.js';
export interface MockRuntimeSessionOptions {
    readonly bridge?: RuntimeBridge;
}
export declare function createMockRuntimeSession(options?: MockRuntimeSessionOptions): RuntimeSessionFacade;
//# sourceMappingURL=mock-session.d.ts.map