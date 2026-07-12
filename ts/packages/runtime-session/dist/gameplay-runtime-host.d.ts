import type { GameplayEventEnvelope, GameplayModuleBindingRegistry, GameplayTriggerDefinition, PrefabTransform } from '@asha/contracts';
export interface GameplayRuntimePrefabBootstrap {
    readonly registryJson: string;
    readonly catalog: {
        readonly assetIds: readonly string[];
        readonly entityDefinitionIds: readonly string[];
    };
    readonly placements: readonly GameplayRuntimePrefabPlacement[];
}
export interface GameplayRuntimePrefabPlacement {
    readonly commandId: string;
    readonly origin: 'authored' | 'player';
    readonly instance: number;
    readonly prefab: number;
    readonly seed: number;
    readonly transform: PrefabTransform;
    readonly overrides: readonly ({
        readonly targetRole: string;
    } & ({
        readonly field: 'transform';
        readonly transform: PrefabTransform;
    } | {
        readonly field: 'entityDefinition';
        readonly stableId: string;
    } | {
        readonly field: 'asset';
        readonly asset: string;
    } | {
        readonly field: 'material';
        readonly asset: string;
    } | {
        readonly field: 'activation';
        readonly active: boolean;
    }))[];
}
export interface GameplayRuntimeHostLoadInput {
    readonly kind: 'gameplay_runtime_host.load.v1';
    readonly projectId: string;
    readonly compositionHash: string;
    readonly declaredReadPlanHash: string;
    readonly bindings: GameplayModuleBindingRegistry;
    readonly triggers: readonly GameplayTriggerDefinition[];
    readonly prefabs?: GameplayRuntimePrefabBootstrap;
}
export type GameplayRuntimeHostMoment = {
    readonly kind: 'tick';
    readonly tick: number;
} | {
    readonly kind: 'actorMovement';
    readonly tick: number;
    readonly actor: number;
    readonly delta: readonly [number, number, number];
} | {
    readonly kind: 'ownerEvent';
    readonly event: GameplayEventEnvelope;
} | {
    readonly kind: 'prefabInteraction';
    readonly tick: number;
    readonly instance: number;
    readonly role: string;
};
export interface GameplayRuntimeRoutingReadout {
    readonly proposalId: string;
    readonly proposalKind: string;
    readonly ownerId: string;
    readonly accepted: boolean;
    readonly proposalHash: string;
    readonly routingHash: string;
    readonly diagnosticCodes: readonly string[];
}
export interface GameplayRuntimeReactionFrameReadout {
    readonly frameHash: string;
    readonly registryDigest: string;
    readonly deliveredEvents: readonly GameplayEventEnvelope[];
    readonly frozenViewHashes: readonly string[];
    readonly invocationOutputHashes: readonly string[];
    readonly routing: readonly GameplayRuntimeRoutingReadout[];
    readonly acceptedModuleFactHashes: readonly string[];
    readonly stateHashBefore: string;
    readonly stateHashAfter: string;
    readonly finalSessionHash: string;
    readonly diagnosticCodes: readonly string[];
}
export interface GameplayRuntimeHostReadout {
    readonly kind: 'gameplay_runtime_host.readout.v1';
    readonly gameplayRegistryDigest: string;
    readonly bindingRegistryHash: string;
    readonly activationHash: string;
    readonly moduleStateHash: string;
    readonly triggerRevision: number;
    readonly triggerSnapshotHash: string;
    readonly activeOverlapCount: number;
    readonly reactionFrameCount: number;
    readonly lastReactionFrameHash: string | null;
    readonly recentFrames: readonly GameplayRuntimeReactionFrameReadout[];
    readonly runtimeHostHash: string;
    readonly prefabs?: GameplayRuntimePrefabReadout;
    readonly moduleStates?: readonly GameplayRuntimeModuleStateReadout[];
}
export interface GameplayRuntimePrefabReadout {
    readonly stateHash: string;
    readonly acceptedCommands: readonly {
        readonly commandId: string;
        readonly instance: number;
        readonly prefab: number;
        readonly origin: 'authored' | 'player';
    }[];
    readonly instances: readonly GameplayRuntimePrefabInstanceReadout[];
}
export interface GameplayRuntimePrefabInstanceReadout {
    readonly instance: number;
    readonly prefab: number;
    readonly origin: 'authored' | 'player';
    readonly provenanceHash: string;
    readonly overrideCount: number;
    readonly parts: readonly {
        readonly part: number;
        readonly namespace: string;
        readonly entity: number;
        readonly parentEntity: number | null;
        readonly translation: readonly [number, number, number];
        readonly sourceKind: 'scene' | 'entityDefinition' | 'voxelObject';
        readonly active: boolean;
        readonly roles: readonly string[];
    }[];
    readonly roles: readonly {
        readonly role: string;
        readonly entity: number;
    }[];
}
export interface GameplayRuntimeModuleStateReadout {
    readonly moduleId: string;
    readonly stateContract: string;
    readonly scope: {
        readonly kind: 'session';
    } | {
        readonly kind: 'entity';
        readonly entity: number;
    } | {
        readonly kind: 'prefabInstance';
        readonly instance: number;
    };
    readonly revision: number;
    readonly stateHash: string;
    readonly initializedFrom: string;
}
export interface GameplayRuntimeHostLoadReceipt {
    readonly kind: 'gameplay_runtime_host.load_receipt.v1';
    readonly accepted: boolean;
    readonly diagnostics: readonly string[];
    readonly readout: GameplayRuntimeHostReadout | null;
}
export interface GameplayRuntimeHostAdvanceReceipt {
    readonly kind: 'gameplay_runtime_host.advance_receipt.v1';
    readonly accepted: boolean;
    readonly diagnostics: readonly string[];
    readonly moment: GameplayRuntimeHostMoment;
    readonly frames: readonly GameplayRuntimeReactionFrameReadout[];
    readonly readout: GameplayRuntimeHostReadout;
}
export interface GameplayRuntimeHostSnapshot {
    readonly kind: 'gameplay_runtime_host.snapshot.v1';
    readonly canonicalText: string;
    readonly snapshotHash: string;
}
/**
 * Consumer-owned native host port. A downstream provider statically links its
 * Rust modules and implements this closed transport; TypeScript never supplies
 * callbacks or an authority mutation function.
 */
export interface GameplayRuntimeHostTransport {
    load(input: GameplayRuntimeHostLoadInput): GameplayRuntimeHostLoadReceipt;
    advance(moment: GameplayRuntimeHostMoment): GameplayRuntimeHostAdvanceReceipt;
    read(): GameplayRuntimeHostReadout;
    save(): GameplayRuntimeHostSnapshot;
    restore(input: GameplayRuntimeHostLoadInput, snapshot: GameplayRuntimeHostSnapshot): GameplayRuntimeHostLoadReceipt;
}
//# sourceMappingURL=gameplay-runtime-host.d.ts.map