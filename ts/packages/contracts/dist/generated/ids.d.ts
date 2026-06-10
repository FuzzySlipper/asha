export type EntityId = number & {
    readonly __brand: 'EntityId';
};
export declare const entityId: (raw: number) => EntityId;
export type SubjectId = number & {
    readonly __brand: 'SubjectId';
};
export declare const subjectId: (raw: number) => SubjectId;
export type ProcessId = number & {
    readonly __brand: 'ProcessId';
};
export declare const processId: (raw: number) => ProcessId;
export type ModeId = number & {
    readonly __brand: 'ModeId';
};
export declare const modeId: (raw: number) => ModeId;
export type SignalId = number & {
    readonly __brand: 'SignalId';
};
export declare const signalId: (raw: number) => SignalId;
export type TagId = number & {
    readonly __brand: 'TagId';
};
export declare const tagId: (raw: number) => TagId;
//# sourceMappingURL=ids.d.ts.map