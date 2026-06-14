export interface VoxelCoord {
    readonly x: number;
    readonly y: number;
    readonly z: number;
}
export interface ChunkCoord {
    readonly x: number;
    readonly y: number;
    readonly z: number;
}
export type VoxelValue = {
    readonly kind: 'empty';
} | {
    readonly kind: 'solid';
    readonly material: number;
};
export type VoxelCommand = {
    readonly op: 'setVoxel';
    readonly grid: number;
    readonly coord: VoxelCoord;
    readonly value: VoxelValue;
} | {
    readonly op: 'fillRegion';
    readonly grid: number;
    readonly min: VoxelCoord;
    readonly max: VoxelCoord;
    readonly value: VoxelValue;
} | {
    readonly op: 'generateChunk';
    readonly grid: number;
    readonly chunk: ChunkCoord;
    readonly seed: number;
    readonly generatorVersion: number;
};
export type VoxelEditEvent = {
    readonly event: 'voxelSet';
    readonly grid: number;
    readonly coord: VoxelCoord;
    readonly value: VoxelValue;
} | {
    readonly event: 'voxelRegionFilled';
    readonly grid: number;
    readonly min: VoxelCoord;
    readonly max: VoxelCoord;
    readonly value: VoxelValue;
} | {
    readonly event: 'chunkGenerated';
    readonly grid: number;
    readonly chunk: ChunkCoord;
    readonly seed: number;
    readonly generatorVersion: number;
    readonly hash: number;
};
export type VoxelEditRejection = {
    readonly reason: 'unknownMaterial';
    readonly material: number;
} | {
    readonly reason: 'emptyRegion';
    readonly min: VoxelCoord;
    readonly max: VoxelCoord;
} | {
    readonly reason: 'chunkNotResident';
    readonly chunk: ChunkCoord;
} | {
    readonly reason: 'generationDivergence';
    readonly chunk: ChunkCoord;
    readonly expected: number;
    readonly actual: number;
};
export interface CommandBatch {
    readonly commands: readonly VoxelCommand[];
}
export interface CommandResult {
    readonly accepted: number;
    readonly rejected: number;
    readonly rejections: readonly VoxelEditRejection[];
}
export type Face = 'posX' | 'negX' | 'posY' | 'negY' | 'posZ' | 'negZ';
export type PickRejection = {
    readonly reason: 'noHit';
} | {
    readonly reason: 'hitMismatch';
    readonly authoritativeVoxel: VoxelCoord;
    readonly authoritativeFace: Face;
    readonly claimedVoxel: VoxelCoord;
    readonly claimedFace: Face;
};
export interface PickRay {
    readonly grid: number;
    readonly origin: readonly [number, number, number];
    readonly direction: readonly [number, number, number];
    readonly maxDistance: number;
}
export interface VoxelHit {
    readonly grid: number;
    readonly voxel: VoxelCoord;
    readonly chunk: ChunkCoord;
    readonly face: Face;
    readonly point: readonly [number, number, number];
    readonly distance: number;
}
export type PickResult = {
    readonly outcome: 'hit';
    readonly hit: VoxelHit;
} | {
    readonly outcome: 'miss';
    readonly rejection: PickRejection;
};
//# sourceMappingURL=voxel.d.ts.map