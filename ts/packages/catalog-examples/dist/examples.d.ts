import type { Catalog } from '@asha/contracts';
import { GENERATED_TUNNEL_GAMEPLAY_PRESET_CATALOG, type FpsGameplayPreset, type FpsGameplayPresetCatalogReadout, type FpsGameplayPresetValidationReport } from '@asha/catalog-core';
export type CatalogExampleAuthorityBoundary = {
    readonly packageRole: '@asha/catalog-examples';
    readonly owns: readonly ['fixture_data', 'invalid_fixture_builders', 'consumer_examples'];
    readonly doesNotOwn: readonly [
        'runtime_authority',
        'state_mutation',
        'command_validation',
        'collision_resolution',
        'combat_damage_application'
    ];
};
export type GeneratedTunnelCatalogExampleBundle = {
    readonly kind: 'catalog_example_bundle.v0';
    readonly exampleId: 'generated_tunnel.catalog_examples.v0';
    readonly gameplayCatalog: typeof GENERATED_TUNNEL_GAMEPLAY_PRESET_CATALOG;
    readonly generatedAssetCatalog: Catalog;
    readonly authorityBoundary: CatalogExampleAuthorityBoundary;
};
export declare const CATALOG_EXAMPLE_AUTHORITY_BOUNDARY: CatalogExampleAuthorityBoundary;
export declare const GENERATED_TUNNEL_ASSET_CATALOG_EXAMPLE: Catalog;
export declare const GENERATED_TUNNEL_CATALOG_EXAMPLE_BUNDLE: GeneratedTunnelCatalogExampleBundle;
export declare function readGeneratedTunnelCatalogExampleReadout(): FpsGameplayPresetCatalogReadout;
export declare function validateGeneratedTunnelCatalogExample(): FpsGameplayPresetValidationReport;
export declare function buildInvalidGeneratedTunnelGameplayPresetExample(): FpsGameplayPreset;
export declare function validateInvalidGeneratedTunnelGameplayPresetExample(): FpsGameplayPresetValidationReport;
//# sourceMappingURL=examples.d.ts.map