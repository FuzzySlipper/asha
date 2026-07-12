import type { ResolvedInputAction } from '@asha/contracts';
export interface EditorResolvedInputFrame {
    readonly cameraForward: number;
    readonly cameraRight: number;
    readonly lookDelta: readonly [number, number];
    readonly primaryToolPressed: boolean;
    readonly cancelPressed: boolean;
}
/**
 * Editor-side expression adapter. It deliberately accepts only resolver output;
 * raw DOM codes and binding choices remain owned by the browser input host and
 * Session input catalog.
 */
export declare class EditorResolvedInputConsumer {
    #private;
    accept(action: ResolvedInputAction): boolean;
    drain(): EditorResolvedInputFrame;
    reset(): void;
}
//# sourceMappingURL=resolved-input.d.ts.map