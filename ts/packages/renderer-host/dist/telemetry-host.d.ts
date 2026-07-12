import type { LiveTelemetryCounter, LiveTelemetrySnapshot, PresentationFrameDiff, TelemetryOverlayDescriptor, TelemetryOverlayDiagnostic, TelemetryOverlayHandle, TelemetryOverlayReadout } from '@asha/contracts';
type CountCounter = Exclude<LiveTelemetryCounter, 'frameTimeMs'>;
export interface AshaLiveTelemetryCollectorOptions {
    readonly expectedCounters: readonly CountCounter[];
    readonly maxFrameTimeSamples?: number;
}
export interface AshaLiveTelemetrySample {
    readonly authorityTick: number;
    readonly frameTimeMs: number;
    readonly counters: Readonly<Partial<Record<CountCounter, number | null | undefined>>>;
}
export declare class AshaLiveTelemetryCollector {
    #private;
    constructor(options: AshaLiveTelemetryCollectorOptions);
    sample(input: AshaLiveTelemetrySample): LiveTelemetrySnapshot;
    readSnapshot(): LiveTelemetrySnapshot;
    tryReadSnapshot(): LiveTelemetrySnapshot | null;
}
export interface AshaTelemetryOverlaySink {
    render(handle: TelemetryOverlayHandle, descriptor: TelemetryOverlayDescriptor, snapshot: LiveTelemetrySnapshot | null): void;
    destroy(handle: TelemetryOverlayHandle): void;
}
export interface AshaTelemetryOverlayHostOptions {
    readonly collector: AshaLiveTelemetryCollector;
    readonly sink: AshaTelemetryOverlaySink;
}
export interface AshaTelemetryOverlayFrameReceipt {
    readonly applied: number;
    readonly diagnostics: readonly TelemetryOverlayDiagnostic[];
    readonly readout: TelemetryOverlayReadout;
}
export declare class AshaTelemetryOverlayHost {
    #private;
    constructor(options: AshaTelemetryOverlayHostOptions);
    applyPresentation(frame: PresentationFrameDiff): AshaTelemetryOverlayFrameReceipt;
    sample(input: AshaLiveTelemetrySample, elapsedMs: number): LiveTelemetrySnapshot;
    setVisible(handle: TelemetryOverlayHandle, visible: boolean): boolean;
    toggleVisible(handle: TelemetryOverlayHandle): boolean | null;
    readout(): TelemetryOverlayReadout;
    cleanup(): void;
}
export {};
//# sourceMappingURL=telemetry-host.d.ts.map