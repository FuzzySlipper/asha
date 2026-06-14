import type { CommandEnvelope, RenderFrameDiff } from '@asha/contracts';
import type { CommandBatch, WorldLoadRequest } from '@asha/runtime-bridge';
/** The abstract fixture world the smoke harness loads through the facade. */
export declare const FIXTURE_WORLD: WorldLoadRequest;
/** A deterministic FNV-1a hash over the fixture world definition (stable evidence). */
export declare function fixtureWorldHash(request: WorldLoadRequest): string;
/**
 * Deterministic, contract-shaped command envelopes (generated `@asha/contracts`
 * types). The smoke edit stage submits these instead of an ad-hoc `{ kind:
 * 'smoke-edit' }` literal, so the edit it proposes is a real authority command.
 */
export declare function fixtureCommandEnvelopes(): readonly CommandEnvelope[];
/**
 * The fixture edit batch for the facade. The facade's `submitCommands` still takes
 * the prototype `{ kind }` proposed-command shape (the generated command contract
 * is not wired into the bridge yet — tracked with the runtime-bridge DTO debt), so
 * each command's stable `kind` is *derived from* a generated CommandEnvelope rather
 * than hand-written, keeping the edit honest and drift-visible.
 */
export declare function fixtureCommandBatch(): CommandBatch;
/**
 * A deterministic fixture render frame: create one mesh node, then upload the quad
 * payload. Drives the renderer through its real create→replaceMeshPayload path.
 */
export declare function fixtureRenderFrame(): RenderFrameDiff;
//# sourceMappingURL=fixtures.d.ts.map