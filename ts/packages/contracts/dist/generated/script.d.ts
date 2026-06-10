import type { EntityId, SubjectId, ProcessId, ModeId, SignalId, TagId } from './ids.js';
export interface EntityView {
    readonly id: EntityId;
    readonly tags: readonly TagId[];
}
export interface ProcessView {
    readonly id: ProcessId;
    readonly mode: ModeId | null;
}
export interface ScriptView {
    readonly entities: readonly EntityView[];
    readonly subjects: readonly SubjectId[];
    readonly processes: readonly ProcessView[];
    readonly modes: readonly ModeId[];
    readonly signals: readonly SignalId[];
    readonly tags: readonly TagId[];
}
export type EntityCommand = {
    readonly kind: 'create';
    readonly id: EntityId;
} | {
    readonly kind: 'addTag';
    readonly id: EntityId;
    readonly tag: TagId;
} | {
    readonly kind: 'removeTag';
    readonly id: EntityId;
    readonly tag: TagId;
} | {
    readonly kind: 'delete';
    readonly id: EntityId;
};
export type SubjectCommand = {
    readonly kind: 'create';
    readonly id: SubjectId;
} | {
    readonly kind: 'delete';
    readonly id: SubjectId;
};
export type ProcessCommand = {
    readonly kind: 'start';
    readonly id: ProcessId;
} | {
    readonly kind: 'setMode';
    readonly id: ProcessId;
    readonly mode: ModeId;
} | {
    readonly kind: 'stop';
    readonly id: ProcessId;
};
export type ModeCommand = {
    readonly kind: 'define';
    readonly id: ModeId;
} | {
    readonly kind: 'undefine';
    readonly id: ModeId;
};
export type SignalCommand = {
    readonly kind: 'define';
    readonly id: SignalId;
} | {
    readonly kind: 'undefine';
    readonly id: SignalId;
};
export type TagCommand = {
    readonly kind: 'define';
    readonly id: TagId;
} | {
    readonly kind: 'undefine';
    readonly id: TagId;
};
export type Command = {
    readonly domain: 'entity';
    readonly command: EntityCommand;
} | {
    readonly domain: 'subject';
    readonly command: SubjectCommand;
} | {
    readonly domain: 'process';
    readonly command: ProcessCommand;
} | {
    readonly domain: 'mode';
    readonly command: ModeCommand;
} | {
    readonly domain: 'signal';
    readonly command: SignalCommand;
} | {
    readonly domain: 'tag';
    readonly command: TagCommand;
};
export type CommandKind = 'input' | 'policy' | 'system';
export interface CommandEnvelope {
    readonly kind: CommandKind;
    readonly command: Command;
}
export type ScriptRejection = {
    readonly reason: 'entityAlreadyExists';
    readonly id: EntityId;
} | {
    readonly reason: 'entityNotFound';
    readonly id: EntityId;
} | {
    readonly reason: 'tagNotFound';
    readonly id: TagId;
} | {
    readonly reason: 'tagAlreadyOnEntity';
    readonly id: EntityId;
    readonly tag: TagId;
} | {
    readonly reason: 'tagNotOnEntity';
    readonly id: EntityId;
    readonly tag: TagId;
} | {
    readonly reason: 'subjectAlreadyExists';
    readonly id: SubjectId;
} | {
    readonly reason: 'subjectNotFound';
    readonly id: SubjectId;
} | {
    readonly reason: 'processAlreadyExists';
    readonly id: ProcessId;
} | {
    readonly reason: 'processNotFound';
    readonly id: ProcessId;
} | {
    readonly reason: 'modeAlreadyExists';
    readonly id: ModeId;
} | {
    readonly reason: 'modeNotFound';
    readonly id: ModeId;
} | {
    readonly reason: 'signalAlreadyExists';
    readonly id: SignalId;
} | {
    readonly reason: 'signalNotFound';
    readonly id: SignalId;
} | {
    readonly reason: 'tagAlreadyDefined';
    readonly id: TagId;
} | {
    readonly reason: 'tagDefinitionNotFound';
    readonly id: TagId;
};
export type ScriptOutcome = {
    readonly status: 'accepted';
} | {
    readonly status: 'rejected';
    readonly rejection: ScriptRejection;
};
//# sourceMappingURL=script.d.ts.map