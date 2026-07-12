export declare const INPUT_BINDING_CATALOG_SCHEMA_VERSION = 1;
export declare const INPUT_CONTEXT_STATE_SCHEMA_VERSION = 1;
export declare const INPUT_ACTION_RECORD_SCHEMA_VERSION = 1;
export type InputActionId = string;
export type InputContextId = string;
export type InputBindingId = string;
export type InputValueKind = 'button' | 'axis1d' | 'axis2d';
export type InputActionPhase = 'pressed' | 'held' | 'released' | 'changed';
export type PlatformInputKind = 'keyboardKey' | 'mouseButton' | 'mouseDelta' | 'mouseWheel';
export type InputValue = {
    readonly kind: 'button';
    readonly pressed: boolean;
} | {
    readonly kind: 'axis1d';
    readonly value: number;
} | {
    readonly kind: 'axis2d';
    readonly x: number;
    readonly y: number;
};
export interface InputActionDefinition {
    readonly actionId: InputActionId;
    readonly valueKind: InputValueKind;
    readonly acceptedPhases: readonly InputActionPhase[];
}
export interface InputContextDefinition {
    readonly contextId: InputContextId;
    readonly priority: number;
    readonly consumesLowerPriority: boolean;
}
export interface InputBindingExtension {
    readonly schemaVersion: number;
    readonly requiredControls: readonly string[];
}
export interface InputBindingRecord {
    readonly bindingId: InputBindingId;
    readonly actionId: InputActionId;
    readonly contextId: InputContextId;
    readonly platformKind: PlatformInputKind;
    readonly control: string;
    readonly scale: number;
    readonly extension: InputBindingExtension | null;
}
export interface InputBindingCatalog {
    readonly schemaVersion: number;
    readonly actions: readonly InputActionDefinition[];
    readonly contexts: readonly InputContextDefinition[];
    readonly bindings: readonly InputBindingRecord[];
}
export interface InputSessionConfigureRequest {
    readonly catalog: InputBindingCatalog;
    readonly initialContexts: readonly InputContextId[];
}
export interface ActiveInputContext {
    readonly contextId: InputContextId;
    readonly stackOrder: number;
}
export interface InputContextStackState {
    readonly schemaVersion: number;
    readonly revision: number;
    readonly activeContexts: readonly ActiveInputContext[];
    readonly stateHash: string;
}
export type InputContextCommand = {
    readonly operation: 'push';
    readonly contextId: InputContextId;
} | {
    readonly operation: 'pop';
    readonly expectedContextId: InputContextId;
} | {
    readonly operation: 'replace';
    readonly contextIds: readonly InputContextId[];
};
export interface InputContextChangeReceipt {
    readonly accepted: boolean;
    readonly state: InputContextStackState;
    readonly diagnostics: readonly InputDiagnostic[];
}
export interface InputSessionSnapshot {
    readonly catalogHash: string;
    readonly contextState: InputContextStackState;
}
export interface RawInputSample {
    readonly sequence: number;
    readonly platformKind: PlatformInputKind;
    readonly control: string;
    readonly phase: InputActionPhase;
    readonly value: InputValue;
}
export interface ResolvedInputAction {
    readonly sequence: number;
    readonly actionId: InputActionId;
    readonly contextId: InputContextId;
    readonly bindingId: InputBindingId;
    readonly phase: InputActionPhase;
    readonly value: InputValue;
}
export interface RecordedInputAction {
    readonly schemaVersion: number;
    readonly action: ResolvedInputAction;
    readonly catalogHash: string;
    readonly contextHash: string;
    readonly recordHash: string;
}
export type InputDiagnosticCode = 'unsupportedCatalogSchema' | 'unsupportedContextSchema' | 'invalidIdentifier' | 'duplicateAction' | 'duplicateContext' | 'duplicateBinding' | 'invalidPriority' | 'unknownAction' | 'unknownContext' | 'conflictingBinding' | 'valueKindMismatch' | 'unsupportedBindingExtension' | 'duplicateActiveContext' | 'nonCanonicalStackOrder' | 'contextStackMismatch' | 'catalogHashMismatch' | 'contextHashMismatch' | 'nonFiniteInput' | 'unsupportedPhase' | 'unboundInput' | 'consumedByContext' | 'unsupportedReplaySchema' | 'replayRecordHashMismatch' | 'replayAlreadyDelivered';
export interface InputDiagnostic {
    readonly code: InputDiagnosticCode;
    readonly path: string;
    readonly message: string;
}
export interface InputResolutionReceipt {
    readonly sequence: number;
    readonly accepted: boolean;
    readonly consumed: boolean;
    readonly action: ResolvedInputAction | null;
    readonly diagnostics: readonly InputDiagnostic[];
    readonly catalogHash: string;
    readonly contextHash: string;
    readonly inputHash: string;
    readonly resolutionHash: string;
    readonly record: RecordedInputAction | null;
}
export interface InputActionReplayReceipt {
    readonly accepted: boolean;
    readonly action: ResolvedInputAction | null;
    readonly diagnostics: readonly InputDiagnostic[];
    readonly catalogHash: string;
    readonly contextHash: string;
    readonly recordHash: string;
    readonly replayHash: string;
}
//# sourceMappingURL=input.d.ts.map