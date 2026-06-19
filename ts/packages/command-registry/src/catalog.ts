import { COMMAND_MANIFEST } from './manifest.js';
import type { StudioCommandCatalog, StudioCommandDefinition, StudioCommandManifest } from './types.js';
import { requireKnownCommand } from './validation.js';

export function buildCommandCatalog(manifest: StudioCommandManifest): StudioCommandCatalog {
  const commands = manifest.map((command) => ({
    id: command.id,
    version: command.version,
    label: command.label,
    summary: command.summary,
    category: command.category,
    operationClass: command.operationClass,
    agentExposureKind: command.agentExposure.kind,
    menuPath: command.menuPath,
    commandPaletteVisible: command.commandPalette.visible,
    commandPaletteKeywords: command.commandPalette.keywords,
    guiMirror: command.guiMirror,
    inputSchemaName: command.inputSchema.name,
    outputSchemaName: command.outputSchema.name,
    inputContractRefs: command.inputContractRefs,
    outputContractRefs: command.outputContractRefs,
    artifacts: command.artifacts,
    stateImpact: command.stateImpact,
    owningLane: command.owningLane,
    owningPackage: command.owningPackage,
    runtimeRequirements: command.runtimeRequirements,
  }));
  return {
    schemaVersion: 1,
    generatedFrom: 'COMMAND_MANIFEST',
    commandRegistryVersion: 'command-registry.v0',
    commands,
  };
}

export function requireCatalogCommand(id: string, catalog: StudioCommandCatalog, manifest: readonly StudioCommandDefinition<object, object>[] = COMMAND_MANIFEST): StudioCommandCatalog['commands'][number] {
  requireKnownCommand(id, manifest);
  const found = catalog.commands.find((command) => command.id === id);
  if (found === undefined) {
    throw new Error(`Command catalog is missing ASHA studio command id: ${id}`);
  }
  return found;
}

export const COMMAND_CATALOG = buildCommandCatalog(COMMAND_MANIFEST);
