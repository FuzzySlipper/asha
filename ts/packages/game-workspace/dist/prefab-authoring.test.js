import { test } from 'node:test';
import assert from 'node:assert/strict';
import { prefabId, prefabInstanceId, prefabPartId, } from '@asha/contracts';
import { applyAshaPrefabAuthoringCommand, buildAshaPrefabDefinition, buildAshaPrefabPart, createAshaPrefabAuthoringState, createAshaPrefabCommand, deleteAshaPrefabCommand, instantiateAshaPrefabCommand, readAshaPrefabAuthoring, replaceAshaPrefabCommand, serializeAshaPrefabRegistrySource, validateAshaPrefabRegistrySourceDocument, } from './index.js';
const prefab = prefabId(70);
const bodyPart = prefabPartId(1);
const sensorPart = prefabPartId(2);
function definition(displayName = 'Interaction console') {
    return buildAshaPrefabDefinition({
        id: prefab,
        displayName,
        parts: [
            buildAshaPrefabPart({
                id: bodyPart,
                namespace: 'body',
                displayName: 'Body',
                source: { kind: 'entityDefinition', stableId: 'demo.console.body' },
            }),
            buildAshaPrefabPart({
                id: sensorPart,
                namespace: 'sensor',
                displayName: 'Sensor',
                parent: bodyPart,
                source: { kind: 'entityDefinition', stableId: 'demo.console.sensor' },
            }),
        ],
        partRoles: [
            { role: 'interaction/sensor', part: sensorPart },
            { role: 'console/body', part: bodyPart },
        ],
    });
}
function bindings() {
    const module = {
        moduleId: 'demo.console-interaction',
        namespace: 'demo.console-interaction',
        version: '1.0.0',
        sdkHash: 'sha256:sdk',
        contractHash: 'sha256:contract',
        artifactHash: 'sha256:artifact',
        providerId: 'provider.demo.console-interaction',
    };
    const contract = {
        namespace: 'demo.console-interaction',
        name: 'configuration',
        version: 1,
        schemaHash: 'sha256:configuration',
    };
    return {
        schemaVersion: 1,
        configurations: [
            {
                configurationId: 'console.blue',
                module,
                configuration: contract,
                codecId: 'codec.console.configuration',
                canonicalConfig: [1],
                configHash: 'sha256:blue',
            },
            {
                configurationId: 'console.red',
                module,
                configuration: contract,
                codecId: 'codec.console.configuration',
                canonicalConfig: [2],
                configHash: 'sha256:red',
            },
        ],
        bindings: [{
                bindingId: 'console.sensor.binding',
                moduleId: module.moduleId,
                configurationId: 'console.blue',
                stateSchema: { ...contract, name: 'state', schemaHash: 'sha256:state' },
                target: { kind: 'prefabPart', part: { prefab, role: 'interaction/sensor' } },
                requiredReads: [],
                outputContracts: [],
                enabled: true,
            }],
        overrides: [{
                bindingId: 'console.sensor.binding',
                prefabInstance: prefabInstanceId(701),
                configurationId: 'console.red',
                enabled: null,
            }],
        registryHash: 'fnv1a64:fixture',
    };
}
void test('public prefab authoring creates edits places and inspects a multi-part definition', () => {
    let state = createAshaPrefabAuthoringState(bindings());
    const created = applyAshaPrefabAuthoringCommand(state, createAshaPrefabCommand(definition()));
    assert.equal(created.ok, true);
    if (!created.ok)
        throw new Error('create should succeed');
    state = created.state;
    assert.deepEqual(created.readout.selected?.parts.map((part) => part.namespace), ['body', 'sensor']);
    assert.deepEqual(created.readout.selected?.roles.map((role) => role.role), ['console/body', 'interaction/sensor']);
    const edited = applyAshaPrefabAuthoringCommand(state, replaceAshaPrefabCommand(definition('Renamed console')));
    assert.equal(edited.ok, true);
    if (!edited.ok)
        throw new Error('edit should succeed');
    state = edited.state;
    assert.equal(edited.readout.selected?.displayName, 'Renamed console');
    for (const command of [
        instantiateAshaPrefabCommand({
            origin: 'authored',
            instance: prefabInstanceId(700),
            prefab,
            seed: 11,
            overrides: [{
                    targetRole: 'console/body',
                    value: { field: 'entityDefinition', stableId: 'demo.console.body.blue' },
                }],
        }),
        instantiateAshaPrefabCommand({
            origin: 'player',
            instance: prefabInstanceId(701),
            prefab,
            seed: 12,
            overrides: [{
                    targetRole: 'console/body',
                    value: { field: 'entityDefinition', stableId: 'demo.console.body.red' },
                }],
        }),
    ]) {
        const placed = applyAshaPrefabAuthoringCommand(state, command);
        assert.equal(placed.ok, true);
        if (!placed.ok)
            throw new Error('placement should succeed');
        state = placed.state;
    }
    const readout = readAshaPrefabAuthoring(state);
    assert.deepEqual(readout.instances.map((instance) => instance.origin), ['authored', 'player']);
    assert.deepEqual(readout.instances.map((instance) => instance.overrideFields), [
        ['console/body.entityDefinition'],
        ['console/body.entityDefinition'],
    ]);
    assert.equal(readout.bindings[0]?.role, 'interaction/sensor');
    assert.equal(readout.bindings[0]?.instanceOverrides[0]?.configurationId, 'console.red');
    assert.deepEqual(readout.configurations.map((configuration) => configuration.configurationId), ['console.blue', 'console.red']);
    assert.deepEqual(readout.nonClaims, ['nestedPrefabs', 'propagatingDefinitionEdits', 'runtimeAuthority']);
    const encoded = serializeAshaPrefabRegistrySource(state.registry);
    assert.match(encoded, /"displayName": "Renamed console"/);
    assert.ok(encoded.endsWith('\n'));
    const rejectedDelete = applyAshaPrefabAuthoringCommand(state, deleteAshaPrefabCommand(prefab));
    assert.equal(rejectedDelete.ok, false);
    assert.equal(!rejectedDelete.ok && rejectedDelete.diagnostics[0]?.code, 'prefabInUse');
});
void test('public prefab authoring rejects duplicate identity and unknown stable roles', () => {
    const initial = createAshaPrefabAuthoringState();
    const created = applyAshaPrefabAuthoringCommand(initial, createAshaPrefabCommand(definition()));
    assert.equal(created.ok, true);
    if (!created.ok)
        throw new Error('create should succeed');
    const duplicate = applyAshaPrefabAuthoringCommand(created.state, createAshaPrefabCommand(definition()));
    assert.equal(duplicate.ok, false);
    assert.equal(!duplicate.ok && duplicate.diagnostics.some((diagnostic) => diagnostic.code === 'duplicatePrefab'), true);
    const badPlacement = applyAshaPrefabAuthoringCommand(created.state, instantiateAshaPrefabCommand({
        origin: 'player',
        instance: prefabInstanceId(702),
        prefab,
        seed: 13,
        overrides: [{ targetRole: 'display-name/Sensor', value: { field: 'activation', active: false } }],
    }));
    assert.equal(badPlacement.ok, false);
    assert.equal(!badPlacement.ok && badPlacement.diagnostics[0]?.code, 'unknownOverrideRole');
});
void test('whole-registry draft validation resolves variant bases and rejects duplicate prefab ids', () => {
    const base = definition();
    const variant = buildAshaPrefabDefinition({
        id: prefabId(71),
        displayName: 'Interaction console variant',
        parts: [],
        partRoles: [],
        variant: {
            base: prefab,
            removedRoles: [],
            overrides: [],
        },
    });
    assert.deepEqual(validateAshaPrefabRegistrySourceDocument({
        schemaVersion: 1,
        definitions: [variant, base],
    }), []);
    assert.equal(validateAshaPrefabRegistrySourceDocument({
        schemaVersion: 1,
        definitions: [base, base],
    }).some((diagnostic) => diagnostic.code === 'duplicatePrefabId'), true);
});
//# sourceMappingURL=prefab-authoring.test.js.map