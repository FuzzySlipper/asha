import {
  PREFAB_DEFINITION_SCHEMA_VERSION,
  PREFAB_REGISTRY_SCHEMA_VERSION,
} from '@asha/contracts';
import type {
  GameplayModuleBindingRegistry,
  PrefabDefinition,
  PrefabId,
  PrefabInstanceId,
  PrefabInstanceRecord,
  PrefabOverride,
  PrefabPart,
  PrefabPartId,
  PrefabPartRoleBinding,
  PrefabRegistry,
  PrefabTransform,
} from '@asha/contracts';

export type AshaPrefabAuthoringDiagnosticCode =
  | 'invalidDefinition'
  | 'duplicatePrefab'
  | 'missingPrefab'
  | 'prefabInUse'
  | 'duplicatePart'
  | 'duplicateRole'
  | 'danglingRole'
  | 'duplicateInstance'
  | 'unknownOverrideRole';

export interface AshaPrefabAuthoringDiagnostic {
  readonly code: AshaPrefabAuthoringDiagnosticCode;
  readonly path: string;
  readonly message: string;
}

export interface AshaPrefabAuthoringState {
  readonly registry: PrefabRegistry;
  readonly instances: readonly AshaPrefabAuthoredInstance[];
  readonly selectedPrefab: PrefabId | null;
  readonly gameplayBindings: GameplayModuleBindingRegistry | null;
}

export interface AshaPrefabAuthoredInstance {
  readonly origin: 'authored' | 'player';
  readonly record: PrefabInstanceRecord;
}

export type AshaPrefabAuthoringCommand =
  | { readonly kind: 'createPrefab'; readonly definition: PrefabDefinition }
  | { readonly kind: 'replacePrefab'; readonly definition: PrefabDefinition }
  | { readonly kind: 'deletePrefab'; readonly prefab: PrefabId }
  | {
      readonly kind: 'instantiatePrefab';
      readonly origin: 'authored' | 'player';
      readonly record: PrefabInstanceRecord;
    };

export type AshaPrefabAuthoringResult =
  | {
      readonly ok: true;
      readonly command: AshaPrefabAuthoringCommand;
      readonly state: AshaPrefabAuthoringState;
      readonly readout: AshaPrefabAuthoringReadout;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly command: AshaPrefabAuthoringCommand;
      readonly state: AshaPrefabAuthoringState;
      readonly diagnostics: readonly AshaPrefabAuthoringDiagnostic[];
    };

export interface AshaPrefabAuthoringReadout {
  readonly registrySchemaVersion: number;
  readonly definitions: readonly AshaPrefabBrowserItem[];
  readonly selected: AshaPrefabDefinitionReadout | null;
  readonly instances: readonly AshaPrefabInstanceReadout[];
  readonly configurations: readonly AshaPrefabConfigurationReadout[];
  readonly bindings: readonly AshaPrefabBindingReadout[];
  readonly nonClaims: readonly ['nestedPrefabs', 'propagatingDefinitionEdits', 'runtimeAuthority'];
}

export interface AshaPrefabBrowserItem {
  readonly prefab: PrefabId;
  readonly displayName: string;
  readonly partCount: number;
  readonly roleCount: number;
  readonly variantBase: PrefabId | null;
}

export interface AshaPrefabDefinitionReadout extends AshaPrefabBrowserItem {
  readonly parts: readonly AshaPrefabPartReadout[];
  readonly roles: readonly PrefabPartRoleBinding[];
}

export interface AshaPrefabPartReadout {
  readonly part: PrefabPartId;
  readonly namespace: string;
  readonly displayName: string;
  readonly parent: PrefabPartId | null;
  readonly roles: readonly string[];
  readonly sourceKind: PrefabPart['source']['kind'];
}

export interface AshaPrefabInstanceReadout {
  readonly instance: PrefabInstanceId;
  readonly prefab: PrefabId;
  readonly origin: 'authored' | 'player';
  readonly overrideFields: readonly string[];
}

export interface AshaPrefabConfigurationReadout {
  readonly configurationId: string;
  readonly moduleId: string;
  readonly configHash: string;
}

export interface AshaPrefabBindingReadout {
  readonly bindingId: string;
  readonly moduleId: string;
  readonly configurationId: string;
  readonly prefab: PrefabId;
  readonly role: string | null;
  readonly enabled: boolean;
  readonly instanceOverrides: readonly {
    readonly sceneInstanceId: string;
    readonly configurationId: string | null;
    readonly enabled: boolean | null;
  }[];
}

export const ASHA_PREFAB_IDENTITY_TRANSFORM: PrefabTransform = {
  translation: [0, 0, 0],
  rotation: [0, 0, 0, 1],
  scale: [1, 1, 1],
};

export function createAshaPrefabAuthoringState(
  gameplayBindings: GameplayModuleBindingRegistry | null = null,
): AshaPrefabAuthoringState {
  return {
    registry: { schemaVersion: PREFAB_REGISTRY_SCHEMA_VERSION, definitions: [] },
    instances: [],
    selectedPrefab: null,
    gameplayBindings,
  };
}

export function buildAshaPrefabDefinition(input: {
  readonly id: PrefabId;
  readonly displayName: string;
  readonly parts: readonly PrefabPart[];
  readonly partRoles: readonly PrefabPartRoleBinding[];
  readonly variant?: PrefabDefinition['variant'];
}): PrefabDefinition {
  return canonicalDefinition({
    id: input.id,
    schemaVersion: PREFAB_DEFINITION_SCHEMA_VERSION,
    displayName: input.displayName,
    parts: input.parts,
    partRoles: input.partRoles,
    variant: input.variant ?? null,
  });
}

export function buildAshaPrefabPart(input: {
  readonly id: PrefabPartId;
  readonly namespace: string;
  readonly displayName: string;
  readonly parent?: PrefabPartId | null;
  readonly transform?: PrefabTransform;
  readonly source: PrefabPart['source'];
}): PrefabPart {
  return {
    id: input.id,
    namespace: input.namespace,
    displayName: input.displayName,
    parent: input.parent ?? null,
    transform: input.transform ?? ASHA_PREFAB_IDENTITY_TRANSFORM,
    source: input.source,
  };
}

export function createAshaPrefabCommand(definition: PrefabDefinition): AshaPrefabAuthoringCommand {
  return { kind: 'createPrefab', definition: canonicalDefinition(definition) };
}

export function replaceAshaPrefabCommand(definition: PrefabDefinition): AshaPrefabAuthoringCommand {
  return { kind: 'replacePrefab', definition: canonicalDefinition(definition) };
}

export function deleteAshaPrefabCommand(prefab: PrefabId): AshaPrefabAuthoringCommand {
  return { kind: 'deletePrefab', prefab };
}

export function instantiateAshaPrefabCommand(input: {
  readonly origin: 'authored' | 'player';
  readonly instance: PrefabInstanceId;
  readonly prefab: PrefabId;
  readonly seed: number;
  readonly transform?: PrefabTransform;
  readonly overrides?: readonly PrefabOverride[];
}): AshaPrefabAuthoringCommand {
  return {
    kind: 'instantiatePrefab',
    origin: input.origin,
    record: {
      instance: input.instance,
      prefab: input.prefab,
      seed: input.seed,
      transform: input.transform ?? ASHA_PREFAB_IDENTITY_TRANSFORM,
      overrides: canonicalOverrides(input.overrides ?? []),
    },
  };
}

export function selectAshaPrefab(
  state: AshaPrefabAuthoringState,
  prefab: PrefabId | null,
): AshaPrefabAuthoringState {
  if (prefab !== null && !state.registry.definitions.some((definition) => definition.id === prefab)) {
    return state;
  }
  return { ...state, selectedPrefab: prefab };
}

export function applyAshaPrefabAuthoringCommand(
  state: AshaPrefabAuthoringState,
  command: AshaPrefabAuthoringCommand,
): AshaPrefabAuthoringResult {
  const diagnostics = validateCommand(state, command);
  if (diagnostics.length > 0) {
    return { ok: false, command, state, diagnostics };
  }
  let definitions = [...state.registry.definitions];
  let instances = [...state.instances];
  let selectedPrefab = state.selectedPrefab;
  switch (command.kind) {
    case 'createPrefab':
      definitions.push(command.definition);
      selectedPrefab = command.definition.id;
      break;
    case 'replacePrefab':
      definitions = definitions.map((definition) =>
        definition.id === command.definition.id ? command.definition : definition,
      );
      selectedPrefab = command.definition.id;
      break;
    case 'deletePrefab':
      definitions = definitions.filter((definition) => definition.id !== command.prefab);
      selectedPrefab = selectedPrefab === command.prefab ? null : selectedPrefab;
      break;
    case 'instantiatePrefab':
      instances.push({ origin: command.origin, record: command.record });
      break;
  }
  const next: AshaPrefabAuthoringState = {
    registry: {
      schemaVersion: PREFAB_REGISTRY_SCHEMA_VERSION,
      definitions: definitions.map(canonicalDefinition).sort((left, right) => left.id - right.id),
    },
    instances: instances.sort((left, right) => left.record.instance - right.record.instance),
    selectedPrefab,
    gameplayBindings: state.gameplayBindings,
  };
  return { ok: true, command, state: next, readout: readAshaPrefabAuthoring(next), diagnostics: [] };
}

export function readAshaPrefabAuthoring(state: AshaPrefabAuthoringState): AshaPrefabAuthoringReadout {
  const selected = state.registry.definitions.find((definition) => definition.id === state.selectedPrefab) ?? null;
  const bindings = state.gameplayBindings;
  return {
    registrySchemaVersion: state.registry.schemaVersion,
    definitions: state.registry.definitions.map(browserItem),
    selected: selected === null ? null : definitionReadout(selected),
    instances: state.instances.map(({ origin, record }) => ({
      instance: record.instance,
      prefab: record.prefab,
      origin,
      overrideFields: record.overrides.map((override) => `${override.targetRole}.${override.value.field}`),
    })),
    configurations: bindings?.configurations.map((configuration) => ({
      configurationId: configuration.configurationId,
      moduleId: configuration.module.moduleId,
      configHash: configuration.configHash,
    })) ?? [],
    bindings: bindings?.bindings.flatMap((binding) => {
      const target = binding.target;
      if (target.kind !== 'prefab' && target.kind !== 'prefabPart') return [];
      const prefab = target.kind === 'prefab' ? target.prefab : target.part.prefab;
      const role = target.kind === 'prefabPart' ? target.part.role : null;
      return [{
        bindingId: binding.bindingId,
        moduleId: binding.moduleId,
        configurationId: binding.configurationId,
        prefab,
        role,
        enabled: binding.enabled,
        instanceOverrides: bindings.overrides
          .filter((override) => override.bindingId === binding.bindingId)
          .map((override) => ({
            sceneInstanceId: override.sceneInstanceId,
            configurationId: override.configurationId,
            enabled: override.enabled,
          })),
      }];
    }) ?? [],
    nonClaims: ['nestedPrefabs', 'propagatingDefinitionEdits', 'runtimeAuthority'],
  };
}

export function serializeAshaPrefabRegistrySource(registry: PrefabRegistry): string {
  const canonical: PrefabRegistry = {
    schemaVersion: PREFAB_REGISTRY_SCHEMA_VERSION,
    definitions: registry.definitions.map(canonicalDefinition).sort((left, right) => left.id - right.id),
  };
  return `${JSON.stringify(canonical, null, 2)}\n`;
}

function validateCommand(
  state: AshaPrefabAuthoringState,
  command: AshaPrefabAuthoringCommand,
): AshaPrefabAuthoringDiagnostic[] {
  if (command.kind === 'createPrefab') {
    const diagnostics = validateDefinition(state, command.definition);
    if (state.registry.definitions.some((definition) => definition.id === command.definition.id)) {
      diagnostics.push(diag('duplicatePrefab', 'definition.id', `prefab ${command.definition.id} already exists`));
    }
    return diagnostics;
  }
  if (command.kind === 'replacePrefab') {
    if (!state.registry.definitions.some((definition) => definition.id === command.definition.id)) {
      return [diag('missingPrefab', 'definition.id', `prefab ${command.definition.id} does not exist`)];
    }
    return validateDefinition(state, command.definition);
  }
  if (command.kind === 'deletePrefab') {
    if (!state.registry.definitions.some((definition) => definition.id === command.prefab)) {
      return [diag('missingPrefab', 'prefab', `prefab ${command.prefab} does not exist`)];
    }
    if (
      state.instances.some((instance) => instance.record.prefab === command.prefab)
      || state.registry.definitions.some((definition) => definition.variant?.base === command.prefab)
    ) {
      return [diag('prefabInUse', 'prefab', `prefab ${command.prefab} is referenced by an instance or variant`)];
    }
    return [];
  }
  const definition = state.registry.definitions.find((candidate) => candidate.id === command.record.prefab);
  if (definition === undefined) {
    return [diag('missingPrefab', 'record.prefab', `prefab ${command.record.prefab} does not exist`)];
  }
  if (state.instances.some((instance) => instance.record.instance === command.record.instance)) {
    return [diag('duplicateInstance', 'record.instance', `instance ${command.record.instance} already exists`)];
  }
  const roles = new Set(resolveRoles(state.registry, definition));
  return command.record.overrides.flatMap((override, index) =>
    roles.has(override.targetRole)
      ? []
      : [diag('unknownOverrideRole', `record.overrides.${index}.targetRole`, `unknown stable role ${override.targetRole}`)],
  );
}

function validateDefinition(
  state: AshaPrefabAuthoringState,
  definition: PrefabDefinition,
): AshaPrefabAuthoringDiagnostic[] {
  const diagnostics: AshaPrefabAuthoringDiagnostic[] = [];
  if (!Number.isSafeInteger(definition.id) || definition.id <= 0 || definition.displayName.trim().length === 0) {
    diagnostics.push(diag('invalidDefinition', 'definition', 'prefab id and display name must be present'));
  }
  const parts = new Set<number>();
  for (const [index, part] of definition.parts.entries()) {
    if (parts.has(part.id)) diagnostics.push(diag('duplicatePart', `definition.parts.${index}.id`, `duplicate part ${part.id}`));
    parts.add(part.id);
  }
  const roles = new Set<string>();
  for (const [index, role] of definition.partRoles.entries()) {
    if (roles.has(role.role)) diagnostics.push(diag('duplicateRole', `definition.partRoles.${index}.role`, `duplicate role ${role.role}`));
    if (!parts.has(role.part)) diagnostics.push(diag('danglingRole', `definition.partRoles.${index}.part`, `role ${role.role} targets missing part ${role.part}`));
    roles.add(role.role);
  }
  if (definition.variant !== null && !state.registry.definitions.some((candidate) => candidate.id === definition.variant?.base)) {
    diagnostics.push(diag('missingPrefab', 'definition.variant.base', `base prefab ${definition.variant.base} does not exist`));
  }
  return diagnostics;
}

function resolveRoles(registry: PrefabRegistry, definition: PrefabDefinition): string[] {
  if (definition.variant === null) return definition.partRoles.map((role) => role.role);
  const base = registry.definitions.find((candidate) => candidate.id === definition.variant?.base);
  if (base === undefined) return [];
  const removed = new Set(definition.variant.removedRoles);
  return resolveRoles(registry, base).filter((role) => !removed.has(role));
}

function browserItem(definition: PrefabDefinition): AshaPrefabBrowserItem {
  return {
    prefab: definition.id,
    displayName: definition.displayName,
    partCount: definition.parts.length,
    roleCount: definition.partRoles.length,
    variantBase: definition.variant?.base ?? null,
  };
}

function definitionReadout(definition: PrefabDefinition): AshaPrefabDefinitionReadout {
  const rolesByPart = new Map<PrefabPartId, string[]>();
  for (const binding of definition.partRoles) {
    const roles = rolesByPart.get(binding.part) ?? [];
    roles.push(binding.role);
    rolesByPart.set(binding.part, roles);
  }
  return {
    ...browserItem(definition),
    parts: definition.parts.map((part) => ({
      part: part.id,
      namespace: part.namespace,
      displayName: part.displayName,
      parent: part.parent,
      roles: rolesByPart.get(part.id) ?? [],
      sourceKind: part.source.kind,
    })),
    roles: definition.partRoles,
  };
}

function canonicalDefinition(definition: PrefabDefinition): PrefabDefinition {
  return {
    ...definition,
    parts: [...definition.parts].sort((left, right) => left.id - right.id),
    partRoles: [...definition.partRoles].sort((left, right) => compareText(left.role, right.role)),
    variant: definition.variant === null ? null : {
      ...definition.variant,
      removedRoles: [...definition.variant.removedRoles].sort(),
      overrides: canonicalOverrides(definition.variant.overrides),
    },
  };
}

function canonicalOverrides(overrides: readonly PrefabOverride[]): PrefabOverride[] {
  return [...overrides].sort((left, right) => compareText(
    `${left.targetRole}.${left.value.field}`,
    `${right.targetRole}.${right.value.field}`,
  ));
}

function compareText(left: string, right: string): number {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function diag(
  code: AshaPrefabAuthoringDiagnosticCode,
  path: string,
  message: string,
): AshaPrefabAuthoringDiagnostic {
  return { code, path, message };
}
