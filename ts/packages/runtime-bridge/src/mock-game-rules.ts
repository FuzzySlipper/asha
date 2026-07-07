import type {
  GameRuleCatalog,
  GameRuleDiagnostic,
  GameRuleModifierState,
  GameRuleResolutionReceipt,
  GameRuleTraceEntry,
} from '@asha/contracts';
import {
  type GameRuleCatalogValidationReceipt,
  type GameRuleEffectIntentRequest,
  type GameRuleRuntimeReadout,
} from './bridge.js';

function fnv1a64(text: string): string {
  let hash = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  const mask = 0xffffffffffffffffn;
  for (let i = 0; i < text.length; i += 1) {
    hash ^= BigInt(text.charCodeAt(i));
    hash = (hash * prime) & mask;
  }
  return hash.toString(16).padStart(16, '0');
}

function mockGameRuleDiagnostic(
  code: GameRuleDiagnostic['code'],
  path: string,
  message: string,
): GameRuleDiagnostic {
  return { code, severity: 'error', path, message };
}

function mockGameRuleCatalogDiagnostics(catalog: GameRuleCatalog): GameRuleDiagnostic[] {
  const diagnostics: GameRuleDiagnostic[] = [];
  if (catalog.catalog.catalogId.trim() === '') {
    diagnostics.push(mockGameRuleDiagnostic('unknownEffectOp', 'catalog.catalogId', 'catalog id is required'));
  }
  if (catalog.catalog.version.trim() === '' || catalog.catalog.contentHash.trim() === '') {
    diagnostics.push(mockGameRuleDiagnostic('unknownEffectOp', 'catalog', 'catalog version and content hash are required'));
  }
  const channels = new Set(catalog.valueChannels.map((channel) => channel.channelId));
  for (const [bundleIndex, bundle] of catalog.bundles.entries()) {
    const modifiers = new Set(bundle.modifiers.map((modifier) => modifier.modifierId));
    for (const [opIndex, op] of bundle.effectOps.entries()) {
      if ('channelId' in op && !channels.has(op.channelId)) {
        diagnostics.push(mockGameRuleDiagnostic(
          'undeclaredValueChannel',
          `bundles[${bundleIndex}].effectOps[${opIndex}].channelId`,
          'effect op references undeclared value channel',
        ));
      }
      if ('modifierId' in op && !modifiers.has(op.modifierId)) {
        diagnostics.push(mockGameRuleDiagnostic(
          'unknownModifier',
          `bundles[${bundleIndex}].effectOps[${opIndex}].modifierId`,
          'effect op references unknown modifier',
        ));
      }
    }
  }
  return diagnostics;
}

function mergeGameRuleModifiers(
  current: readonly GameRuleModifierState[],
  incoming: readonly GameRuleModifierState[],
): GameRuleModifierState[] {
  const next = [...current];
  for (const modifier of incoming) {
    const index = next.findIndex((candidate) =>
      candidate.modifierId === modifier.modifierId &&
      candidate.source === modifier.source &&
      candidate.target === modifier.target);
    if (index === -1) {
      next.push(modifier);
    } else {
      next[index] = modifier;
    }
  }
  return next;
}

export class MockGameRuleRuntime {
  #activeModifiers: GameRuleModifierState[] = [];
  #recentTrace: GameRuleTraceEntry[] = [];
  #recentReplayHashes: string[] = [];

  reset(): void {
    this.#activeModifiers = [];
    this.#recentTrace = [];
    this.#recentReplayHashes = [];
  }

  validateCatalog(catalog: GameRuleCatalog): GameRuleCatalogValidationReceipt {
    const diagnostics = mockGameRuleCatalogDiagnostics(catalog);
    const catalogHash = `fnv1a64:${fnv1a64(JSON.stringify(catalog))}`;
    const trace = [{
      step: 1,
      code: diagnostics.length === 0 ? 'catalog.accepted' : 'catalog.rejected',
      message: diagnostics.length === 0 ? 'reference catalog validation accepted' : 'reference catalog validation rejected',
      refs: [{ key: 'catalogHash', value: catalogHash }],
    }];
    const evidenceHash = `fnv1a64:${fnv1a64(`${catalogHash}|catalogValidation`)}`;
    this.#recentTrace = trace;
    this.#recentReplayHashes = [...this.#recentReplayHashes, evidenceHash];
    return {
      accepted: diagnostics.length === 0,
      catalogHash,
      diagnostics,
      trace,
      evidence: [{
        kind: 'catalogValidation',
        uri: `asha://game-rules/catalog-validation/${catalog.catalog.catalogId}`,
        contentHash: evidenceHash,
      }],
    };
  }

  submitEffectIntent(input: GameRuleEffectIntentRequest): GameRuleResolutionReceipt {
    const { catalog, request } = input;
    const diagnostics = mockGameRuleCatalogDiagnostics(catalog);
    if (request.catalog.catalogId !== catalog.catalog.catalogId) {
      diagnostics.push(
        mockGameRuleDiagnostic('unknownEffectOp', 'catalog.catalogId', 'request catalog does not match supplied catalog'),
      );
    }
    const bundle = catalog.bundles.find((candidate) => candidate.bundleId === request.bundleId);
    if (bundle === undefined) {
      diagnostics.push(mockGameRuleDiagnostic('unknownEffectOp', 'bundleId', 'requested effect bundle does not exist'));
    }
    const requestHash = `fnv1a64:${fnv1a64(JSON.stringify(request))}`;
    const pendingValueDeltas = bundle === undefined ? [] : bundle.effectOps.flatMap((op) => {
      if (op.kind === 'applyDelta') return [{ channelId: op.channelId, amount: op.amount }];
      if (op.kind === 'restore' || op.kind === 'grant') return [{ channelId: op.channelId, amount: op.amount }];
      if (op.kind === 'spend') return [{ channelId: op.channelId, amount: -op.amount }];
      return [];
    });
    const appliedModifiers = bundle === undefined ? [] : bundle.effectOps.flatMap((op) => {
      if (op.kind !== 'applyModifier' && op.kind !== 'schedulePeriodicEffect') return [];
      const modifier = bundle.modifiers.find((candidate) => candidate.modifierId === op.modifierId);
      if (modifier === undefined) return [];
      const duration = op.kind === 'schedulePeriodicEffect' ? op.duration : modifier.duration;
      const cadence = op.kind === 'schedulePeriodicEffect' ? op.cadence : modifier.tickCadence;
      return [{
        modifierId: modifier.modifierId,
        source: request.source,
        target: request.target,
        stacks: 1,
        appliedTick: request.tick,
        expiresTick: duration.kind === 'ticks' ? request.tick + duration.ticks : null,
        nextTick: cadence === null ? null : request.tick + cadence.periodTicks,
        sourceHash: modifier.sourceHash,
      }];
    });
    const trace = [{
      step: 1,
      code: diagnostics.length === 0 ? 'resolution.accepted' : 'resolution.rejected',
      message: diagnostics.length === 0 ? 'reference effect intent resolved' : 'reference effect intent rejected',
      refs: [{ key: 'requestHash', value: requestHash }],
    }];
    const replayHash = `fnv1a64:${fnv1a64(`${requestHash}|${JSON.stringify(pendingValueDeltas)}|${JSON.stringify(appliedModifiers)}`)}`;
    this.#recentTrace = trace;
    this.#recentReplayHashes = [...this.#recentReplayHashes, replayHash];
    if (diagnostics.length === 0) {
      this.#activeModifiers = mergeGameRuleModifiers(this.#activeModifiers, appliedModifiers);
    }
    return {
      accepted: diagnostics.length === 0,
      requestHash,
      pendingValueDeltas,
      appliedModifiers,
      diagnostics,
      trace,
      evidence: [{ kind: 'resolutionReceipt', uri: `asha://game-rules/receipt/${requestHash}`, contentHash: replayHash }],
      replayHash,
    };
  }

  readRuntimeReadout(): GameRuleRuntimeReadout {
    return {
      backend: 'reference_bridge',
      authoritySurface: 'runtime_session.game_rules.reference.v0',
      activeModifiers: this.#activeModifiers,
      recentTrace: this.#recentTrace,
      recentReplayHashes: this.#recentReplayHashes,
      latestReplayHash: this.#recentReplayHashes.at(-1) ?? null,
    };
  }
}
