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
];
function commandLabel(definition) {
    return definition.id ?? '<missing id>';
}
function hasOwn(definition, field) {
    return Object.prototype.hasOwnProperty.call(definition, field);
}
function mutatesOrWrites(impact) {
    return impact.authority === 'mutate' || impact.editor === 'mutate' || impact.render === 'capture' || impact.workspace === 'write';
}
function isNonEmptyString(value) {
    return typeof value === 'string' && value.trim().length > 0;
}
function arraysEqual(left, right) {
    if (left === undefined || right === undefined || left.length !== right.length) {
        return false;
    }
    return left.every((value, index) => value === right[index]);
}
function visitSchemaShape(commandId, fieldPath, shape, issues) {
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
function hasField(value, fieldName) {
    return Object.prototype.hasOwnProperty.call(value, fieldName);
}
function isPlainObject(value) {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
}
function hasExactKeys(value, keys) {
    const actual = Object.keys(value);
    return actual.length === keys.length && keys.every((key) => hasField(value, key));
}
function isFiniteNumber(value) {
    return typeof value === 'number' && Number.isFinite(value);
}
function isInteger(value) {
    return typeof value === 'number' && Number.isInteger(value);
}
function isString(value) {
    return typeof value === 'string';
}
function isNumberTuple3(value) {
    return Array.isArray(value) && value.length === 3 && value.every(isFiniteNumber);
}
function isLiteral(value, allowed) {
    return typeof value === 'string' && allowed.includes(value);
}
function isVoxelCoord(value) {
    return isPlainObject(value) && hasExactKeys(value, ['x', 'y', 'z']) && isInteger(value.x) && isInteger(value.y) && isInteger(value.z);
}
function isVoxelValue(value) {
    if (!isPlainObject(value) || !hasField(value, 'kind')) {
        return false;
    }
    if (value.kind === 'empty') {
        return hasExactKeys(value, ['kind']);
    }
    return value.kind === 'solid' && hasExactKeys(value, ['kind', 'material']) && isInteger(value.material);
}
function isVoxelCommand(value) {
    if (!isPlainObject(value) || !hasField(value, 'op')) {
        return false;
    }
    if (value.op === 'setVoxel') {
        return hasExactKeys(value, ['op', 'grid', 'coord', 'value']) && isInteger(value.grid) && isVoxelCoord(value.coord) && isVoxelValue(value.value);
    }
    if (value.op === 'fillRegion') {
        return hasExactKeys(value, ['op', 'grid', 'min', 'max', 'value']) && isInteger(value.grid) && isVoxelCoord(value.min) && isVoxelCoord(value.max) && isVoxelValue(value.value);
    }
    return value.op === 'generateChunk' && hasExactKeys(value, ['op', 'grid', 'chunk', 'seed', 'generatorVersion']) && isInteger(value.grid) && isVoxelCoord(value.chunk) && isInteger(value.seed) && isInteger(value.generatorVersion);
}
function isViewport(value) {
    return value === null || (isPlainObject(value) && hasExactKeys(value, ['width', 'height']) && isFiniteNumber(value.width) && isFiniteNumber(value.height));
}
function isScreenPoint(value) {
    return isPlainObject(value) && hasExactKeys(value, ['x', 'y', 'space']) && isFiniteNumber(value.x) && isFiniteNumber(value.y) && isLiteral(value.space, ['normalized_0_1', 'pixel']);
}
function isScreenPointToPickRayRequest(value) {
    return isPlainObject(value) && hasExactKeys(value, ['camera', 'grid', 'viewport', 'screenPoint', 'maxDistance']) && isInteger(value.camera) && isInteger(value.grid) && isViewport(value.viewport) && isScreenPoint(value.screenPoint) && isFiniteNumber(value.maxDistance);
}
function isPickRaySnapshot(value) {
    return isPlainObject(value) && hasExactKeys(value, ['camera', 'tick', 'grid', 'screenPoint', 'origin', 'direction', 'maxDistance', 'cameraProjectionHash', 'rayHash']) && isInteger(value.camera) && isInteger(value.tick) && isInteger(value.grid) && isScreenPoint(value.screenPoint) && isNumberTuple3(value.origin) && isNumberTuple3(value.direction) && isFiniteNumber(value.maxDistance) && isString(value.cameraProjectionHash) && isString(value.rayHash);
}
function isVoxelSelectionSnapshot(value) {
    return isPlainObject(value) && hasExactKeys(value, ['pickRay', 'outcome', 'selectedVoxel', 'selectedFace', 'editAnchor', 'selectionHash']) && isPickRaySnapshot(value.pickRay) && isLiteral(value.outcome, ['hit', 'miss']) && (value.selectedVoxel === null || isVoxelCoord(value.selectedVoxel)) && (value.selectedFace === null || isLiteral(value.selectedFace, ['posX', 'negX', 'posY', 'negY', 'posZ', 'negZ'])) && (value.editAnchor === null || isVoxelCoord(value.editAnchor)) && isString(value.selectionHash);
}
function validateContractValue(value, exportName) {
    switch (exportName) {
        case 'ScreenPointToPickRayRequest':
            return isScreenPointToPickRayRequest(value);
        case 'VoxelCoord':
            return isVoxelCoord(value);
        case 'VoxelSelectionSnapshot':
            return isVoxelSelectionSnapshot(value);
        case 'VoxelCommand':
            return isVoxelCommand(value);
        default:
            return false;
    }
}
function validateValueAgainstShape(value, shape) {
    switch (shape.kind) {
        case 'empty':
            return isPlainObject(value) && Object.keys(value).length === 1 && value.kind === 'empty';
        case 'contract':
            return validateContractValue(value, shape.ref.exportName);
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
                if (!validateValueAgainstShape(value[field.name], field.shape)) {
                    return false;
                }
            }
            return true;
        }
    }
}
export function validateExampleAgainstSchema(commandId, field, value, schemaShape) {
    if (validateValueAgainstShape(value, schemaShape)) {
        return [];
    }
    return [{ commandId, field, message: `${field} does not match its declared schema` }];
}
export function validateCommandDefinition(definition) {
    const commandId = commandLabel(definition);
    const issues = [];
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
export function validateCommandManifest(manifest) {
    const issues = [];
    const seen = new Set();
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
export function requireKnownCommand(id, manifest) {
    const found = manifest.find((command) => command.id === id);
    if (found === undefined) {
        throw new Error(`Unknown ASHA studio command id: ${id}`);
    }
    return found;
}
//# sourceMappingURL=validation.js.map