import type { EntityId, SubjectId, ProcessId, ModeId, SignalId, TagId } from './ids.js';
import type { CommandEnvelope } from './script.js';
export type StepIndex = number & {
    readonly __brand: 'StepIndex';
};
export declare const stepIndex: (raw: number) => StepIndex;
export type ReplayHash = number & {
    readonly __brand: 'ReplayHash';
};
export declare const replayHash: (raw: number) => ReplayHash;
export declare const REPLAY_FORMAT_VERSION = 1;
export type DomainEvent = {
    readonly event: 'entityCreated';
    readonly id: EntityId;
} | {
    readonly event: 'entityTagAdded';
    readonly id: EntityId;
    readonly tag: TagId;
} | {
    readonly event: 'entityTagRemoved';
    readonly id: EntityId;
    readonly tag: TagId;
} | {
    readonly event: 'entityDeleted';
    readonly id: EntityId;
} | {
    readonly event: 'subjectCreated';
    readonly id: SubjectId;
} | {
    readonly event: 'subjectDeleted';
    readonly id: SubjectId;
} | {
    readonly event: 'processStarted';
    readonly id: ProcessId;
} | {
    readonly event: 'processModeSet';
    readonly id: ProcessId;
    readonly mode: ModeId;
} | {
    readonly event: 'processStopped';
    readonly id: ProcessId;
} | {
    readonly event: 'modeDefined';
    readonly id: ModeId;
} | {
    readonly event: 'modeUndefined';
    readonly id: ModeId;
} | {
    readonly event: 'signalDefined';
    readonly id: SignalId;
} | {
    readonly event: 'signalUndefined';
    readonly id: SignalId;
} | {
    readonly event: 'tagDefined';
    readonly id: TagId;
} | {
    readonly event: 'tagUndefined';
    readonly id: TagId;
};
export type StepOutcome = {
    readonly status: 'accepted';
    readonly events: readonly DomainEvent[];
} | {
    readonly status: 'rejected';
    readonly summary: string;
};
export interface ReplayStep {
    readonly index: StepIndex;
    readonly command: CommandEnvelope;
    readonly outcome: StepOutcome;
    readonly postHash: ReplayHash;
}
export interface SnapshotMeta {
    readonly step: StepIndex;
    readonly hash: ReplayHash;
    readonly snapshotVersion: number;
}
export interface ReplayRecord {
    readonly formatVersion: number;
    readonly initialHash: ReplayHash;
    readonly steps: readonly ReplayStep[];
    readonly snapshots: readonly SnapshotMeta[];
}
//# sourceMappingURL=replay.d.ts.map