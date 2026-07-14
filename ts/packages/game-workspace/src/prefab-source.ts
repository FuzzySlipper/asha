import {
  PREFAB_DEFINITION_SCHEMA_VERSION,
  PREFAB_REGISTRY_SCHEMA_VERSION,
  prefabId,
  prefabPartId,
} from '@asha/contracts';
import type {
  PrefabDefinition,
  PrefabDiagnostic,
  PrefabDiagnosticCode,
  PrefabId,
  PrefabOverride,
  PrefabOverrideValue,
  PrefabPart,
  PrefabPartId,
  PrefabPartRoleBinding,
  PrefabPartSource,
  PrefabRegistry,
  PrefabTransform,
  PrefabVariantDelta,
} from '@asha/contracts';

/**
 * Consumer-known stored sources used only for early authoring diagnostics.
 * Rust ProjectBundle load remains the authority for accepting these identities.
 */
export interface AshaPrefabRegistrySourceValidationContext {
  readonly assetIds: readonly string[];
  readonly entityDefinitionIds: readonly string[];
}

export interface AshaPrefabRegistrySourceDecodeDiagnostic {
  readonly code: 'invalidSourceDocument';
  readonly path: string;
  readonly message: string;
}

export type AshaPrefabRegistrySourceDiagnostic =
  | PrefabDiagnostic
  | AshaPrefabRegistrySourceDecodeDiagnostic;

export type AshaPrefabRegistrySourceResult =
  | {
      readonly ok: true;
      readonly registry: PrefabRegistry;
      readonly diagnostics: readonly [];
      readonly authority: 'typescript_early_diagnostic_only';
    }
  | {
      readonly ok: false;
      readonly registry: null;
      readonly diagnostics: readonly AshaPrefabRegistrySourceDiagnostic[];
      readonly authority: 'typescript_early_diagnostic_only';
    };

const EMPTY_VALIDATION_CONTEXT: AshaPrefabRegistrySourceValidationContext = {
  assetIds: [],
  entityDefinitionIds: [],
};

/**
 * Decode unknown source JSON and expose a typed registry only after the complete
 * consumer-side prefab validation pass succeeds. This is deliberately an early
 * diagnostic boundary, not a substitute for Rust ProjectBundle authority.
 */
export function decodeAndValidateAshaPrefabRegistrySourceDocument(
  source: unknown,
  context: AshaPrefabRegistrySourceValidationContext = EMPTY_VALIDATION_CONTEXT,
): AshaPrefabRegistrySourceResult {
  let registry: PrefabRegistry;
  try {
    registry = decodeRegistry(source);
  } catch (error) {
    const diagnostic = error instanceof SourceDecodeFailure
      ? error.diagnostic
      : {
          code: 'invalidSourceDocument' as const,
          path: '$',
          message: error instanceof Error ? error.message : String(error),
        };
    return {
      ok: false,
      registry: null,
      diagnostics: [diagnostic],
      authority: 'typescript_early_diagnostic_only',
    };
  }

  const diagnostics = validateAshaPrefabRegistrySourceDocument(registry, context);
  if (diagnostics.length > 0) {
    return {
      ok: false,
      registry: null,
      diagnostics,
      authority: 'typescript_early_diagnostic_only',
    };
  }
  return {
    ok: true,
    registry: canonicalRegistry(registry),
    diagnostics: [],
    authority: 'typescript_early_diagnostic_only',
  };
}

/** Validate an already decoded registry with the same stored-policy checks. */
export function validateAshaPrefabRegistrySourceDocument(
  registry: PrefabRegistry,
  context: AshaPrefabRegistrySourceValidationContext = EMPTY_VALIDATION_CONTEXT,
): readonly PrefabDiagnostic[] {
  const diagnostics: PrefabDiagnostic[] = [];
  if (registry.schemaVersion !== PREFAB_REGISTRY_SCHEMA_VERSION) {
    diagnostics.push(diagnostic(
      'unsupportedRegistrySchema',
      'schemaVersion',
      `expected prefab registry schema ${PREFAB_REGISTRY_SCHEMA_VERSION}`,
    ));
  }

  const definitions = new Map<PrefabId, PrefabDefinition>();
  for (const [index, definition] of registry.definitions.entries()) {
    const path = `definitions[${index}]`;
    if (definitions.has(definition.id)) {
      diagnostics.push(diagnostic(
        'duplicatePrefabId',
        `${path}.id`,
        `duplicate prefab id ${definition.id}`,
      ));
    }
    definitions.set(definition.id, definition);
    validateDefinition(definition, context, path, diagnostics);
  }

  validateVariants(definitions, context, diagnostics);
  diagnostics.sort(compareDiagnostic);
  return diagnostics;
}

function validateDefinition(
  definition: PrefabDefinition,
  context: AshaPrefabRegistrySourceValidationContext,
  path: string,
  diagnostics: PrefabDiagnostic[],
): void {
  if (definition.schemaVersion !== PREFAB_DEFINITION_SCHEMA_VERSION) {
    diagnostics.push(diagnostic(
      'unsupportedDefinitionSchema',
      `${path}.schemaVersion`,
      `expected prefab definition schema ${PREFAB_DEFINITION_SCHEMA_VERSION}`,
    ));
  }
  if (definition.displayName.trim().length === 0) {
    diagnostics.push(diagnostic(
      'missingDisplayName',
      `${path}.displayName`,
      'display name must not be blank',
    ));
  }
  if (definition.variant !== null
    && (definition.parts.length > 0 || definition.partRoles.length > 0)) {
    diagnostics.push(diagnostic(
      'variantDefinesParts',
      path,
      'Wave 1 variants are deltas and may not define new parts or roles',
    ));
  }

  const parts = new Map<PrefabPartId, PrefabPart>();
  const namespaces = new Set<string>();
  for (const [index, part] of definition.parts.entries()) {
    const partPath = `${path}.parts[${index}]`;
    if (parts.has(part.id)) {
      diagnostics.push(diagnostic(
        'duplicatePartId',
        `${partPath}.id`,
        `duplicate part id ${part.id}`,
      ));
    }
    parts.set(part.id, part);
    if (!isScopedKey(part.namespace)) {
      diagnostics.push(diagnostic(
        'invalidPartNamespace',
        `${partPath}.namespace`,
        'part namespace must be slash-scoped lowercase kebab-case',
      ));
    } else if (namespaces.has(part.namespace)) {
      diagnostics.push(diagnostic(
        'duplicatePartNamespace',
        `${partPath}.namespace`,
        `duplicate part namespace ${part.namespace}`,
      ));
    }
    namespaces.add(part.namespace);
    if (!isValidTransform(part.transform)) {
      diagnostics.push(diagnostic(
        'invalidPartTransform',
        `${partPath}.transform`,
        'transform values must be finite and scale axes non-zero',
      ));
    }
    validateSource(part.source, context, `${partPath}.source`, diagnostics);
  }

  for (const [index, part] of definition.parts.entries()) {
    if (part.parent !== null && !parts.has(part.parent)) {
      diagnostics.push(diagnostic(
        'missingParentPart',
        `${path}.parts[${index}].parent`,
        `unknown parent part ${part.parent}`,
      ));
    }
  }
  validatePartCycles(parts, path, diagnostics);

  const roles = new Set<string>();
  for (const [index, binding] of definition.partRoles.entries()) {
    const rolePath = `${path}.partRoles[${index}]`;
    if (!isScopedKey(binding.role)) {
      diagnostics.push(diagnostic(
        'invalidPartRole',
        `${rolePath}.role`,
        'part role must be slash-scoped lowercase kebab-case',
      ));
    }
    if (roles.has(binding.role)) {
      diagnostics.push(diagnostic(
        'duplicatePartRole',
        `${rolePath}.role`,
        `duplicate part role ${binding.role}`,
      ));
    }
    roles.add(binding.role);
    if (!parts.has(binding.part)) {
      diagnostics.push(diagnostic(
        'danglingPartRole',
        `${rolePath}.part`,
        `role targets unknown part ${binding.part}`,
      ));
    }
  }
}

function validateSource(
  source: PrefabPartSource,
  context: AshaPrefabRegistrySourceValidationContext,
  path: string,
  diagnostics: PrefabDiagnostic[],
): void {
  if (source.kind === 'entityDefinition') {
    if (!context.entityDefinitionIds.includes(source.stableId)) {
      diagnostics.push(diagnostic(
        'unknownEntityDefinition',
        path,
        `unknown EntityDefinition ${source.stableId}`,
      ));
    }
    return;
  }
  validateAsset(
    source.asset,
    source.kind === 'scene' ? 'scene' : 'voxel-object',
    context,
    path,
    diagnostics,
  );
}

function validateAsset(
  asset: string,
  expectedKind: 'scene' | 'voxel-object' | 'material',
  context: AshaPrefabRegistrySourceValidationContext,
  path: string,
  diagnostics: PrefabDiagnostic[],
): void {
  const actualKind = assetKind(asset);
  if (actualKind === null) {
    diagnostics.push(diagnostic(
      'assetKindMismatch',
      path,
      `malformed ${expectedKind} asset id ${asset}`,
    ));
  } else if (actualKind !== expectedKind) {
    diagnostics.push(diagnostic(
      'assetKindMismatch',
      path,
      `expected ${expectedKind} asset, found ${actualKind}`,
    ));
  } else if (!context.assetIds.includes(asset)) {
    diagnostics.push(diagnostic('unknownAsset', path, `unknown asset ${asset}`));
  }
}

function validatePartCycles(
  parts: ReadonlyMap<PrefabPartId, PrefabPart>,
  path: string,
  diagnostics: PrefabDiagnostic[],
): void {
  for (const start of parts.keys()) {
    const seen = new Set<PrefabPartId>();
    let cursor: PrefabPartId | null = start;
    while (cursor !== null) {
      if (seen.has(cursor)) {
        diagnostics.push(diagnostic(
          'partHierarchyCycle',
          `${path}.parts`,
          `part hierarchy cycle includes ${cursor}`,
        ));
        break;
      }
      seen.add(cursor);
      cursor = parts.get(cursor)?.parent ?? null;
    }
  }
}

function validateVariants(
  definitions: ReadonlyMap<PrefabId, PrefabDefinition>,
  context: AshaPrefabRegistrySourceValidationContext,
  diagnostics: PrefabDiagnostic[],
): void {
  const orderedDefinitions = [...definitions.values()].sort((left, right) => left.id - right.id);
  for (const definition of orderedDefinitions) {
    const variant = definition.variant;
    if (variant === null) {
      continue;
    }
    const path = `prefab[${definition.id}].variant`;
    const base = definitions.get(variant.base);
    if (base === undefined) {
      diagnostics.push(diagnostic(
        'missingBasePrefab',
        `${path}.base`,
        `unknown base prefab ${variant.base}`,
      ));
      continue;
    }
    if (base.id === definition.id) {
      diagnostics.push(diagnostic('variantCycle', path, 'variant may not base itself'));
      continue;
    }
    if (base.variant !== null) {
      const code: PrefabDiagnosticCode = variantChainReaches(base, definition.id, definitions)
        ? 'variantCycle'
        : 'variantDepthExceeded';
      diagnostics.push(diagnostic(code, path, 'Wave 1 permits exactly one variant level'));
      continue;
    }
    validateVariantDelta(variant, base, context, path, diagnostics);
  }
}

function variantChainReaches(
  start: PrefabDefinition,
  target: PrefabId,
  definitions: ReadonlyMap<PrefabId, PrefabDefinition>,
): boolean {
  let cursor: PrefabDefinition | undefined = start;
  const seen = new Set<PrefabId>();
  while (cursor !== undefined) {
    if (cursor.id === target || seen.has(cursor.id)) {
      return true;
    }
    seen.add(cursor.id);
    cursor = cursor.variant === null ? undefined : definitions.get(cursor.variant.base);
  }
  return false;
}

function validateVariantDelta(
  variant: PrefabVariantDelta,
  base: PrefabDefinition,
  context: AshaPrefabRegistrySourceValidationContext,
  path: string,
  diagnostics: PrefabDiagnostic[],
): void {
  const roles = new Map<string, PrefabPartId>(
    base.partRoles.map((binding) => [binding.role, binding.part]),
  );
  const parts = new Map<PrefabPartId, PrefabPart>(
    base.parts.map((part) => [part.id, part]),
  );
  const removed = new Set<string>();
  for (const [index, role] of variant.removedRoles.entries()) {
    if (removed.has(role)) {
      diagnostics.push(diagnostic(
        'duplicateRemovedRole',
        `${path}.removedRoles[${index}]`,
        `role ${role} is removed more than once`,
      ));
    }
    removed.add(role);
    if (!roles.has(role)) {
      diagnostics.push(diagnostic(
        'unknownRemovedRole',
        `${path}.removedRoles[${index}]`,
        `unknown base role ${role}`,
      ));
    }
  }

  const removedParts = new Set<PrefabPartId>();
  for (const role of removed) {
    const part = roles.get(role);
    if (part !== undefined) {
      removedParts.add(part);
    }
  }
  for (const removedPart of removedParts) {
    for (const binding of base.partRoles) {
      if (binding.part === removedPart && !removed.has(binding.role)) {
        diagnostics.push(diagnostic(
          'unsafePartRemoval',
          `${path}.removedRoles`,
          `removing part ${removedPart} through one role would leave retained role ${binding.role} dangling`,
        ));
      }
    }
    if (base.parts.some((part) =>
      part.parent === removedPart && !removedParts.has(part.id))) {
      diagnostics.push(diagnostic(
        'unsafePartRemoval',
        `${path}.removedRoles`,
        `removing part ${removedPart} would leave a retained child dangling`,
      ));
    }
  }

  const targets = new Set<string>();
  for (const [index, item] of variant.overrides.entries()) {
    const itemPath = `${path}.overrides[${index}]`;
    const partId = roles.get(item.targetRole);
    if (partId === undefined) {
      diagnostics.push(diagnostic(
        'invalidOverrideTarget',
        `${itemPath}.targetRole`,
        `unknown base role ${item.targetRole}`,
      ));
      continue;
    }
    if (removedParts.has(partId)) {
      diagnostics.push(diagnostic(
        'deletedRoleReferenced',
        itemPath,
        `override role ${item.targetRole} resolves to removed part ${partId}`,
      ));
    }
    const target = `${item.targetRole}\u0000${item.value.field}`;
    if (targets.has(target)) {
      diagnostics.push(diagnostic(
        'duplicateOverride',
        itemPath,
        `duplicate ${item.value.field} override for ${item.targetRole}`,
      ));
    }
    targets.add(target);
    const part = parts.get(partId);
    if (part !== undefined) {
      validateOverrideValue(item.value, part, context, itemPath, diagnostics);
    }
  }
}

function validateOverrideValue(
  value: PrefabOverrideValue,
  part: PrefabPart,
  context: AshaPrefabRegistrySourceValidationContext,
  path: string,
  diagnostics: PrefabDiagnostic[],
): void {
  if (value.field === 'transform') {
    if (!isValidTransform(value.transform)) {
      diagnostics.push(diagnostic('invalidOverrideValue', path, 'override transform is invalid'));
    }
    return;
  }
  if (value.field === 'entityDefinition') {
    if (part.source.kind !== 'entityDefinition'
      || !context.entityDefinitionIds.includes(value.stableId)) {
      diagnostics.push(diagnostic(
        'invalidOverrideValue',
        path,
        'EntityDefinition override requires an entity-definition part and known id',
      ));
    }
    return;
  }
  if (value.field === 'asset') {
    if (part.source.kind === 'entityDefinition') {
      diagnostics.push(diagnostic(
        'invalidOverrideValue',
        path,
        'asset override cannot target an EntityDefinition part',
      ));
      return;
    }
    validateAsset(
      value.asset,
      part.source.kind === 'scene' ? 'scene' : 'voxel-object',
      context,
      path,
      diagnostics,
    );
    return;
  }
  if (value.field === 'material') {
    if (part.source.kind === 'entityDefinition') {
      diagnostics.push(diagnostic(
        'invalidOverrideValue',
        path,
        'material override requires a Scene or VoxelObject part',
      ));
      return;
    }
    validateAsset(value.asset, 'material', context, path, diagnostics);
  }
}

function decodeRegistry(value: unknown): PrefabRegistry {
  const record = sourceRecord(value, '$');
  return {
    schemaVersion: sourceU32(field(record, 'schemaVersion', '$'), 'schemaVersion'),
    definitions: sourceArray(field(record, 'definitions', '$'), 'definitions')
      .map((definition, index) => decodeDefinition(definition, `definitions[${index}]`)),
  };
}

function decodeDefinition(value: unknown, path: string): PrefabDefinition {
  const record = sourceRecord(value, path);
  const variantValue = record['variant'];
  return {
    id: prefabId(sourceId(field(record, 'id', path), `${path}.id`)),
    schemaVersion: sourceU32(field(record, 'schemaVersion', path), `${path}.schemaVersion`),
    displayName: sourceString(field(record, 'displayName', path), `${path}.displayName`),
    parts: sourceArray(field(record, 'parts', path), `${path}.parts`)
      .map((part, index) => decodePart(part, `${path}.parts[${index}]`)),
    partRoles: sourceArray(field(record, 'partRoles', path), `${path}.partRoles`)
      .map((role, index) => decodePartRole(role, `${path}.partRoles[${index}]`)),
    variant: variantValue === undefined || variantValue === null
      ? null
      : decodeVariant(variantValue, `${path}.variant`),
  };
}

function decodePart(value: unknown, path: string): PrefabPart {
  const record = sourceRecord(value, path);
  const parentValue = record['parent'];
  return {
    id: prefabPartId(sourceId(field(record, 'id', path), `${path}.id`)),
    namespace: sourceString(field(record, 'namespace', path), `${path}.namespace`),
    displayName: sourceString(field(record, 'displayName', path), `${path}.displayName`),
    parent: parentValue === undefined || parentValue === null
      ? null
      : prefabPartId(sourceId(parentValue, `${path}.parent`)),
    transform: decodeTransform(field(record, 'transform', path), `${path}.transform`),
    source: decodeSource(field(record, 'source', path), `${path}.source`),
  };
}

function decodePartRole(value: unknown, path: string): PrefabPartRoleBinding {
  const record = sourceRecord(value, path);
  return {
    role: sourceString(field(record, 'role', path), `${path}.role`),
    part: prefabPartId(sourceId(field(record, 'part', path), `${path}.part`)),
  };
}

function decodeSource(value: unknown, path: string): PrefabPartSource {
  const record = sourceRecord(value, path);
  const kind = sourceString(field(record, 'kind', path), `${path}.kind`);
  if (kind === 'scene') {
    return { kind, asset: sourceString(field(record, 'asset', path), `${path}.asset`) };
  }
  if (kind === 'entityDefinition') {
    return {
      kind,
      stableId: sourceString(field(record, 'stableId', path), `${path}.stableId`),
    };
  }
  if (kind === 'voxelObject') {
    return { kind, asset: sourceString(field(record, 'asset', path), `${path}.asset`) };
  }
  throw sourceFailure(`${path}.kind`, `unknown prefab source kind ${kind}`);
}

function decodeVariant(value: unknown, path: string): PrefabVariantDelta {
  const record = sourceRecord(value, path);
  return {
    base: prefabId(sourceId(field(record, 'base', path), `${path}.base`)),
    removedRoles: sourceArray(field(record, 'removedRoles', path), `${path}.removedRoles`)
      .map((role, index) => sourceString(role, `${path}.removedRoles[${index}]`)),
    overrides: sourceArray(field(record, 'overrides', path), `${path}.overrides`)
      .map((item, index) => decodeOverride(item, `${path}.overrides[${index}]`)),
  };
}

function decodeOverride(value: unknown, path: string): PrefabOverride {
  const record = sourceRecord(value, path);
  return {
    targetRole: sourceString(field(record, 'targetRole', path), `${path}.targetRole`),
    value: decodeOverrideValue(field(record, 'value', path), `${path}.value`),
  };
}

function decodeOverrideValue(value: unknown, path: string): PrefabOverrideValue {
  const record = sourceRecord(value, path);
  const fieldName = sourceString(field(record, 'field', path), `${path}.field`);
  if (fieldName === 'transform') {
    return {
      field: fieldName,
      transform: decodeTransform(field(record, 'transform', path), `${path}.transform`),
    };
  }
  if (fieldName === 'entityDefinition') {
    return {
      field: fieldName,
      stableId: sourceString(field(record, 'stableId', path), `${path}.stableId`),
    };
  }
  if (fieldName === 'asset' || fieldName === 'material') {
    return {
      field: fieldName,
      asset: sourceString(field(record, 'asset', path), `${path}.asset`),
    };
  }
  if (fieldName === 'activation') {
    return {
      field: fieldName,
      active: sourceBoolean(field(record, 'active', path), `${path}.active`),
    };
  }
  throw sourceFailure(`${path}.field`, `unknown prefab override field ${fieldName}`);
}

function decodeTransform(value: unknown, path: string): PrefabTransform {
  const record = sourceRecord(value, path);
  return {
    translation: sourceNumberTuple3(field(record, 'translation', path), `${path}.translation`),
    rotation: sourceNumberTuple4(field(record, 'rotation', path), `${path}.rotation`),
    scale: sourceNumberTuple3(field(record, 'scale', path), `${path}.scale`),
  };
}

function canonicalRegistry(registry: PrefabRegistry): PrefabRegistry {
  return {
    schemaVersion: PREFAB_REGISTRY_SCHEMA_VERSION,
    definitions: registry.definitions
      .map((definition) => ({
        ...definition,
        parts: [...definition.parts].sort((left, right) => left.id - right.id),
        partRoles: [...definition.partRoles].sort((left, right) => compareText(left.role, right.role)),
        variant: definition.variant === null ? null : {
          ...definition.variant,
          removedRoles: [...definition.variant.removedRoles].sort(),
          overrides: [...definition.variant.overrides].sort((left, right) => compareText(
            `${left.targetRole}.${left.value.field}`,
            `${right.targetRole}.${right.value.field}`,
          )),
        },
      }))
      .sort((left, right) => left.id - right.id),
  };
}

function isValidTransform(transform: PrefabTransform): boolean {
  return [...transform.translation, ...transform.rotation, ...transform.scale]
    .every(Number.isFinite)
    && transform.scale.every((axis) => axis !== 0);
}

function isScopedKey(value: string): boolean {
  return value.length > 0 && value.split('/').every(isKebabSegment);
}

function isKebabSegment(value: string): boolean {
  return /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(value);
}

function assetKind(value: string): string | null {
  const [prefix, ...nameSegments] = value.split('/');
  const knownKinds = new Set([
    'material',
    'mesh',
    'sprite',
    'sprite-sheet',
    'texture',
    'audio',
    'font',
    'voxel-volume',
    'voxel-object',
    'script',
    'scene',
  ]);
  if (prefix === undefined
    || !knownKinds.has(prefix)
    || nameSegments.length === 0
    || !nameSegments.every(isKebabSegment)) {
    return null;
  }
  return prefix;
}

function compareDiagnostic(left: PrefabDiagnostic, right: PrefabDiagnostic): number {
  return compareText(left.path, right.path)
    || compareText(left.code, right.code)
    || compareText(left.message, right.message);
}

function compareText(left: string, right: string): number {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function diagnostic(
  code: PrefabDiagnosticCode,
  path: string,
  message: string,
): PrefabDiagnostic {
  return { code, path, message };
}

type SourceRecord = Readonly<Record<string, unknown>>;

class SourceDecodeFailure extends Error {
  constructor(readonly diagnostic: AshaPrefabRegistrySourceDecodeDiagnostic) {
    super(diagnostic.message);
  }
}

function sourceFailure(path: string, message: string): SourceDecodeFailure {
  return new SourceDecodeFailure({ code: 'invalidSourceDocument', path, message });
}

function sourceRecord(value: unknown, path: string): SourceRecord {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw sourceFailure(path, 'expected an object');
  }
  return value as SourceRecord;
}

function field(record: SourceRecord, key: string, path: string): unknown {
  if (!Object.prototype.hasOwnProperty.call(record, key)) {
    throw sourceFailure(path === '$' ? key : `${path}.${key}`, `missing field ${key}`);
  }
  return record[key];
}

function sourceArray(value: unknown, path: string): readonly unknown[] {
  if (!Array.isArray(value)) {
    throw sourceFailure(path, 'expected an array');
  }
  return value;
}

function sourceString(value: unknown, path: string): string {
  if (typeof value !== 'string') {
    throw sourceFailure(path, 'expected a string');
  }
  return value;
}

function sourceBoolean(value: unknown, path: string): boolean {
  if (typeof value !== 'boolean') {
    throw sourceFailure(path, 'expected a boolean');
  }
  return value;
}

function sourceId(value: unknown, path: string): number {
  if (!Number.isSafeInteger(value) || typeof value !== 'number' || value < 0) {
    throw sourceFailure(path, 'expected a non-negative safe integer');
  }
  return value;
}

function sourceU32(value: unknown, path: string): number {
  const number = sourceId(value, path);
  if (number > 0xffff_ffff) {
    throw sourceFailure(path, 'expected an unsigned 32-bit integer');
  }
  return number;
}

function sourceNumber(value: unknown, path: string): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw sourceFailure(path, 'expected a finite number');
  }
  return Math.fround(value);
}

function sourceNumberTuple3(value: unknown, path: string): readonly [number, number, number] {
  const values = sourceArray(value, path);
  if (values.length !== 3) {
    throw sourceFailure(path, 'expected exactly 3 numbers');
  }
  return [
    sourceNumber(values[0], `${path}[0]`),
    sourceNumber(values[1], `${path}[1]`),
    sourceNumber(values[2], `${path}[2]`),
  ];
}

function sourceNumberTuple4(
  value: unknown,
  path: string,
): readonly [number, number, number, number] {
  const values = sourceArray(value, path);
  if (values.length !== 4) {
    throw sourceFailure(path, 'expected exactly 4 numbers');
  }
  return [
    sourceNumber(values[0], `${path}[0]`),
    sourceNumber(values[1], `${path}[1]`),
    sourceNumber(values[2], `${path}[2]`),
    sourceNumber(values[3], `${path}[3]`),
  ];
}
