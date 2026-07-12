import { PREFAB_DEFINITION_SCHEMA_VERSION, PREFAB_REGISTRY_SCHEMA_VERSION, } from '@asha/contracts';
export const ASHA_PREFAB_IDENTITY_TRANSFORM = {
    translation: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
};
export function createAshaPrefabAuthoringState(gameplayBindings = null) {
    return {
        registry: { schemaVersion: PREFAB_REGISTRY_SCHEMA_VERSION, definitions: [] },
        instances: [],
        selectedPrefab: null,
        gameplayBindings,
    };
}
export function buildAshaPrefabDefinition(input) {
    return canonicalDefinition({
        id: input.id,
        schemaVersion: PREFAB_DEFINITION_SCHEMA_VERSION,
        displayName: input.displayName,
        parts: input.parts,
        partRoles: input.partRoles,
        variant: input.variant ?? null,
    });
}
export function buildAshaPrefabPart(input) {
    return {
        id: input.id,
        namespace: input.namespace,
        displayName: input.displayName,
        parent: input.parent ?? null,
        transform: input.transform ?? ASHA_PREFAB_IDENTITY_TRANSFORM,
        source: input.source,
    };
}
export function createAshaPrefabCommand(definition) {
    return { kind: 'createPrefab', definition: canonicalDefinition(definition) };
}
export function replaceAshaPrefabCommand(definition) {
    return { kind: 'replacePrefab', definition: canonicalDefinition(definition) };
}
export function deleteAshaPrefabCommand(prefab) {
    return { kind: 'deletePrefab', prefab };
}
export function instantiateAshaPrefabCommand(input) {
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
export function selectAshaPrefab(state, prefab) {
    if (prefab !== null && !state.registry.definitions.some((definition) => definition.id === prefab)) {
        return state;
    }
    return { ...state, selectedPrefab: prefab };
}
export function applyAshaPrefabAuthoringCommand(state, command) {
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
            definitions = definitions.map((definition) => definition.id === command.definition.id ? command.definition : definition);
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
    const next = {
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
export function readAshaPrefabAuthoring(state) {
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
            if (target.kind !== 'prefab' && target.kind !== 'prefabPart')
                return [];
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
                        instance: override.prefabInstance,
                        configurationId: override.configurationId,
                        enabled: override.enabled,
                    })),
                }];
        }) ?? [],
        nonClaims: ['nestedPrefabs', 'propagatingDefinitionEdits', 'runtimeAuthority'],
    };
}
export function serializeAshaPrefabRegistrySource(registry) {
    const canonical = {
        schemaVersion: PREFAB_REGISTRY_SCHEMA_VERSION,
        definitions: registry.definitions.map(canonicalDefinition).sort((left, right) => left.id - right.id),
    };
    return `${JSON.stringify(canonical, null, 2)}\n`;
}
export function validateAshaPrefabRegistrySourceDocument(registry) {
    const state = {
        registry,
        instances: [],
        selectedPrefab: null,
        gameplayBindings: null,
    };
    const diagnostics = [];
    const prefabIds = new Set();
    for (const definition of registry.definitions) {
        if (prefabIds.has(definition.id)) {
            diagnostics.push({
                code: 'duplicatePrefabId',
                path: 'definition.id',
                message: `prefab ${definition.id} already exists`,
            });
        }
        prefabIds.add(definition.id);
        for (const item of validateDefinition(state, definition)) {
            diagnostics.push({ code: authoringDiagnosticToPrefabCode(item.code), path: item.path, message: item.message });
        }
    }
    return diagnostics;
}
function validateCommand(state, command) {
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
        if (state.instances.some((instance) => instance.record.prefab === command.prefab)
            || state.registry.definitions.some((definition) => definition.variant?.base === command.prefab)) {
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
    return command.record.overrides.flatMap((override, index) => roles.has(override.targetRole)
        ? []
        : [diag('unknownOverrideRole', `record.overrides.${index}.targetRole`, `unknown stable role ${override.targetRole}`)]);
}
function validateDefinition(state, definition) {
    const diagnostics = [];
    if (!Number.isSafeInteger(definition.id) || definition.id <= 0 || definition.displayName.trim().length === 0) {
        diagnostics.push(diag('invalidDefinition', 'definition', 'prefab id and display name must be present'));
    }
    const parts = new Set();
    for (const [index, part] of definition.parts.entries()) {
        if (parts.has(part.id))
            diagnostics.push(diag('duplicatePart', `definition.parts.${index}.id`, `duplicate part ${part.id}`));
        parts.add(part.id);
    }
    const roles = new Set();
    for (const [index, role] of definition.partRoles.entries()) {
        if (roles.has(role.role))
            diagnostics.push(diag('duplicateRole', `definition.partRoles.${index}.role`, `duplicate role ${role.role}`));
        if (!parts.has(role.part))
            diagnostics.push(diag('danglingRole', `definition.partRoles.${index}.part`, `role ${role.role} targets missing part ${role.part}`));
        roles.add(role.role);
    }
    if (definition.variant !== null && !state.registry.definitions.some((candidate) => candidate.id === definition.variant?.base)) {
        diagnostics.push(diag('missingPrefab', 'definition.variant.base', `base prefab ${definition.variant.base} does not exist`));
    }
    return diagnostics;
}
function resolveRoles(registry, definition) {
    if (definition.variant === null)
        return definition.partRoles.map((role) => role.role);
    const base = registry.definitions.find((candidate) => candidate.id === definition.variant?.base);
    if (base === undefined)
        return [];
    const removed = new Set(definition.variant.removedRoles);
    return resolveRoles(registry, base).filter((role) => !removed.has(role));
}
function browserItem(definition) {
    return {
        prefab: definition.id,
        displayName: definition.displayName,
        partCount: definition.parts.length,
        roleCount: definition.partRoles.length,
        variantBase: definition.variant?.base ?? null,
    };
}
function definitionReadout(definition) {
    const rolesByPart = new Map();
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
function canonicalDefinition(definition) {
    return {
        ...definition,
        parts: [...definition.parts].sort((left, right) => left.id - right.id),
        partRoles: [...definition.partRoles].sort((left, right) => left.role.localeCompare(right.role)),
        variant: definition.variant === null ? null : {
            ...definition.variant,
            removedRoles: [...definition.variant.removedRoles].sort(),
            overrides: canonicalOverrides(definition.variant.overrides),
        },
    };
}
function canonicalOverrides(overrides) {
    return [...overrides].sort((left, right) => `${left.targetRole}.${left.value.field}`.localeCompare(`${right.targetRole}.${right.value.field}`));
}
function authoringDiagnosticToPrefabCode(code) {
    switch (code) {
        case 'duplicatePrefab': return 'duplicatePrefabId';
        case 'duplicatePart': return 'duplicatePartId';
        case 'duplicateRole': return 'duplicatePartRole';
        case 'danglingRole': return 'danglingPartRole';
        case 'unknownOverrideRole': return 'invalidOverrideTarget';
        default: return 'missingDisplayName';
    }
}
function diag(code, path, message) {
    return { code, path, message };
}
//# sourceMappingURL=prefab-authoring.js.map