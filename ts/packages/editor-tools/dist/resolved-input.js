/**
 * Editor-side expression adapter. It deliberately accepts only resolver output;
 * raw DOM codes and binding choices remain owned by the browser input host and
 * Session input catalog.
 */
export class EditorResolvedInputConsumer {
    #held = new Set();
    #lookX = 0;
    #lookY = 0;
    #primaryToolPressed = false;
    #cancelPressed = false;
    accept(action) {
        if (!action.actionId.startsWith('editor.'))
            return false;
        if (action.value.kind === 'button') {
            if (action.phase === 'released' || !action.value.pressed)
                this.#held.delete(action.actionId);
            else
                this.#held.add(action.actionId);
            if (action.phase === 'pressed' && action.value.pressed) {
                this.#primaryToolPressed ||= action.actionId === 'editor.tool.primary';
                this.#cancelPressed ||= action.actionId === 'editor.tool.cancel';
            }
            return true;
        }
        if (action.actionId === 'editor.camera.look' && action.value.kind === 'axis2d') {
            this.#lookX += action.value.x;
            this.#lookY += action.value.y;
            return true;
        }
        return false;
    }
    drain() {
        const frame = {
            cameraForward: direction(this.#held.has('editor.camera.forward'), this.#held.has('editor.camera.backward')),
            cameraRight: direction(this.#held.has('editor.camera.right'), this.#held.has('editor.camera.left')),
            lookDelta: [this.#lookX, this.#lookY],
            primaryToolPressed: this.#primaryToolPressed,
            cancelPressed: this.#cancelPressed,
        };
        this.#lookX = 0;
        this.#lookY = 0;
        this.#primaryToolPressed = false;
        this.#cancelPressed = false;
        return frame;
    }
    reset() {
        this.#held.clear();
        this.#lookX = 0;
        this.#lookY = 0;
        this.#primaryToolPressed = false;
        this.#cancelPressed = false;
    }
}
function direction(positive, negative) {
    if (positive === negative)
        return 0;
    return positive ? 1 : -1;
}
//# sourceMappingURL=resolved-input.js.map