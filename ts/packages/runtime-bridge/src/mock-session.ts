import { createMockRuntimeBridge } from './mock.js';
import {
  createRuntimeSessionFacade,
  type RuntimeSessionFacade,
} from './runtime-session.js';
import type { RuntimeBridge } from './bridge.js';

export interface MockRuntimeSessionOptions {
  readonly bridge?: RuntimeBridge;
}

export function createMockRuntimeSession(options: MockRuntimeSessionOptions = {}): RuntimeSessionFacade {
  return createRuntimeSessionFacade({ bridge: options.bridge ?? createMockRuntimeBridge(), mode: 'reference' });
}
