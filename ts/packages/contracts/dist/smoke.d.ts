import { type EntityId, type CommandEnvelope, type ScriptView, type ReplayRecord } from './index.js';
export declare const __contractSmoke: {
    readonly entity: EntityId;
    readonly addTag: {
        readonly domain: "entity";
        readonly command: import("./index.js").EntityCommand;
    };
    readonly envelope: CommandEnvelope;
    readonly view: ScriptView;
    readonly outcome: {
        readonly status: "accepted";
    };
    readonly createDiff: {
        readonly op: "create";
        readonly handle: import("./index.js").RenderHandle;
        readonly parent: import("./index.js").RenderHandle | null;
        readonly node: import("./index.js").RenderNode;
    };
    readonly diff: {
        readonly op: "destroy";
        readonly handle: import("./index.js").RenderHandle;
    };
    readonly record: ReplayRecord;
};
//# sourceMappingURL=smoke.d.ts.map