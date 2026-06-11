import type { EntityId, TagId } from './ids.js';
export type RenderHandle = number & {
    readonly __brand: 'RenderHandle';
};
export declare const renderHandle: (raw: number) => RenderHandle;
export interface Transform {
    readonly translation: readonly [number, number, number];
    readonly rotation: readonly [number, number, number, number];
    readonly scale: readonly [number, number, number];
}
export type Geometry = {
    readonly shape: 'cube';
} | {
    readonly shape: 'sphere';
} | {
    readonly shape: 'quad';
} | {
    readonly shape: 'point';
} | {
    readonly shape: 'line';
    readonly a: readonly [number, number, number];
    readonly b: readonly [number, number, number];
};
export interface Material {
    readonly color: readonly [number, number, number, number];
    readonly wireframe: boolean;
}
export type RenderLayer = 'scene' | 'debug';
export interface RenderMetadata {
    readonly source: EntityId | null;
    readonly tags: readonly TagId[];
    readonly label: string | null;
}
export interface RenderNode {
    readonly geometry: Geometry;
    readonly material: Material;
    readonly transform: Transform;
    readonly visible: boolean;
    readonly layer: RenderLayer;
    readonly metadata: RenderMetadata;
}
export type MeshAttributeKind = 'f32';
export type MeshAttributeName = 'position' | 'normal' | 'uv' | 'color';
export interface MeshAttribute {
    readonly name: MeshAttributeName;
    readonly components: number;
    readonly kind: MeshAttributeKind;
}
export type MeshIndexWidth = 'u32';
export interface MeshBufferLayout {
    readonly vertexCount: number;
    readonly indexCount: number;
    readonly indexWidth: MeshIndexWidth;
    readonly attributes: readonly MeshAttribute[];
}
export interface MeshGroupDescriptor {
    readonly materialSlot: number;
    readonly start: number;
    readonly count: number;
}
export interface MeshBoundsDescriptor {
    readonly min: readonly [number, number, number];
    readonly max: readonly [number, number, number];
}
export type MeshPayloadSource = {
    readonly kind: 'inline';
    readonly positions: readonly number[];
    readonly normals: readonly number[];
    readonly indices: readonly number[];
} | {
    readonly kind: 'handle';
    readonly buffer: number;
    readonly positionsByteOffset: number;
    readonly normalsByteOffset: number;
    readonly indicesByteOffset: number;
};
export interface MeshPayloadDescriptor {
    readonly layout: MeshBufferLayout;
    readonly groups: readonly MeshGroupDescriptor[];
    readonly bounds: MeshBoundsDescriptor;
    readonly source: MeshPayloadSource;
}
export type RenderDiff = {
    readonly op: 'create';
    readonly handle: RenderHandle;
    readonly parent: RenderHandle | null;
    readonly node: RenderNode;
} | {
    readonly op: 'update';
    readonly handle: RenderHandle;
    readonly transform: Transform | null;
    readonly material: Material | null;
    readonly visible: boolean | null;
    readonly metadata: RenderMetadata | null;
} | {
    readonly op: 'destroy';
    readonly handle: RenderHandle;
} | {
    readonly op: 'replaceMeshPayload';
    readonly handle: RenderHandle;
    readonly payload: MeshPayloadDescriptor;
};
export interface RenderFrameDiff {
    readonly ops: readonly RenderDiff[];
}
//# sourceMappingURL=render.d.ts.map