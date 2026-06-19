const CAMERA_HANDLE = 1;
const field = (name, shape, summary, required = true) => ({ name, required, shape, summary });
const scalar = (scalar) => ({ kind: 'scalar', scalar });
const stringShape = scalar('string');
const booleanShape = scalar('boolean');
const integerShape = scalar('integer');
const hashShape = scalar('state_hash');
const nullable = (inner) => ({ kind: 'nullable', inner });
const objectShape = (fields) => ({ kind: 'object', allowExtraFields: false, fields });
const objectSchema = (name, fields) => ({ name, version: 1, shape: objectShape(fields) });
const arrayOf = (items, minItems) => ({ kind: 'array', items, ...(minItems === undefined ? {} : { minItems }) });
const literal = (values) => ({ kind: 'literal', values });
const contract = (exportName) => ({ kind: 'contract', ref: { package: '@asha/contracts', exportName } });
const EMPTY_INPUT = { name: 'EmptyInput', version: 1, shape: { kind: 'empty' } };
const EMPTY_OUTPUT = objectSchema('EmptyOutput', [field('kind', literal(['ok']), 'Acknowledgement literal.')]);
const SESSION_ID_FIELD = field('sessionId', stringShape, 'Stable studio session identifier.');
const SCENARIO_ID_FIELD = field('scenarioId', stringShape, 'Named public studio scenario identifier.');
const VOXEL_COORD_SCHEMA = contract('VoxelCoord');
const VOXEL_COMMAND_SCHEMA = contract('VoxelCommand');
const COMPAT = {
    contracts: 'contracts.v0',
    runtimeBridge: 'runtime-bridge.v0',
    commandRegistry: 'command-registry.v0',
};
const REGISTRY_COMPAT = { contracts: 'contracts.v0', commandRegistry: 'command-registry.v0' };
const OWNER = '@asha/command-registry';
const noStateImpact = { authority: 'none', editor: 'none', render: 'none', workspace: 'none' };
const readAuthority = { authority: 'read', editor: 'none', render: 'none', workspace: 'none' };
const readEditor = { authority: 'none', editor: 'read', render: 'none', workspace: 'none' };
const mutateEditor = { authority: 'none', editor: 'mutate', render: 'none', workspace: 'none' };
const mutateAuthority = { authority: 'mutate', editor: 'read', render: 'none', workspace: 'none' };
const captureRender = { authority: 'read', editor: 'read', render: 'capture', workspace: 'none' };
const writeWorkspace = { authority: 'read', editor: 'read', render: 'read', workspace: 'write' };
const sessionWorkspace = { authority: 'read', editor: 'mutate', render: 'none', workspace: 'write' };
function artifact(type, summary, required = true) {
    return { type, required, producedWhen: required ? 'always' : 'when_available', summary };
}
function runtime(operation) {
    return { kind: 'runtime_bridge_operation', operation };
}
function summarizeShape(shape) {
    switch (shape.kind) {
        case 'empty':
            return 'No arguments.';
        case 'contract':
            return `Uses ${shape.ref.exportName} from ${shape.ref.package}.`;
        case 'scalar':
            return `${shape.scalar} value.`;
        case 'literal':
            return `One of: ${shape.values.join(', ')}.`;
        case 'nullable':
            return `Nullable ${summarizeShape(shape.inner).replace(/\.$/, '')}.`;
        case 'array':
            return `Array of ${summarizeShape(shape.items).replace(/\.$/, '')}.`;
        case 'object':
            if (shape.fields.length === 0) {
                return 'Object with no fields.';
            }
            return shape.fields.map((fieldDef) => `${fieldDef.name}: ${fieldDef.summary}`).join(' ');
    }
}
function summarizeSchema(schema) {
    return `${schema.name}: ${summarizeShape(schema.shape)}`;
}
function summarizeArtifacts(artifacts) {
    return artifacts.map((decl) => `${decl.type}: ${decl.summary}`).join(' ');
}
function def(definition) {
    return definition;
}
function base(args) {
    const agentExposure = args.agentExposure ?? { kind: 'read_only' };
    return def({
        id: args.id,
        version: 1,
        label: args.label,
        summary: args.summary,
        category: args.category,
        menuPath: args.menuPath,
        commandPalette: { visible: true, keywords: args.keywords },
        inputSchema: args.inputSchema,
        outputSchema: args.outputSchema,
        inputContractRefs: args.inputContractRefs ?? [],
        outputContractRefs: args.outputContractRefs ?? [],
        operationClass: args.operationClass,
        agentExposure,
        guiMirror: {
            required: agentExposure.kind !== 'hidden',
            menuPath: args.menuPath,
            commandPaletteVisible: true,
            argumentSummary: summarizeSchema(args.inputSchema),
            resultSummary: summarizeSchema(args.outputSchema),
            artifactSummary: summarizeArtifacts(args.artifacts),
            ...(args.panel === undefined ? {} : { panel: args.panel }),
            ...(args.dialog === undefined ? {} : { dialog: args.dialog }),
        },
        undo: args.undo ?? { kind: 'not_undoable', reason: 'Read-only or diagnostic command has no mutation to reverse.' },
        retry: args.retry ?? 'safe_to_retry',
        idempotency: args.idempotency ?? { kind: 'idempotent', keyFields: ['sessionId'] },
        artifacts: args.artifacts,
        stateImpact: args.stateImpact,
        owningLane: 'ts-command-registry',
        owningPackage: OWNER,
        runtimeRequirements: args.runtimeRequirements,
        compatibility: args.compatibility ?? COMPAT,
        ...(args.knownLimitations === undefined ? {} : { knownLimitations: args.knownLimitations }),
        typedInputExample: args.typedInputExample,
        typedOutputExample: args.typedOutputExample,
    });
}
const scenarioListOutput = objectSchema('ScenarioListOutput', [
    field('scenarios', arrayOf({ kind: 'object', allowExtraFields: false, fields: [field('id', stringShape, 'Scenario id.'), field('label', stringShape, 'Human-readable scenario label.')] }), 'Bounded public scenario list.'),
]);
const scenarioIdInput = objectSchema('ScenarioIdInput', [SCENARIO_ID_FIELD]);
const sessionIdInput = objectSchema('SessionIdInput', [SESSION_ID_FIELD]);
const sessionStatusOutput = objectSchema('SessionStatusOutput', [SESSION_ID_FIELD, field('status', literal(['not_started', 'ready', 'degraded', 'unavailable']), 'Session/runtime status.')]);
const worldSummaryOutput = objectSchema('WorldSummaryOutput', [
    field('authorityHash', nullable(hashShape), 'Authority hash when the public runtime can provide it.'),
    field('voxelVolumeCount', integerShape, 'Number of public voxel volumes.'),
    field('sceneNodeCount', integerShape, 'Number of public scene nodes.'),
]);
const editorStateOutput = objectSchema('EditorStateOutput', [
    field('editorVersion', stringShape, 'Editor state/schema version.'),
    field('selectedVoxel', nullable(VOXEL_COORD_SCHEMA), 'Currently selected voxel, if any.'),
]);
const screenPointInput = objectSchema('ScreenPointInput', [
    SESSION_ID_FIELD,
    field('request', contract('ScreenPointToPickRayRequest'), 'Generated public screen-point/camera selection request.'),
]);
const voxelSelectionOutput = objectSchema('VoxelSelectionOutput', [field('selection', contract('VoxelSelectionSnapshot'), 'Generated public selection evidence snapshot.')]);
const voxelInspectionInput = objectSchema('VoxelInspectionInput', [SESSION_ID_FIELD, field('voxel', VOXEL_COORD_SCHEMA, 'Voxel coordinate to inspect.')]);
const voxelInspectionOutput = objectSchema('VoxelInspectionOutput', [
    field('voxel', VOXEL_COORD_SCHEMA, 'Inspected voxel coordinate.'),
    field('materialId', nullable(integerShape), 'Material id when occupied.'),
    field('occupied', booleanShape, 'Whether the voxel is occupied.'),
]);
const previewInput = objectSchema('VoxelBrushPreviewInput', [
    SESSION_ID_FIELD,
    field('anchor', VOXEL_COORD_SCHEMA, 'Preview anchor coordinate.'),
    field('commands', arrayOf(VOXEL_COMMAND_SCHEMA, 1), 'Typed voxel command preview set.'),
]);
const previewOutput = objectSchema('VoxelBrushPreviewOutput', [
    field('targetVoxels', arrayOf(VOXEL_COORD_SCHEMA), 'Voxels affected by the preview.'),
    field('previewVersion', stringShape, 'Editor preview version.'),
]);
const applyInput = objectSchema('ApplyVoxelBrushInput', [
    SESSION_ID_FIELD,
    field('commands', arrayOf(VOXEL_COMMAND_SCHEMA, 1), 'Typed authority voxel commands.'),
    field('expectedStateHash', nullable(hashShape), 'Expected authority state hash or null when unavailable.'),
]);
const applyOutput = objectSchema('ApplyVoxelBrushOutput', [
    field('accepted', booleanShape, 'Whether authority accepted the command batch.'),
    field('authorityBeforeHash', nullable(hashShape), 'Before hash when available.'),
    field('authorityAfterHash', nullable(hashShape), 'After hash when available.'),
]);
const lastCommandResultOutput = objectSchema('LastCommandResultOutput', [
    field('sequenceId', nullable(stringShape), 'Last command sequence id, if any.'),
    field('status', nullable(literal(['ok', 'rejected', 'partial', 'failed', 'unavailable'])), 'Last command status, if any.'),
]);
const captureInput = objectSchema('CaptureBeforeAfterInput', [
    SESSION_ID_FIELD,
    field('beforeArtifactId', scalar('artifact_ref'), 'Before visual evidence artifact id.'),
    field('afterArtifactId', scalar('artifact_ref'), 'After visual evidence artifact id.'),
]);
const captureOutput = objectSchema('CaptureBeforeAfterOutput', [
    field('artifactId', scalar('artifact_ref'), 'Combined before/after artifact id.'),
    field('renderBeforeHash', nullable(hashShape), 'Before render evidence hash when available.'),
    field('renderAfterHash', nullable(hashShape), 'After render evidence hash when available.'),
]);
const exportInput = objectSchema('ExportAgentReadoutInput', [SESSION_ID_FIELD, field('includeVisualEvidence', booleanShape, 'Whether exported readout references visual artifacts.')]);
const exportOutput = objectSchema('ExportAgentReadoutOutput', [field('artifactId', scalar('artifact_ref'), 'Exported readout artifact id.'), field('commandCount', integerShape, 'Number of commands included in the readout.')]);
const selectionExample = {
    pickRay: {
        camera: CAMERA_HANDLE,
        tick: 0,
        grid: 0,
        screenPoint: { x: 0.5, y: 0.5, space: 'normalized_0_1' },
        origin: [0, 0, 0],
        direction: [1, 0, 0],
        maxDistance: 128,
        cameraProjectionHash: 'projection-hash',
        rayHash: 'ray-hash',
    },
    outcome: 'miss',
    selectedVoxel: null,
    selectedFace: null,
    editAnchor: null,
    selectionHash: 'selection-hash',
};
export const COMMAND_MANIFEST = [
    base({
        id: 'session.list_scenarios', label: 'List Studio Scenarios', summary: 'List named public scenarios available to a studio session.', category: 'session', menuPath: ['Session', 'List Scenarios'], keywords: ['scenario', 'list'],
        inputSchema: EMPTY_INPUT, outputSchema: scenarioListOutput, operationClass: 'read_only', stateImpact: noStateImpact, compatibility: REGISTRY_COMPAT, runtimeRequirements: [{ kind: 'none' }], artifacts: [artifact('scenario_manifest', 'Bounded scenario list for session loading.')], typedInputExample: { kind: 'empty' }, typedOutputExample: { scenarios: [{ id: 'voxel-basic', label: 'Basic Voxel Scenario' }] }, panel: 'inspector', dialog: 'readout_only', idempotency: { kind: 'idempotent', keyFields: [] },
    }),
    base({
        id: 'session.start', label: 'Start Studio Session', summary: 'Create/reset a studio session around a named scenario.', category: 'session', menuPath: ['Session', 'Start'], keywords: ['session', 'start'],
        inputSchema: scenarioIdInput, outputSchema: EMPTY_OUTPUT, operationClass: 'workspace_io', agentExposure: { kind: 'workspace_io', batchable: false }, stateImpact: sessionWorkspace, runtimeRequirements: [runtime('initialize_engine'), runtime('load_world_bundle')], artifacts: [artifact('session_status', 'Initial session status and compatibility readback.')], typedInputExample: { scenarioId: 'voxel-basic' }, typedOutputExample: { kind: 'ok' }, panel: 'timeline', dialog: 'simple_form', retry: 'retry_after_status_readback', idempotency: { kind: 'conditional', condition: 'Idempotent when scenarioId and session reset token match.' },
    }),
    base({
        id: 'session.load_scenario', label: 'Load Scenario', summary: 'Load a named scenario into the active studio session.', category: 'session', menuPath: ['Session', 'Load Scenario'], keywords: ['load', 'scenario'],
        inputSchema: scenarioIdInput, outputSchema: EMPTY_OUTPUT, operationClass: 'workspace_io', agentExposure: { kind: 'workspace_io', batchable: false }, stateImpact: sessionWorkspace, runtimeRequirements: [runtime('load_world_bundle')], artifacts: [artifact('session_status', 'Scenario load status and diagnostics.')], typedInputExample: { scenarioId: 'voxel-basic' }, typedOutputExample: { kind: 'ok' }, panel: 'timeline', dialog: 'simple_form', retry: 'retry_after_status_readback', idempotency: { kind: 'conditional', condition: 'Safe when current session already targets the same scenario id.' },
    }),
    base({
        id: 'inspection.session_status', label: 'Inspect Session Status', summary: 'Read studio/runtime readiness, compatibility, and degradation status.', category: 'inspection', menuPath: ['Inspect', 'Session Status'], keywords: ['status', 'compatibility'],
        inputSchema: sessionIdInput, outputSchema: sessionStatusOutput, operationClass: 'read_only', stateImpact: noStateImpact, runtimeRequirements: [runtime('get_composition_status')], artifacts: [artifact('session_status', 'Status readback for the active session.')], typedInputExample: { sessionId: 'session-1' }, typedOutputExample: { sessionId: 'session-1', status: 'ready' }, panel: 'diagnostics', dialog: 'readout_only',
    }),
    base({
        id: 'inspection.world_summary', label: 'Inspect World Summary', summary: 'Read compact public world and authority evidence.', category: 'inspection', menuPath: ['Inspect', 'World Summary'], keywords: ['world', 'hash'],
        inputSchema: sessionIdInput, outputSchema: worldSummaryOutput, operationClass: 'read_only', stateImpact: readAuthority, runtimeRequirements: [runtime('read_voxel_mesh_evidence')], artifacts: [artifact('world_summary', 'Compact authority/render-neutral world summary.')], typedInputExample: { sessionId: 'session-1' }, typedOutputExample: { authorityHash: null, voxelVolumeCount: 1, sceneNodeCount: 1 }, panel: 'inspector', dialog: 'readout_only',
    }),
    base({
        id: 'inspection.editor_state', label: 'Inspect Editor State', summary: 'Read command-registry/editor-local selection and preview state.', category: 'inspection', menuPath: ['Inspect', 'Editor State'], keywords: ['editor', 'selection'],
        inputSchema: sessionIdInput, outputSchema: editorStateOutput, operationClass: 'read_only', stateImpact: readEditor, runtimeRequirements: [{ kind: 'editor_store' }], artifacts: [artifact('editor_state', 'Editor-local state snapshot.')], typedInputExample: { sessionId: 'session-1' }, typedOutputExample: { editorVersion: 'editor.v0', selectedVoxel: null }, panel: 'inspector', dialog: 'readout_only', compatibility: REGISTRY_COMPAT,
    }),
    base({
        id: 'selection.voxel_from_screen_point', label: 'Select Voxel From Screen Point', summary: 'Project a screen point through public camera evidence into typed ASHA voxel selection evidence.', category: 'selection', menuPath: ['Select', 'Voxel From Screen Point'], keywords: ['screen point', 'pick', 'select', 'voxel'],
        inputSchema: screenPointInput, outputSchema: voxelSelectionOutput, operationClass: 'editor_local', stateImpact: mutateEditor, runtimeRequirements: [runtime('select_voxel'), { kind: 'editor_store' }], artifacts: [artifact('selection_snapshot', 'Selected voxel hit or no-hit result.')], typedInputExample: { sessionId: 'session-1', request: { camera: CAMERA_HANDLE, grid: 0, viewport: null, screenPoint: { x: 0.5, y: 0.5, space: 'normalized_0_1' }, maxDistance: 128 } }, typedOutputExample: { selection: selectionExample }, panel: 'viewport', dialog: 'none', inputContractRefs: [{ package: '@asha/contracts', exportName: 'ScreenPointToPickRayRequest' }], outputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelSelectionSnapshot' }], agentExposure: { kind: 'editor_local' }, undo: { kind: 'editor_local', inverseData: ['previous selection snapshot'] }, idempotency: { kind: 'conditional', condition: 'Same screen point, camera, viewport, and unchanged projection evidence selects the same voxel.' },
    }),
    base({
        id: 'inspection.voxel', label: 'Inspect Voxel', summary: 'Read typed voxel/material state for one public coordinate.', category: 'inspection', menuPath: ['Inspect', 'Voxel'], keywords: ['voxel', 'inspect'],
        inputSchema: voxelInspectionInput, outputSchema: voxelInspectionOutput, operationClass: 'read_only', stateImpact: readAuthority, runtimeRequirements: [runtime('read_voxel_mesh_evidence')], artifacts: [artifact('voxel_inspection', 'Voxel occupancy/material readout.')], typedInputExample: { sessionId: 'session-1', voxel: { x: 0, y: 0, z: 0 } }, typedOutputExample: { voxel: { x: 0, y: 0, z: 0 }, materialId: null, occupied: false }, panel: 'inspector', dialog: 'readout_only', inputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCoord' }], outputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCoord' }],
    }),
    base({
        id: 'preview.voxel_brush', label: 'Preview Voxel Brush', summary: 'Preview a typed voxel edit without mutating authority.', category: 'preview', menuPath: ['Edit', 'Preview Voxel Brush'], keywords: ['preview', 'brush', 'voxel'],
        inputSchema: previewInput, outputSchema: previewOutput, operationClass: 'editor_local', stateImpact: mutateEditor, runtimeRequirements: [{ kind: 'editor_store' }], artifacts: [artifact('voxel_preview', 'Editor-local target voxel preview.')], typedInputExample: { sessionId: 'session-1', anchor: { x: 0, y: 0, z: 0 }, commands: [{ op: 'setVoxel', grid: 0, coord: { x: 0, y: 0, z: 0 }, value: { kind: 'solid', material: 1 } }] }, typedOutputExample: { targetVoxels: [{ x: 0, y: 0, z: 0 }], previewVersion: 'preview.v0' }, panel: 'viewport', dialog: 'simple_form', inputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCoord' }, { package: '@asha/contracts', exportName: 'VoxelCommand' }], outputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCoord' }], agentExposure: { kind: 'editor_local' }, undo: { kind: 'editor_local', inverseData: ['previous preview snapshot'] }, idempotency: { kind: 'idempotent', keyFields: ['sessionId', 'anchor', 'commands'] }, compatibility: REGISTRY_COMPAT,
    }),
    base({
        id: 'authority.voxel.apply_brush', label: 'Apply Voxel Brush', summary: 'Apply typed voxel commands through ASHA authority validation.', category: 'authority_edit', menuPath: ['Edit', 'Apply Voxel Brush'], keywords: ['apply', 'voxel', 'authority'],
        inputSchema: applyInput, outputSchema: applyOutput, operationClass: 'authority_mutating', stateImpact: mutateAuthority, runtimeRequirements: [runtime('submit_commands'), runtime('read_voxel_mesh_evidence')], artifacts: [artifact('command_result', 'Accepted/rejected authority command result with state hash evidence.')], typedInputExample: { sessionId: 'session-1', commands: [{ op: 'setVoxel', grid: 0, coord: { x: 0, y: 0, z: 0 }, value: { kind: 'solid', material: 1 } }], expectedStateHash: null }, typedOutputExample: { accepted: true, authorityBeforeHash: null, authorityAfterHash: null }, panel: 'timeline', dialog: 'advanced_form', inputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCommand' }], agentExposure: { kind: 'authority_mutating', requiresPreview: true, batchable: true }, undo: { kind: 'authority_reversing', inverseCommandRefs: [], requiresSameStateHash: true }, retry: 'safe_to_retry_if_state_hash_unchanged', idempotency: { kind: 'conditional', condition: 'Safe when expectedStateHash still matches and command sequence id has not committed.' }, knownLimitations: ['V1 records reversal posture but does not declare a generic authority undo stack.'],
    }),
    base({
        id: 'inspection.last_command_result', label: 'Inspect Last Command Result', summary: 'Read the last timeline command result for human/agent correlation.', category: 'inspection', menuPath: ['Inspect', 'Last Command Result'], keywords: ['timeline', 'result'],
        inputSchema: sessionIdInput, outputSchema: lastCommandResultOutput, operationClass: 'read_only', stateImpact: readEditor, runtimeRequirements: [{ kind: 'editor_store' }], artifacts: [artifact('command_result', 'Last known command result reference.', false)], typedInputExample: { sessionId: 'session-1' }, typedOutputExample: { sequenceId: null, status: null }, panel: 'timeline', dialog: 'readout_only', compatibility: REGISTRY_COMPAT,
    }),
    base({
        id: 'render.capture_before_after', label: 'Capture Before/After Evidence', summary: 'Capture/render before-after evidence as non-authoritative artifacts.', category: 'render_evidence', menuPath: ['Evidence', 'Capture Before/After'], keywords: ['capture', 'evidence', 'render'],
        inputSchema: captureInput, outputSchema: captureOutput, operationClass: 'render_evidence', stateImpact: captureRender, runtimeRequirements: [runtime('read_render_diffs'), { kind: 'render_surface' }, { kind: 'artifact_writer' }], artifacts: [artifact('render_before_after', 'Before/after visual evidence artifact.')], typedInputExample: { sessionId: 'session-1', beforeArtifactId: 'artifact-before', afterArtifactId: 'artifact-after' }, typedOutputExample: { artifactId: 'artifact-before-after', renderBeforeHash: null, renderAfterHash: null }, panel: 'evidence', dialog: 'simple_form', agentExposure: { kind: 'render_evidence' }, retry: 'retry_after_status_readback', idempotency: { kind: 'conditional', condition: 'Safe after reading current render/artifact status.' }, knownLimitations: ['Render screenshots are evidence only and never authority.'],
    }),
    base({
        id: 'export.agent_readout', label: 'Export Agent Readout', summary: 'Export command timeline, compatibility, diagnostics, and artifact refs for review.', category: 'export', menuPath: ['Export', 'Agent Readout'], keywords: ['export', 'agent', 'review'],
        inputSchema: exportInput, outputSchema: exportOutput, operationClass: 'diagnostic_export', stateImpact: writeWorkspace, runtimeRequirements: [{ kind: 'artifact_writer' }], artifacts: [artifact('agent_readout', 'Human/agent review artifact index.')], typedInputExample: { sessionId: 'session-1', includeVisualEvidence: true }, typedOutputExample: { artifactId: 'agent-readout', commandCount: 0 }, panel: 'export', dialog: 'simple_form', agentExposure: { kind: 'diagnostic_export' }, retry: 'safe_to_retry', idempotency: { kind: 'idempotent', keyFields: ['sessionId', 'includeVisualEvidence'] }, compatibility: REGISTRY_COMPAT,
    }),
];
export const COMMAND_IDS = COMMAND_MANIFEST.map((command) => command.id);
//# sourceMappingURL=manifest.js.map