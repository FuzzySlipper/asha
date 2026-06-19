import type { StudioCommandCatalog, StudioCommandDefinition, StudioCommandManifest } from './types.js';
export declare function buildCommandCatalog(manifest: StudioCommandManifest): StudioCommandCatalog;
export declare function requireCatalogCommand(id: string, catalog: StudioCommandCatalog, manifest?: readonly StudioCommandDefinition<object, object>[]): StudioCommandCatalog['commands'][number];
export declare const COMMAND_CATALOG: StudioCommandCatalog;
//# sourceMappingURL=catalog.d.ts.map