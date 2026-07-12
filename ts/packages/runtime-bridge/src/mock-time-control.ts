import type {
  TimeControlCommand,
  TimeControlReceipt,
  TimeControlRejection,
  TimeControlState,
} from '@asha/contracts';
import type { StepResult } from './bridge.js';
import { RuntimeBridgeError } from './bridge.js';
import { fnv1a64 } from './mock-primitives.js';

const MAX_SPEED_MULTIPLIER = 16;
const MAX_EXACT_STEP_TICKS = 10_000;

export class MockTimeController {
  #initialized = false;
  #mode: TimeControlState['mode'] = 'running';
  #speedMultiplier = 1;
  #revision = 0;
  #authorityTick = 0;

  initialize(): void {
    this.#initialized = true;
    this.#mode = 'running';
    this.#speedMultiplier = 1;
    this.#revision = 0;
    this.#authorityTick = 0;
  }

  read(): TimeControlState {
    this.#requireInitialized('readTimeControlState');
    return this.#state();
  }

  apply(command: TimeControlCommand): TimeControlReceipt {
    this.#requireInitialized('applyTimeControlCommand');
    const before = this.#state();
    let rejection: TimeControlRejection | null = null;
    let exactTicksAdvanced = 0;
    if (command.operation === 'pause') {
      if (this.#mode === 'paused') rejection = 'alreadyPaused';
      else this.#mode = 'paused';
    } else if (command.operation === 'resume') {
      if (this.#mode === 'running') rejection = 'alreadyRunning';
      else this.#mode = 'running';
    } else if (command.operation === 'setSpeedMultiplier') {
      if (!Number.isInteger(command.multiplier)
        || command.multiplier < 1
        || command.multiplier > MAX_SPEED_MULTIPLIER) {
        rejection = 'invalidSpeedMultiplier';
      } else {
        this.#speedMultiplier = command.multiplier;
      }
    } else if (this.#mode !== 'paused') {
      rejection = 'notPausedForExactStep';
    } else if (!Number.isInteger(command.ticks)
      || command.ticks < 1
      || command.ticks > MAX_EXACT_STEP_TICKS) {
      rejection = 'invalidStepCount';
    } else {
      exactTicksAdvanced = command.ticks;
      this.#authorityTick += command.ticks;
    }
    if (rejection === null) this.#revision += 1;
    const after = this.#state();
    const accepted = rejection === null;
    return {
      accepted,
      before,
      after,
      exactTicksAdvanced,
      rejection,
      receiptHash: `fnv1a64:${fnv1a64([
        accepted,
        before.stateHash,
        after.stateHash,
        exactTicksAdvanced,
        rejection,
      ].join('|'))}`,
    };
  }

  step(tick: number): StepResult {
    this.#requireInitialized('stepSimulation');
    if (this.#mode === 'paused') return { tick: this.#authorityTick, diffCount: 0 };
    this.#authorityTick = tick;
    return { tick, diffCount: tick % 4 };
  }

  #state(): TimeControlState {
    const stateHash = `fnv1a64:${fnv1a64([
      1,
      this.#mode,
      this.#speedMultiplier,
      this.#revision,
      this.#authorityTick,
    ].join('|'))}`;
    return {
      schemaVersion: 1,
      mode: this.#mode,
      speedMultiplier: this.#speedMultiplier,
      revision: this.#revision,
      authorityTick: this.#authorityTick,
      stateHash,
    };
  }

  #requireInitialized(operation: string): void {
    if (!this.#initialized) {
      throw new RuntimeBridgeError('not_initialized', `${operation} before initializeEngine`);
    }
  }
}
