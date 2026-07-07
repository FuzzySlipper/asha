import type { GameRuleCatalog, GameRuleResolutionReceipt } from '@asha/contracts';
import { type GameRuleCatalogValidationReceipt, type GameRuleEffectIntentRequest, type GameRuleRuntimeReadout } from './bridge.js';
export declare class MockGameRuleRuntime {
    #private;
    reset(): void;
    validateCatalog(catalog: GameRuleCatalog): GameRuleCatalogValidationReceipt;
    submitEffectIntent(input: GameRuleEffectIntentRequest): GameRuleResolutionReceipt;
    readRuntimeReadout(): GameRuleRuntimeReadout;
}
//# sourceMappingURL=mock-game-rules.d.ts.map