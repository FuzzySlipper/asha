import type { WorldBundleManifest, SaveSummary, CompactionSummary } from '@asha/contracts';
import type { WorldLoadRequest, WorldSaveSummary } from './index.js';
type IfEqual<A, B, Yes, No> = (<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2 ? Yes : No;
type AssertExact<A, B> = IfEqual<A, B, A, never>;
export type _SchemaVersionMatches = AssertExact<WorldLoadRequest['bundleSchemaVersion'], WorldBundleManifest['bundleSchemaVersion']>;
export type _ProtocolVersionMatches = AssertExact<WorldLoadRequest['protocolVersion'], WorldBundleManifest['protocolVersion']>;
export type _CompactedEditsMatches = AssertExact<WorldSaveSummary['compactedEdits'], CompactionSummary['compactedEdits']>;
export type _RetainedEditsMatches = AssertExact<WorldSaveSummary['retainedEdits'], CompactionSummary['retainedEdits']>;
export type _CompactionSectionPresent = AssertExact<SaveSummary['compaction'], CompactionSummary>;
export {};
//# sourceMappingURL=world-dto-conformance.test.d.ts.map