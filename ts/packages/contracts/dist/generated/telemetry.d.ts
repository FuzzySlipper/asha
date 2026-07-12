export type TelemetrySource = 'runtime' | 'policy' | 'renderer' | 'devtools' | 'replay';
export type TelemetryLevel = 'debug' | 'info' | 'warning' | 'error';
export type TelemetryMetricKind = 'counter' | 'gauge' | 'durationMs';
export interface TelemetryMetric {
    readonly name: string;
    readonly kind: TelemetryMetricKind;
    readonly value: number;
    readonly unit: string | null;
}
export type TelemetryEvent = {
    readonly kind: 'metric';
    readonly source: TelemetrySource;
    readonly level: TelemetryLevel;
    readonly sequence: number;
    readonly metric: TelemetryMetric;
} | {
    readonly kind: 'trace';
    readonly source: TelemetrySource;
    readonly level: TelemetryLevel;
    readonly sequence: number;
    readonly span: string;
    readonly message: string;
};
export interface TelemetryEnvelope {
    readonly protocolVersion: number;
    readonly emittedAtTick: number;
    readonly events: readonly TelemetryEvent[];
}
export type LiveTelemetryCounter = 'frameTimeMs' | 'entityCount' | 'activeCapabilityCount' | 'residentChunkCount' | 'dirtyChunkCount' | 'renderDiffCount' | 'renderHandleCount' | 'drawCallCount' | 'activeAudioSourceCount' | 'activeBillboardCount' | 'activeParticleCount' | 'droppedFeedbackCount';
export interface LiveTelemetryMetric {
    readonly counter: LiveTelemetryCounter;
    readonly kind: TelemetryMetricKind;
    readonly value: number;
    readonly unit: string;
}
export type LiveTelemetryDiagnosticCode = 'counterUnavailable' | 'invalidSample';
export interface LiveTelemetryDiagnostic {
    readonly code: LiveTelemetryDiagnosticCode;
    readonly counter: LiveTelemetryCounter | null;
    readonly message: string;
}
export interface LiveTelemetrySnapshot {
    readonly schemaVersion: number;
    readonly authorityTick: number;
    readonly sampleSequence: number;
    readonly metrics: readonly LiveTelemetryMetric[];
    readonly frameTimeHistoryMs: readonly number[];
    readonly diagnostics: readonly LiveTelemetryDiagnostic[];
}
//# sourceMappingURL=telemetry.d.ts.map