import type { VoxelCommand, VoxelCoord } from '@asha/contracts';
import { EditorStore } from '@asha/editor-tools';
export { EditorStore } from '@asha/editor-tools';
export type { EditorContext, EditorAction, VoxelSelection } from '@asha/editor-tools';
/**
 * Where committed commands go. In real wiring this is backed by
 * `@asha/runtime-bridge` (`submitCommands`), which sends the batch to Rust for
 * validation + application. Injected so the editor controller stays decoupled from
 * the transport and is trivially testable.
 */
export type CommandSink = (commands: readonly VoxelCommand[]) => void;
/**
 * The single authority-safe edit path. Holds the persistent {@link EditorStore},
 * computes a non-authoritative preview, and — only on {@link commit} — submits the
 * proposed command through the injected {@link CommandSink}. It never mutates voxel
 * state itself.
 */
export declare class VoxelEditController {
    #private;
    readonly store: EditorStore;
    constructor(sink: CommandSink, store?: EditorStore);
    /** The cells the current brush would affect — non-authoritative preview data. */
    preview(): VoxelCoord[];
    /** The command the current context would commit, without submitting it. */
    proposal(): VoxelCommand | null;
    /**
     * Submit the current proposal through the bridge path (the only mutation route).
     * Returns the submitted command, or `null` if there was nothing to commit (no
     * selection / non-editing tool) — in which case the sink is not called.
     */
    commit(): VoxelCommand | null;
}
//# sourceMappingURL=index.d.ts.map