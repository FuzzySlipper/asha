import type { NativeAddon } from '@asha/native-bridge';

type ComposedGameplayHandlers = Pick<
  NativeAddon,
  | 'readComposedRuntimeSession'
  | 'readGameplayModuleView'
  | 'readGameplayPrefabPartInteractionTarget'
  | 'applyGameplayPrefabPartInteraction'
>;

export function createNativeComposedGameplayHandlers(
  calls: string[],
  hashA: string,
  hashB: string,
  hashC: string,
): ComposedGameplayHandlers {
  return {
    readComposedRuntimeSession: (handle: number) => {
      void handle;
      calls.push('composedRead');
      return {
        schemaVersion: 1,
        entityAuthorityHash: hashA,
        gameplay: {
          gameplayRegistryDigest: hashA,
          semanticCompatibilityDigest: hashB,
          artifactProvenanceDigest: hashA,
          compositionLoadMode: 'compatible',
          compatibilityDiagnostics: [],
          bindingRegistryHash: hashA,
          activationHash: hashA,
          moduleStateHash: hashA,
          authorityStateHash: hashA,
          triggerRevision: 0,
          triggerSnapshotHash: hashA,
          activeOverlapCount: 0,
          reactionFrameCount: 1,
          lastReactionFrameHash: hashB,
          decisionReceiptCount: 1,
          pendingDecisionCount: 0,
          lastDecisionReceiptHash: hashC,
          schedulerStateHash: hashA,
          schedulerPendingActionCount: 0,
          schedulerOutstandingDispatchCount: 0,
          schedulerOutstandingEventDeliveryCount: 0,
          schedulerFactCount: 0,
          schedulerTruncated: false,
          runtimeHostHash: hashB,
        },
        fpsSessionEpoch: 1,
        fpsReplayHash: hashC,
        runtimeSessionHash: hashA,
      };
    },
    readGameplayModuleView: (
      handle,
      namespace,
      name,
      version,
      schemaHash,
      scopeKind,
      scopeValue,
      expectedRuntimeSessionHash,
    ) => {
      void handle;
      calls.push(`moduleView:${namespace}:${name}:${scopeKind}:${scopeValue ?? 'none'}`);
      return {
        view: { namespace, name, version, schemaHash },
        providerId: 'provider.fixture-pulse',
        scopeKind,
        scopeValue: scopeValue ?? null,
        revision: 1,
        canonicalPayload: Uint8Array.from([52]),
        viewHash: hashB,
        runtimeSessionHash: expectedRuntimeSessionHash,
      };
    },
    applyGameplayPrefabPartInteraction: (
      handle,
      actor,
      role,
      maxDistanceMillimeters,
      tick,
      expectedRuntimeSessionHash,
    ) => {
      void handle;
      void tick;
      calls.push(`prefabInteraction:${actor}:${role}:${maxDistanceMillimeters}`);
      return {
        actor,
        instance: 700,
        role,
        target: 777,
        distanceMillimeters: 850,
        eventHash: hashB,
        reactionFrameHash: hashC,
        runtimeSessionHash: expectedRuntimeSessionHash,
      };
    },
    readGameplayPrefabPartInteractionTarget: (
      handle,
      actor,
      role,
      maxDistanceMillimeters,
      expectedRuntimeSessionHash,
    ) => {
      void handle;
      calls.push(`prefabInteractionTarget:${actor}:${role}:${maxDistanceMillimeters}`);
      return {
        actor,
        role,
        eligible: true,
        instance: 700,
        target: 777,
        distanceMillimeters: 850,
        runtimeSessionHash: expectedRuntimeSessionHash,
      };
    },
  };
}
