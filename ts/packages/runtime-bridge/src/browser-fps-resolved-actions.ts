import type { ResolvedInputAction } from '@asha/contracts';

export interface BrowserFpsResolvedFrame {
  readonly moveForward: number;
  readonly moveRight: number;
  readonly pitchDeltaPixels: number;
  readonly yawDeltaPixels: number;
  readonly primaryFirePressed: boolean;
}

export class BrowserFpsResolvedActionConsumer {
  readonly #held = new Set<string>();
  #lookX = 0;
  #lookY = 0;
  #primaryFirePressed = false;

  accept(action: ResolvedInputAction): void {
    if (action.value.kind === 'button') {
      if (action.phase === 'released' || !action.value.pressed) this.#held.delete(action.actionId);
      else this.#held.add(action.actionId);
      if (action.actionId === 'gameplay.primaryFire') {
        this.#primaryFirePressed = action.phase === 'pressed' && action.value.pressed;
      }
      return;
    }
    if (action.actionId === 'gameplay.look' && action.value.kind === 'axis2d') {
      this.#lookX += action.value.x;
      this.#lookY += action.value.y;
    }
  }

  drain(): BrowserFpsResolvedFrame {
    const frame = {
      moveForward: direction(this.#held.has('gameplay.move.forward'), this.#held.has('gameplay.move.backward')),
      moveRight: direction(this.#held.has('gameplay.move.right'), this.#held.has('gameplay.move.left')),
      pitchDeltaPixels: this.#lookY,
      yawDeltaPixels: this.#lookX,
      primaryFirePressed: this.#primaryFirePressed,
    };
    this.#lookX = 0;
    this.#lookY = 0;
    this.#primaryFirePressed = false;
    return frame;
  }

  reset(): void {
    this.#held.clear();
    this.#lookX = 0;
    this.#lookY = 0;
    this.#primaryFirePressed = false;
  }
}

function direction(positive: boolean, negative: boolean): number {
  if (positive === negative) return 0;
  return positive ? 1 : -1;
}

