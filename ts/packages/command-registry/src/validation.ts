import type { SchemaShape, StudioCommandDefinition } from './types.js';

export type DraftStudioCommandDefinition = Partial<StudioCommandDefinition<object, object>>;

export interface ManifestValidationIssue {
  readonly commandId: string;
  readonly field: string;
  readonly message: string;
}

const REQUIRED_METADATA_FIELDS = [
  'id',
  'version',
  'label',
  'summary',
  'category',
  'menuPath',
  'commandPalette',
  'inputSchema',
  'outputSchema',
  'operationClass',
  'agentExposure',
  'guiMirror',
  'undo',
  'retry',
  'idempotency',
  'artifacts',
  'stateImpact',
  'owningLane',
  'owningPackage',
  'runtimeRequirements',
  'compatibility',
] as const;

function commandLabel(definition: DraftStudioCommandDefinition): string {
  return definition.id ?? '<missing id>';
}

function hasOwn(definition: DraftStudioCommandDefinition, field: (typeof REQUIRED_METADATA_FIELDS)[number]): boolean {
  return Object.prototype.hasOwnProperty.call(definition, field);
}

function visitSchemaShape(commandId: string, fieldPath: string, shape: SchemaShape, issues: ManifestValidationIssue[]): void {
  switch (shape.kind) {
    case 'empty':
    case 'contract':
    case 'literal':
    case 'scalar':
    case 'artifactRef':
      return;
    case 'object':
      if (shape.allowExtraFields !== false) {
        issues.push({ commandId, field: fieldPath, message: 'object schemas must fail closed with allowExtraFields=false' });
      }
      for (const field of shape.fields) {
        visitSchemaShape(commandId, `${fieldPath}.${field.name}`, field.shape, issues);
      }
      return;
    case 'array':
      visitSchemaShape(commandId, `${fieldPath}[]`, shape.items, issues);
      return;
  }
}

export function validateCommandDefinition(definition: DraftStudioCommandDefinition): readonly ManifestValidationIssue[] {
  const commandId = commandLabel(definition);
  const issues: ManifestValidationIssue[] = [];

  for (const field of REQUIRED_METADATA_FIELDS) {
    if (!hasOwn(definition, field)) {
      issues.push({ commandId, field, message: 'missing required command metadata' });
    }
  }

  if (definition.id !== undefined && !/^[a-z]+(\.[a-z0-9_]+)+$/.test(definition.id)) {
    issues.push({ commandId, field: 'id', message: 'command id must be stable dotted lowercase' });
  }
  if (definition.version !== undefined && (!Number.isInteger(definition.version) || definition.version < 1)) {
    issues.push({ commandId, field: 'version', message: 'version must be a positive integer' });
  }
  if (definition.menuPath !== undefined && definition.menuPath.length === 0) {
    issues.push({ commandId, field: 'menuPath', message: 'menu path must be visible and non-empty' });
  }
  if (definition.artifacts !== undefined && definition.artifacts.length === 0) {
    issues.push({ commandId, field: 'artifacts', message: 'commands must declare artifacts, even when optional' });
  }

  if (definition.agentExposure !== undefined && definition.agentExposure.kind !== 'hidden') {
    if (definition.guiMirror?.required !== true) {
      issues.push({ commandId, field: 'guiMirror.required', message: 'agent-exposed commands require a GUI mirror' });
    }
    if (definition.guiMirror?.menuPath === undefined || definition.guiMirror.menuPath.length === 0) {
      issues.push({ commandId, field: 'guiMirror.menuPath', message: 'agent-exposed commands require GUI/menu path metadata' });
    }
    if (definition.guiMirror?.commandPaletteVisible !== true && definition.guiMirror?.panel === undefined) {
      issues.push({ commandId, field: 'guiMirror', message: 'agent-exposed commands require command-palette visibility or a panel route' });
    }
  }

  if (definition.inputSchema !== undefined) {
    visitSchemaShape(commandId, 'inputSchema.shape', definition.inputSchema.shape, issues);
  }
  if (definition.outputSchema !== undefined) {
    visitSchemaShape(commandId, 'outputSchema.shape', definition.outputSchema.shape, issues);
  }

  return issues;
}

export function validateCommandManifest(manifest: readonly DraftStudioCommandDefinition[]): readonly ManifestValidationIssue[] {
  const issues: ManifestValidationIssue[] = [];
  const seen = new Set<string>();
  for (const definition of manifest) {
    issues.push(...validateCommandDefinition(definition));
    if (definition.id !== undefined) {
      if (seen.has(definition.id)) {
        issues.push({ commandId: definition.id, field: 'id', message: 'duplicate command id' });
      }
      seen.add(definition.id);
    }
  }
  return issues;
}

export function requireKnownCommand(id: string, manifest: readonly StudioCommandDefinition<object, object>[]): StudioCommandDefinition<object, object> {
  const found = manifest.find((command) => command.id === id);
  if (found === undefined) {
    throw new Error(`Unknown ASHA studio command id: ${id}`);
  }
  return found;
}
