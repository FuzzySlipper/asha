import type { RenderFrameDiff } from '@asha/contracts';
import type { CompositionStatus, ProjectBundleLoadRequest } from './bridge.js';
import type { EncounterDirectorState } from '@asha/runtime-session';
import type { RuntimeSessionHashRecord, RuntimeSessionHashValue, RuntimeSessionIdentity, RuntimeSessionLifecycleState, RuntimeSessionNonClaim } from '@asha/runtime-session';
export declare function referenceRuntimeSessionNonClaims(): readonly RuntimeSessionNonClaim[];
export declare function identityHashRecord(identity: RuntimeSessionIdentity): RuntimeSessionHashRecord;
export declare function encounterStateHashRecord(state: EncounterDirectorState): RuntimeSessionHashRecord;
export declare function lifecycleStateHashRecord(state: RuntimeSessionLifecycleState): RuntimeSessionHashRecord;
export declare function projectBundleHashRecord(projectBundle: ProjectBundleLoadRequest): RuntimeSessionHashRecord;
export declare function compositionHashRecord(composition: CompositionStatus): RuntimeSessionHashRecord;
export declare function renderFrameHashRecord(frame: RenderFrameDiff): RuntimeSessionHashRecord;
export declare function stableHash(value: RuntimeSessionHashValue | undefined): string;
//# sourceMappingURL=runtime-session-hash.d.ts.map