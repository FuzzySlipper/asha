export const frameCursor = (frame) => frame;
/** Typed, classified error for every facade operation. No JSON error blobs. */
export class RuntimeBridgeError extends Error {
    kind;
    constructor(kind, message) {
        super(`runtime bridge error [${kind}]: ${message}`);
        this.kind = kind;
        this.name = 'RuntimeBridgeError';
    }
}
export function nonNegativeSafeInteger(value, field) {
    if (!Number.isSafeInteger(value) || value < 0) {
        throw new RuntimeBridgeError('invalid_input', `${field} must be a non-negative safe integer`);
    }
    return value;
}
export function u32(value, field) {
    nonNegativeSafeInteger(value, field);
    if (value > 0xffffffff) {
        throw new RuntimeBridgeError('invalid_input', `${field} must fit in u32`);
    }
    return value;
}
//# sourceMappingURL=bridge.js.map