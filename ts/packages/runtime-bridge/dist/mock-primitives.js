import { RuntimeBridgeError } from './bridge.js';
export function matrixKey(values) {
    return values.map((value) => value.toFixed(3)).join(',');
}
export function fnv1a64(text) {
    let hash = 0xcbf29ce484222325n;
    for (let index = 0; index < text.length; index += 1) {
        hash ^= BigInt(text.charCodeAt(index));
        hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
    }
    return hash.toString(16).padStart(16, '0');
}
export function validateVec3(value, field) {
    if (value.length !== 3 || value.some((component) => !Number.isFinite(component))) {
        throw new RuntimeBridgeError('invalid_input', `${field} must be a finite vec3`);
    }
}
//# sourceMappingURL=mock-primitives.js.map