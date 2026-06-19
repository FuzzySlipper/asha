const EMPTY_INPUT = { name: 'EmptyInput', version: 1, shape: { kind: 'empty' } };
const EMPTY_OUTPUT = { name: 'EmptyOutput', version: 1, shape: { kind: 'object', allowExtraFields: false, fields: [{ name: 'kind', required: true, summary: 'Acknowledgement literal.', shape: { kind: 'literal', values: ['ok'] } }] } };
const SESSION_ID_FIELD = { name: 'sessionId', required: true, summary: 'Stable studio session identifier.', shape: { kind: 'scalar', scalar: 'string' } };
const SCENARIO_ID_FIELD = { name: 'scenarioId', required: true, summary: 'Named public studio scenario identifier.', shape: { kind: 'scalar', scalar: 'string' } };
const VOXEL_COORD_SCHEMA = { kind: 'contract', ref: { package: '@asha/contracts', exportName: 'VoxelCoord' } };
const PICK_RAY_SCHEMA = { kind: 'contract', ref: { package: '@asha/contracts', exportName: 'PickRay' } };
const VOXEL_COMMAND_SCHEMA = { kind: 'contract', ref: { package: '@asha/contracts', exportName: 'VoxelCommand' } };
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
function artifact(type, summary, required = true) {
    return { type, required, producedWhen: required ? 'always' : 'when_available', summary };
}
function runtime(operation) {
    return { kind: 'runtime_bridge_operation', operation };
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
const scenarioIdInput = {
    name: 'ScenarioIdInput',
    version: 1,
    shape: { kind: 'object', allowExtraFields: false, fields: [SCENARIO_ID_FIELD] },
};
const sessionIdInput = {
    name: 'SessionIdInput',
    version: 1,
    shape: { kind: 'object', allowExtraFields: false, fields: [SESSION_ID_FIELD] },
};
const screenPointInput = {
    name: 'ScreenPointInput',
    version: 1,
    shape: { kind: 'object', allowExtraFields: false, fields: [SESSION_ID_FIELD, { name: 'ray', required: true, summary: 'Public contract pick ray.', shape: PICK_RAY_SCHEMA }] },
};
const voxelInspectionInput = {
    name: 'VoxelInspectionInput',
    version: 1,
    shape: { kind: 'object', allowExtraFields: false, fields: [SESSION_ID_FIELD, { name: 'voxel', required: true, summary: 'Voxel coordinate to inspect.', shape: VOXEL_COORD_SCHEMA }] },
};
const previewInput = {
    name: 'VoxelBrushPreviewInput',
    version: 1,
    shape: {
        kind: 'object',
        allowExtraFields: false,
        fields: [
            SESSION_ID_FIELD,
            { name: 'anchor', required: true, summary: 'Preview anchor coordinate.', shape: VOXEL_COORD_SCHEMA },
            { name: 'commands', required: true, summary: 'Typed voxel command preview set.', shape: { kind: 'array', minItems: 1, items: VOXEL_COMMAND_SCHEMA } },
        ],
    },
};
const applyInput = {
    name: 'ApplyVoxelBrushInput',
    version: 1,
    shape: {
        kind: 'object',
        allowExtraFields: false,
        fields: [
            SESSION_ID_FIELD,
            { name: 'commands', required: true, summary: 'Typed authority voxel commands.', shape: { kind: 'array', minItems: 1, items: VOXEL_COMMAND_SCHEMA } },
            { name: 'expectedStateHash', required: true, summary: 'Expected authority state hash or null in typed execution adapter.', shape: { kind: 'scalar', scalar: 'state_hash' } },
        ],
    },
};
const captureInput = {
    name: 'CaptureBeforeAfterInput',
    version: 1,
    shape: {
        kind: 'object',
        allowExtraFields: false,
        fields: [
            SESSION_ID_FIELD,
            { name: 'beforeArtifactId', required: true, summary: 'Before visual evidence artifact id.', shape: { kind: 'scalar', scalar: 'artifact_ref' } },
            { name: 'afterArtifactId', required: true, summary: 'After visual evidence artifact id.', shape: { kind: 'scalar', scalar: 'artifact_ref' } },
        ],
    },
};
const exportInput = {
    name: 'ExportAgentReadoutInput',
    version: 1,
    shape: {
        kind: 'object',
        allowExtraFields: false,
        fields: [
            SESSION_ID_FIELD,
            { name: 'includeVisualEvidence', required: true, summary: 'Whether exported readout references visual artifacts.', shape: { kind: 'scalar', scalar: 'boolean' } },
        ],
    },
};
const simpleOutput = (name, artifactType) => ({
    name,
    version: 1,
    shape: { kind: 'artifactRef', artifactType },
});
export const COMMAND_MANIFEST = [
    base({
        id: 'session.list_scenarios', label: 'List Studio Scenarios', summary: 'List named public scenarios available to a studio session.', category: 'session', menuPath: ['Session', 'List Scenarios'], keywords: ['scenario', 'list'],
        inputSchema: EMPTY_INPUT, outputSchema: simpleOutput('ScenarioListOutput', 'scenario_manifest'), operationClass: 'read_only', stateImpact: noStateImpact, compatibility: REGISTRY_COMPAT, runtimeRequirements: [{ kind: 'none' }], artifacts: [artifact('scenario_manifest', 'Bounded scenario list for session loading.')], typedInputExample: { kind: 'empty' }, typedOutputExample: { scenarios: [{ id: 'voxel-basic', label: 'Basic Voxel Scenario' }] }, panel: 'inspector', dialog: 'readout_only', idempotency: { kind: 'idempotent', keyFields: [] },
    }),
    base({
        id: 'session.start', label: 'Start Studio Session', summary: 'Create/reset a studio session around a named scenario.', category: 'session', menuPath: ['Session', 'Start'], keywords: ['session', 'start'],
        inputSchema: scenarioIdInput, outputSchema: EMPTY_OUTPUT, operationClass: 'workspace_io', stateImpact: { authority: 'read', editor: 'mutate', render: 'none', workspace: 'read' }, runtimeRequirements: [runtime('initialize_engine'), runtime('load_world_bundle')], artifacts: [artifact('session_status', 'Initial session status and compatibility readback.')], typedInputExample: { scenarioId: 'voxel-basic' }, typedOutputExample: { kind: 'ok' }, panel: 'timeline', dialog: 'simple_form', retry: 'retry_after_status_readback', idempotency: { kind: 'conditional', condition: 'Idempotent when scenarioId and session reset token match.' },
    }),
    base({
        id: 'session.load_scenario', label: 'Load Scenario', summary: 'Load a named scenario into the active studio session.', category: 'session', menuPath: ['Session', 'Load Scenario'], keywords: ['load', 'scenario'],
        inputSchema: scenarioIdInput, outputSchema: EMPTY_OUTPUT, operationClass: 'workspace_io', stateImpact: { authority: 'read', editor: 'mutate', render: 'none', workspace: 'read' }, runtimeRequirements: [runtime('load_world_bundle')], artifacts: [artifact('session_status', 'Scenario load status and diagnostics.')], typedInputExample: { scenarioId: 'voxel-basic' }, typedOutputExample: { kind: 'ok' }, panel: 'timeline', dialog: 'simple_form', retry: 'retry_after_status_readback', idempotency: { kind: 'conditional', condition: 'Safe when current session already targets the same scenario id.' },
    }),
    base({
        id: 'inspection.session_status', label: 'Inspect Session Status', summary: 'Read studio/runtime readiness, compatibility, and degradation status.', category: 'inspection', menuPath: ['Inspect', 'Session Status'], keywords: ['status', 'compatibility'],
        inputSchema: sessionIdInput, outputSchema: simpleOutput('SessionStatusOutput', 'session_status'), operationClass: 'read_only', stateImpact: noStateImpact, runtimeRequirements: [runtime('get_composition_status')], artifacts: [artifact('session_status', 'Status readback for the active session.')], typedInputExample: { sessionId: 'session-1' }, typedOutputExample: { sessionId: 'session-1', status: 'ready' }, panel: 'diagnostics', dialog: 'readout_only',
    }),
    base({
        id: 'inspection.world_summary', label: 'Inspect World Summary', summary: 'Read compact public world and authority evidence.', category: 'inspection', menuPath: ['Inspect', 'World Summary'], keywords: ['world', 'hash'],
        inputSchema: sessionIdInput, outputSchema: simpleOutput('WorldSummaryOutput', 'world_summary'), operationClass: 'read_only', stateImpact: readAuthority, runtimeRequirements: [runtime('read_voxel_mesh_evidence')], artifacts: [artifact('world_summary', 'Compact authority/render-neutral world summary.')], typedInputExample: { sessionId: 'session-1' }, typedOutputExample: { authorityHash: null, voxelVolumeCount: 1, sceneNodeCount: 1 }, panel: 'inspector', dialog: 'readout_only',
    }),
    base({
        id: 'inspection.editor_state', label: 'Inspect Editor State', summary: 'Read command-registry/editor-local selection and preview state.', category: 'inspection', menuPath: ['Inspect', 'Editor State'], keywords: ['editor', 'selection'],
        inputSchema: sessionIdInput, outputSchema: simpleOutput('EditorStateOutput', 'editor_state'), operationClass: 'read_only', stateImpact: readEditor, runtimeRequirements: [{ kind: 'editor_store' }], artifacts: [artifact('editor_state', 'Editor-local state snapshot.')], typedInputExample: { sessionId: 'session-1' }, typedOutputExample: { editorVersion: 'editor.v0', selectedVoxel: null }, panel: 'inspector', dialog: 'readout_only', compatibility: REGISTRY_COMPAT,
    }),
    base({
        id: 'selection.voxel_from_screen_point', label: 'Select Voxel From Screen Point', summary: 'Project a screen pick into a typed ASHA voxel hit and editor selection.', category: 'selection', menuPath: ['Select', 'Voxel From Screen Point'], keywords: ['pick', 'select', 'voxel'],
        inputSchema: screenPointInput, outputSchema: simpleOutput('VoxelSelectionOutput', 'selection_snapshot'), operationClass: 'editor_local', stateImpact: mutateEditor, runtimeRequirements: [runtime('pick_voxel'), runtime('select_voxel'), { kind: 'editor_store' }], artifacts: [artifact('selection_snapshot', 'Selected voxel hit or no-hit result.')], typedInputExample: { sessionId: 'session-1', ray: { grid: 0, origin: [0, 0, 0], direction: [1, 0, 0], maxDistance: 128 } }, typedOutputExample: { hit: null }, panel: 'viewport', dialog: 'none', inputContractRefs: [{ package: '@asha/contracts', exportName: 'PickRay' }], outputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelHit' }], agentExposure: { kind: 'editor_local' }, undo: { kind: 'editor_local', inverseData: ['previous selection snapshot'] }, idempotency: { kind: 'conditional', condition: 'Same pick ray and unchanged view state selects the same voxel.' },
    }),
    base({
        id: 'inspection.voxel', label: 'Inspect Voxel', summary: 'Read typed voxel/material state for one public coordinate.', category: 'inspection', menuPath: ['Inspect', 'Voxel'], keywords: ['voxel', 'inspect'],
        inputSchema: voxelInspectionInput, outputSchema: simpleOutput('VoxelInspectionOutput', 'voxel_inspection'), operationClass: 'read_only', stateImpact: readAuthority, runtimeRequirements: [runtime('read_voxel_mesh_evidence')], artifacts: [artifact('voxel_inspection', 'Voxel occupancy/material readout.')], typedInputExample: { sessionId: 'session-1', voxel: { x: 0, y: 0, z: 0 } }, typedOutputExample: { voxel: { x: 0, y: 0, z: 0 }, materialId: null, occupied: false }, panel: 'inspector', dialog: 'readout_only', inputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCoord' }], outputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCoord' }],
    }),
    base({
        id: 'preview.voxel_brush', label: 'Preview Voxel Brush', summary: 'Preview a typed voxel edit without mutating authority.', category: 'preview', menuPath: ['Edit', 'Preview Voxel Brush'], keywords: ['preview', 'brush', 'voxel'],
        inputSchema: previewInput, outputSchema: simpleOutput('VoxelBrushPreviewOutput', 'voxel_preview'), operationClass: 'editor_local', stateImpact: mutateEditor, runtimeRequirements: [{ kind: 'editor_store' }], artifacts: [artifact('voxel_preview', 'Editor-local target voxel preview.')], typedInputExample: { sessionId: 'session-1', anchor: { x: 0, y: 0, z: 0 }, commands: [{ op: 'setVoxel', grid: 0, coord: { x: 0, y: 0, z: 0 }, value: { kind: 'solid', material: 1 } }] }, typedOutputExample: { targetVoxels: [{ x: 0, y: 0, z: 0 }], previewVersion: 'preview.v0' }, panel: 'viewport', dialog: 'simple_form', inputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCoord' }, { package: '@asha/contracts', exportName: 'VoxelCommand' }], outputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCoord' }], agentExposure: { kind: 'editor_local' }, undo: { kind: 'editor_local', inverseData: ['previous preview snapshot'] }, idempotency: { kind: 'idempotent', keyFields: ['sessionId', 'anchor', 'commands'] }, compatibility: REGISTRY_COMPAT,
    }),
    base({
        id: 'authority.voxel.apply_brush', label: 'Apply Voxel Brush', summary: 'Apply typed voxel commands through ASHA authority validation.', category: 'authority_edit', menuPath: ['Edit', 'Apply Voxel Brush'], keywords: ['apply', 'voxel', 'authority'],
        inputSchema: applyInput, outputSchema: simpleOutput('ApplyVoxelBrushOutput', 'command_result'), operationClass: 'authority_mutating', stateImpact: mutateAuthority, runtimeRequirements: [runtime('submit_commands'), runtime('read_voxel_mesh_evidence')], artifacts: [artifact('command_result', 'Accepted/rejected authority command result with state hash evidence.')], typedInputExample: { sessionId: 'session-1', commands: [{ op: 'setVoxel', grid: 0, coord: { x: 0, y: 0, z: 0 }, value: { kind: 'solid', material: 1 } }], expectedStateHash: null }, typedOutputExample: { accepted: true, authorityBeforeHash: null, authorityAfterHash: null }, panel: 'timeline', dialog: 'advanced_form', inputContractRefs: [{ package: '@asha/contracts', exportName: 'VoxelCommand' }], agentExposure: { kind: 'authority_mutating', requiresPreview: true, batchable: true }, undo: { kind: 'authority_reversing', inverseCommandRefs: [], requiresSameStateHash: true }, retry: 'safe_to_retry_if_state_hash_unchanged', idempotency: { kind: 'conditional', condition: 'Safe when expectedStateHash still matches and command sequence id has not committed.' }, knownLimitations: ['V1 records reversal posture but does not declare a generic authority undo stack.'],
    }),
    base({
        id: 'inspection.last_command_result', label: 'Inspect Last Command Result', summary: 'Read the last timeline command result for human/agent correlation.', category: 'inspection', menuPath: ['Inspect', 'Last Command Result'], keywords: ['timeline', 'result'],
        inputSchema: sessionIdInput, outputSchema: simpleOutput('LastCommandResultOutput', 'command_result'), operationClass: 'read_only', stateImpact: readEditor, runtimeRequirements: [{ kind: 'editor_store' }], artifacts: [artifact('command_result', 'Last known command result reference.', false)], typedInputExample: { sessionId: 'session-1' }, typedOutputExample: { sequenceId: null, status: null }, panel: 'timeline', dialog: 'readout_only', compatibility: REGISTRY_COMPAT,
    }),
    base({
        id: 'render.capture_before_after', label: 'Capture Before/After Evidence', summary: 'Capture/render before-after evidence as non-authoritative artifacts.', category: 'render_evidence', menuPath: ['Evidence', 'Capture Before/After'], keywords: ['capture', 'evidence', 'render'],
        inputSchema: captureInput, outputSchema: simpleOutput('CaptureBeforeAfterOutput', 'render_before_after'), operationClass: 'render_evidence', stateImpact: captureRender, runtimeRequirements: [runtime('read_render_diffs'), { kind: 'render_surface' }, { kind: 'artifact_writer' }], artifacts: [artifact('render_before_after', 'Before/after visual evidence artifact.')], typedInputExample: { sessionId: 'session-1', beforeArtifactId: 'artifact-before', afterArtifactId: 'artifact-after' }, typedOutputExample: { artifactId: 'artifact-before-after', renderBeforeHash: null, renderAfterHash: null }, panel: 'evidence', dialog: 'simple_form', agentExposure: { kind: 'render_evidence' }, retry: 'retry_after_status_readback', idempotency: { kind: 'conditional', condition: 'Safe after reading current render/artifact status.' }, knownLimitations: ['Render screenshots are evidence only and never authority.'],
    }),
    base({
        id: 'export.agent_readout', label: 'Export Agent Readout', summary: 'Export command timeline, compatibility, diagnostics, and artifact refs for review.', category: 'export', menuPath: ['Export', 'Agent Readout'], keywords: ['export', 'agent', 'review'],
        inputSchema: exportInput, outputSchema: simpleOutput('ExportAgentReadoutOutput', 'agent_readout'), operationClass: 'diagnostic_export', stateImpact: writeWorkspace, runtimeRequirements: [{ kind: 'artifact_writer' }], artifacts: [artifact('agent_readout', 'Human/agent review artifact index.')], typedInputExample: { sessionId: 'session-1', includeVisualEvidence: true }, typedOutputExample: { artifactId: 'agent-readout', commandCount: 0 }, panel: 'export', dialog: 'simple_form', agentExposure: { kind: 'diagnostic_export' }, retry: 'safe_to_retry', idempotency: { kind: 'idempotent', keyFields: ['sessionId', 'includeVisualEvidence'] }, compatibility: REGISTRY_COMPAT,
    }),
];
export const COMMAND_IDS = COMMAND_MANIFEST.map((command) => command.id);
//# sourceMappingURL=manifest.js.map