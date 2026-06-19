import type { SchemaShape, StateImpact, StudioCommandDefinition } from './types.js';

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

function mutatesOrWrites(impact: StateImpact): boolean {
  return impact.authority === 'mutate' || impact.editor === 'mutate' || impact.render === 'capture' || impact.workspace === 'write';
}

function isNonEmptyString(value: unknown): boolean {
  return typeof value === 'string' && value.trim().length > 0;
}

function arraysEqual(left: readonly string[] | undefined, right: readonly string[] | undefined): boolean {
  if (left === undefined || right === undefined || left.length !== right.length) {
    return false;
  }
  return left.every((value, index) => value === right[index]);
}

function visitSchemaShape(commandId: string, fieldPath: string, shape: SchemaShape, issues: ManifestValidationIssue[]): void {
  switch (shape.kind) {
    case 'empty':
    case 'contract':
    case 'literal':
    case 'scalar':
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
    case 'nullable':
      visitSchemaShape(commandId, `${fieldPath}?`, shape.inner, issues);
      return;
  }
}

function hasField(value: object, fieldName: string): boolean {
  return Object.prototype.hasOwnProperty.call(value, fieldName);
}

function validateValueAgainstShape(value: unknown, shape: SchemaShape): boolean {
  switch (shape.kind) {
    case 'empty':
      return typeof value === 'object' && value !== null && Object.keys(value).length === 1 && hasField(value, 'kind');
    case 'contract':
      return typeof value === 'object' && value !== null;
    case 'literal':
      return typeof value === 'string' && shape.values.includes(value);
    case 'nullable':
      return value === null || validateValueAgainstShape(value, shape.inner);
    case 'scalar':
      switch (shape.scalar) {
        case 'string':
        case 'state_hash':
        case 'artifact_ref':
          return typeof value === 'string';
        case 'number':
          return typeof value === 'number' && Number.isFinite(value);
        case 'integer':
          return typeof value === 'number' && Number.isInteger(value);
        case 'boolean':
          return typeof value === 'boolean';
        case 'null':
          return value === null;
      }
    case 'array':
      return Array.isArray(value) && (shape.minItems === undefined || value.length >= shape.minItems) && value.every((item) => validateValueAgainstShape(item, shape.items));
    case 'object': {
      if (typeof value !== 'object' || value === null || Array.isArray(value)) {
        return false;
      }
      const keys = Object.keys(value);
      const allowed = new Set(shape.fields.map((field) => field.name));
      if (keys.some((key) => !allowed.has(key))) {
        return false;
      }
      for (const field of shape.fields) {
        if (!hasField(value, field.name)) {
          if (field.required) {
            return false;
          }
          continue;
        }
        if (!validateValueAgainstShape((value as { readonly [key: string]: unknown })[field.name], field.shape)) {
          return false;
        }
      }
      return true;
    }
  }
}

export function validateExampleAgainstSchema(commandId: string, field: 'typedInputExample' | 'typedOutputExample', value: object, schemaShape: SchemaShape): readonly ManifestValidationIssue[] {
  if (validateValueAgainstShape(value, schemaShape)) {
    return [];
  }
  return [{ commandId, field, message: `${field} does not match its declared schema` }];
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
    if (!isNonEmptyString(definition.label)) {
      issues.push({ commandId, field: 'label', message: 'agent-exposed commands require a human-visible label' });
    }
    if (!isNonEmptyString(definition.summary)) {
      issues.push({ commandId, field: 'summary', message: 'agent-exposed commands require a human-visible summary' });
    }
    if (definition.operationClass === undefined) {
      issues.push({ commandId, field: 'operationClass', message: 'agent-exposed commands require an operation class' });
    }
    if (definition.owningLane === undefined) {
      issues.push({ commandId, field: 'owningLane', message: 'agent-exposed commands require owning lane metadata' });
    }
    if (definition.owningPackage === undefined) {
      issues.push({ commandId, field: 'owningPackage', message: 'agent-exposed commands require owning package metadata' });
    }
    if (definition.guiMirror?.required !== true) {
      issues.push({ commandId, field: 'guiMirror.required', message: 'agent-exposed commands require a GUI mirror' });
    }
    if (definition.guiMirror?.menuPath === undefined || definition.guiMirror.menuPath.length === 0) {
      issues.push({ commandId, field: 'guiMirror.menuPath', message: 'agent-exposed commands require GUI/menu path metadata' });
    }
    if (!arraysEqual(definition.guiMirror?.menuPath, definition.menuPath)) {
      issues.push({ commandId, field: 'guiMirror.menuPath', message: 'GUI mirror menu path must match command menu path' });
    }
    if (definition.guiMirror?.commandPaletteVisible !== true && definition.guiMirror?.panel === undefined) {
      issues.push({ commandId, field: 'guiMirror', message: 'agent-exposed commands require command-palette visibility or a panel route' });
    }
    if (!isNonEmptyString(definition.guiMirror?.argumentSummary)) {
      issues.push({ commandId, field: 'guiMirror.argumentSummary', message: 'agent-exposed commands require GUI argument summary metadata' });
    }
    if (!isNonEmptyString(definition.guiMirror?.resultSummary)) {
      issues.push({ commandId, field: 'guiMirror.resultSummary', message: 'agent-exposed commands require GUI result/output summary metadata' });
    }
    if (!isNonEmptyString(definition.guiMirror?.artifactSummary)) {
      issues.push({ commandId, field: 'guiMirror.artifactSummary', message: 'agent-exposed commands require GUI artifact summary metadata' });
    }
  }

  if (definition.agentExposure?.kind === 'read_only') {
    if (definition.operationClass !== undefined && definition.operationClass !== 'read_only') {
      issues.push({ commandId, field: 'agentExposure', message: 'read_only exposure is only valid for read_only operations' });
    }
    if (definition.stateImpact !== undefined && mutatesOrWrites(definition.stateImpact)) {
      issues.push({ commandId, field: 'agentExposure', message: 'read_only exposure is invalid for mutating/writing/capturing state impacts' });
    }
  }

  if (definition.inputSchema !== undefined) {
    visitSchemaShape(commandId, 'inputSchema.shape', definition.inputSchema.shape, issues);
  }
  if (definition.outputSchema !== undefined) {
    visitSchemaShape(commandId, 'outputSchema.shape', definition.outputSchema.shape, issues);
  }
  if (definition.inputSchema !== undefined && definition.typedInputExample !== undefined) {
    issues.push(...validateExampleAgainstSchema(commandId, 'typedInputExample', definition.typedInputExample, definition.inputSchema.shape));
  }
  if (definition.outputSchema !== undefined && definition.typedOutputExample !== undefined) {
    issues.push(...validateExampleAgainstSchema(commandId, 'typedOutputExample', definition.typedOutputExample, definition.outputSchema.shape));
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
