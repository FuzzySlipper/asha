import { createMockRuntimeBridge } from './mock.js';
import { createRuntimeSessionFacade, } from './runtime-session.js';
export function createMockRuntimeSession(options = {}) {
    return createRuntimeSessionFacade({ bridge: options.bridge ?? createMockRuntimeBridge(), mode: 'reference' });
}
//# sourceMappingURL=mock-session.js.map