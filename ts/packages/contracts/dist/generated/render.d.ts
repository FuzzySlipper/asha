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
};
export interface RenderFrameDiff {
    readonly ops: readonly RenderDiff[];
}
//# sourceMappingURL=render.d.ts.map