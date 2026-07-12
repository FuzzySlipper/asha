import { fnv1a64, matrixKey } from './mock-primitives.js';
function f32(value) {
    return Math.fround(value);
}
function multiplyMatrix4(a, b) {
    const out = new Array(16).fill(0);
    for (let col = 0; col < 4; col += 1) {
        for (let row = 0; row < 4; row += 1) {
            let sum = 0;
            for (let k = 0; k < 4; k += 1) {
                sum = f32(sum + f32((a[k * 4 + row] ?? 0) * (b[col * 4 + k] ?? 0)));
            }
            out[col * 4 + row] = sum;
        }
    }
    return out;
}
function viewMatrixFromSnapshot(snapshot) {
    const { right, up, forward } = snapshot.basis;
    const position = snapshot.pose.position;
    const dotRight = f32(f32(right[0] * position[0]) + f32(right[1] * position[1]) + f32(right[2] * position[2]));
    const dotUp = f32(f32(up[0] * position[0]) + f32(up[1] * position[1]) + f32(up[2] * position[2]));
    const dotForward = f32(f32(forward[0] * position[0]) + f32(forward[1] * position[1]) + f32(forward[2] * position[2]));
    return [
        right[0], up[0], -forward[0], 0,
        right[1], up[1], -forward[1], 0,
        right[2], up[2], -forward[2], 0,
        -dotRight, -dotUp, dotForward, 1,
    ];
}
function projectionMatrixFromSnapshot(snapshot, viewport) {
    const aspect = f32(viewport.width / viewport.height);
    const f = f32(1 / Math.tan(f32((snapshot.projection.fovYDegrees * Math.PI) / 360)));
    const near = snapshot.projection.near;
    const far = snapshot.projection.far;
    return [
        f32(f / aspect), 0, 0, 0,
        0, f, 0, 0,
        0, 0, f32((far + near) / (near - far)), -1,
        0, 0, f32((2 * far * near) / (near - far)), 0,
    ];
}
export function mockCameraProjectionSnapshot(snapshot, viewport = snapshot.viewport) {
    const viewMatrix = viewMatrixFromSnapshot(snapshot);
    const projectionMatrix = projectionMatrixFromSnapshot(snapshot, viewport);
    const viewProjectionMatrix = multiplyMatrix4(projectionMatrix, viewMatrix);
    const projectionHash = `fnv1a64:${fnv1a64(matrixKey([
        ...viewMatrix,
        ...projectionMatrix,
        ...viewProjectionMatrix,
    ]))}`;
    return {
        ...snapshot,
        viewport,
        viewMatrix,
        projectionMatrix,
        viewProjectionMatrix,
        projectionHash,
    };
}
//# sourceMappingURL=mock-camera-projection.js.map