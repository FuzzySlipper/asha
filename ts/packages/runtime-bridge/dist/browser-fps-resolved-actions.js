export class BrowserFpsResolvedActionConsumer {
    #held = new Set();
    #lookX = 0;
    #lookY = 0;
    #primaryFirePressed = false;
    accept(action) {
        if (action.value.kind === 'button') {
            if (action.phase === 'released' || !action.value.pressed)
                this.#held.delete(action.actionId);
            else
                this.#held.add(action.actionId);
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
    drain() {
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
    reset() {
        this.#held.clear();
        this.#lookX = 0;
        this.#lookY = 0;
        this.#primaryFirePressed = false;
    }
}
function direction(positive, negative) {
    if (positive === negative)
        return 0;
    return positive ? 1 : -1;
}
//# sourceMappingURL=browser-fps-resolved-actions.js.map