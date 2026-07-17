// Backend-neutral retained editor/product viewport.

import type {
  CameraBasis,
  CameraPose,
  EditorGridDescriptor,
  EditorGridProjectionReadout,
  PerspectiveProjection,
  RenderDiff,
  RenderFrameDiff,
  RenderHandle,
  RenderLayer,
  TagId,
} from '@asha/contracts';
import { renderHandle } from '@asha/contracts';
import { RenderProjection, type RenderProjectionSnapshot } from '@asha/render-projection';
import { mountAshaRendererEditorBackend } from '@asha/renderer-three/backend';
import {
  loadRendererAnimatedMeshSource,
  type AshaRendererAnimatedMeshResourceManifest,
  type AshaRendererAnimatedMeshResourceResolver,
} from './animated-mesh-host.js';

export const ASHA_RENDERER_EDITOR_VIEWPORT_COMPATIBILITY_VERSION = 'editor-viewport.v0';
export const ASHA_RENDERER_EDITOR_VIEWPORT_MAX_FRAME_OPS = 4096;
export const ASHA_RENDERER_EDITOR_VIEWPORT_MAX_RETAINED_OPS = 8192;

export type AshaRendererEditorViewportChannel = 'runtime' | 'authored' | 'overlay';
export type AshaRendererEditorViewportStatus = 'mounted' | 'running' | 'stopped' | 'disposed';
export type AshaRendererEditorViewportCameraSource = 'stored_editor' | 'runtime_authority';

export interface AshaRendererEditorViewportChannelPolicy {
  readonly channel: AshaRendererEditorViewportChannel;
  readonly order: number;
  readonly allowedLayers: readonly RenderLayer[];
  readonly depthPolicy: 'shared_scene_depth' | 'overlay_after_depth_clear';
}

export const ASHA_RENDERER_EDITOR_VIEWPORT_CHANNEL_POLICIES: readonly AshaRendererEditorViewportChannelPolicy[] = [
  {
    channel: 'runtime',
    order: 0,
    allowedLayers: ['scene', 'debug'],
    depthPolicy: 'shared_scene_depth',
  },
  {
    channel: 'authored',
    order: 1,
    allowedLayers: ['scene', 'debug'],
    depthPolicy: 'shared_scene_depth',
  },
  {
    channel: 'overlay',
    order: 2,
    allowedLayers: ['debug'],
    depthPolicy: 'overlay_after_depth_clear',
  },
] as const;

export interface AshaRendererEditorViewportCamera {
  readonly source: AshaRendererEditorViewportCameraSource;
  readonly pose: CameraPose;
  readonly basis: CameraBasis;
  readonly projection: PerspectiveProjection;
}

export interface AshaRendererEditorViewportSize {
  readonly width: number;
  readonly height: number;
  readonly pixelRatio: number;
}

export interface AshaRendererEditorViewportBufferSource {
  readonly borrow: (handle: number) => Uint8Array;
  readonly release: (handle: number) => void;
}

export interface AshaRendererEditorViewportOptions {
  readonly animatedMeshManifest?: AshaRendererAnimatedMeshResourceManifest;
  readonly autoStart?: boolean;
  readonly bufferSource?: AshaRendererEditorViewportBufferSource;
  readonly clearColor?: number;
  readonly initialCamera?: AshaRendererEditorViewportCamera;
  readonly initialGrid?: EditorGridDescriptor | null;
  readonly pixelRatio?: number;
  readonly resolveAnimatedMeshResource?: AshaRendererAnimatedMeshResourceResolver;
}

export type AshaRendererEditorViewportDiagnosticCode =
  | 'backend_rejected'
  | 'channel_disposed'
  | 'frame_limit_exceeded'
  | 'invalid_camera'
  | 'invalid_frame'
  | 'invalid_grid'
  | 'invalid_handle'
  | 'invalid_pick_request'
  | 'invalid_viewport_size'
  | 'overlay_requires_debug_layer'
  | 'viewport_disposed';

export interface AshaRendererEditorViewportDiagnostic {
  readonly channel: AshaRendererEditorViewportChannel | null;
  readonly code: AshaRendererEditorViewportDiagnosticCode;
  readonly message: string;
  readonly recoverable: boolean;
}

export interface AshaRendererEditorViewportChannelSnapshot {
  readonly channel: AshaRendererEditorViewportChannel;
  readonly disposed: boolean;
  readonly generation: number;
  readonly hash: string;
  readonly retainedOpCount: number;
  readonly projection: RenderProjectionSnapshot;
}

export interface AshaRendererEditorViewportChannelReceipt {
  readonly applied: boolean;
  readonly channel: AshaRendererEditorViewportChannel;
  readonly diagnostics: readonly AshaRendererEditorViewportDiagnostic[];
  readonly generation: number;
  readonly snapshotHash: string;
}

export interface AshaRendererEditorViewportChannelHandle {
  readonly channel: AshaRendererEditorViewportChannel;
  readonly apply: (frame: RenderFrameDiff) => AshaRendererEditorViewportChannelReceipt;
  readonly clear: () => AshaRendererEditorViewportChannelReceipt;
  readonly dispose: () => AshaRendererEditorViewportChannelReceipt;
  readonly replace: (frame: RenderFrameDiff) => AshaRendererEditorViewportChannelReceipt;
  readonly snapshot: () => AshaRendererEditorViewportChannelSnapshot;
}

export interface AshaRendererEditorViewportPickFilter {
  readonly channels?: readonly AshaRendererEditorViewportChannel[];
  readonly handles?: readonly RenderHandle[];
  readonly layers?: readonly RenderLayer[];
  readonly tags?: readonly TagId[];
}

export interface AshaRendererEditorViewportPickRequest {
  /** Canvas-relative pixels from the top-left corner. */
  readonly point: readonly [number, number];
  readonly filter?: AshaRendererEditorViewportPickFilter;
  readonly maxDistance?: number;
}

export interface AshaRendererEditorViewportPickHint {
  readonly channel: AshaRendererEditorViewportChannel;
  readonly distance: number;
  readonly handle: RenderHandle;
  readonly label: string | null;
  readonly layer: RenderLayer;
  readonly normal: readonly [number, number, number];
  readonly position: readonly [number, number, number];
  readonly sourceTrace: {
    readonly entity: import('@asha/contracts').EntityId;
    readonly kind: 'render_metadata_entity';
  } | null;
  readonly tags: readonly TagId[];
}

export interface AshaRendererEditorViewportPickReceipt {
  readonly diagnostics: readonly AshaRendererEditorViewportDiagnostic[];
  readonly hint: AshaRendererEditorViewportPickHint | null;
  readonly kind: 'asha_renderer_editor_viewport_pick.v0';
}

export interface AshaRendererEditorViewportCameraReceipt {
  readonly applied: boolean;
  readonly diagnostics: readonly AshaRendererEditorViewportDiagnostic[];
  readonly hash: string;
}

export interface AshaRendererEditorViewportSizeReceipt {
  readonly applied: boolean;
  readonly diagnostics: readonly AshaRendererEditorViewportDiagnostic[];
  readonly size: AshaRendererEditorViewportSize;
}

export interface AshaRendererEditorViewportReadout {
  readonly kind: 'asha_renderer_editor_viewport_readout.v0';
  readonly compatibilityVersion: typeof ASHA_RENDERER_EDITOR_VIEWPORT_COMPATIBILITY_VERSION;
  readonly status: AshaRendererEditorViewportStatus;
  readonly camera: AshaRendererEditorViewportCamera;
  readonly size: AshaRendererEditorViewportSize;
  readonly channels: readonly AshaRendererEditorViewportChannelSnapshot[];
  readonly channelPolicies: readonly AshaRendererEditorViewportChannelPolicy[];
  readonly diagnostics: readonly AshaRendererEditorViewportDiagnostic[];
  readonly grid: EditorGridProjectionReadout | null;
  readonly viewportHash: string;
}

export interface AshaRendererEditorViewport {
  readonly kind: 'asha_renderer_editor_viewport.v0';
  readonly channels: Readonly<Record<AshaRendererEditorViewportChannel, AshaRendererEditorViewportChannelHandle>>;
  readonly camera: () => AshaRendererEditorViewportCamera;
  readonly dispose: () => void;
  readonly grid: () => EditorGridProjectionReadout | null;
  readonly pick: (request: AshaRendererEditorViewportPickRequest) => AshaRendererEditorViewportPickReceipt;
  readonly readout: () => AshaRendererEditorViewportReadout;
  readonly renderOnce: (timeMs?: number) => void;
  readonly resize: (size: AshaRendererEditorViewportSize) => AshaRendererEditorViewportSizeReceipt;
  readonly setCamera: (camera: AshaRendererEditorViewportCamera) => AshaRendererEditorViewportCameraReceipt;
  readonly setGrid: (descriptor: EditorGridDescriptor | null) => AshaRendererEditorViewportGridReceipt;
  readonly start: () => void;
  readonly stop: () => void;
}

export interface AshaRendererEditorViewportGridReceipt {
  readonly applied: boolean;
  readonly diagnostics: readonly AshaRendererEditorViewportDiagnostic[];
  readonly grid: EditorGridProjectionReadout | null;
  readonly hash: string;
}

interface ChannelState {
  readonly channel: AshaRendererEditorViewportChannel;
  disposed: boolean;
  generation: number;
  history: readonly RenderDiff[];
  projection: RenderProjection;
}

export interface AshaRendererEditorViewportBackendPort {
  readonly dispose: () => void;
  readonly gridReadout: () => EditorGridProjectionReadout | null;
  readonly pick: (request: BackendPickRequest) => BackendPickReceipt;
  readonly renderOnce: (timeMs?: number) => void;
  readonly replaceChannel: (channel: AshaRendererEditorViewportChannel, frame: RenderFrameDiff) => void;
  readonly resize: (size: AshaRendererEditorViewportSize) => void;
  readonly setCamera: (camera: Omit<AshaRendererEditorViewportCamera, 'source'>) => void;
  readonly setGrid: (descriptor: EditorGridDescriptor | null) => void;
  readonly snapshot: () => string;
  readonly start: () => void;
  readonly stop: () => void;
}

interface BackendPickRequest {
  readonly filter?: {
    readonly channels?: readonly AshaRendererEditorViewportChannel[];
    readonly handles?: readonly RenderHandle[];
    readonly layers?: readonly RenderLayer[];
    readonly tags?: readonly TagId[];
  };
  readonly maxDistance?: number;
  readonly point: readonly [number, number];
}

interface BackendPickReceipt {
  readonly diagnostics: readonly { readonly code: string; readonly message: string }[];
  readonly hit: {
    readonly channel: AshaRendererEditorViewportChannel;
    readonly distance: number;
    readonly handle: RenderHandle;
    readonly label: string | null;
    readonly layer: RenderLayer;
    readonly normal: readonly [number, number, number];
    readonly position: readonly [number, number, number];
    readonly sourceTrace: AshaRendererEditorViewportPickHint['sourceTrace'];
    readonly tags: readonly TagId[];
  } | null;
}

const CHANNELS: readonly AshaRendererEditorViewportChannel[] = ['runtime', 'authored', 'overlay'];
const CHANNEL_HANDLE_OFFSETS: Readonly<Record<AshaRendererEditorViewportChannel, number>> = {
  runtime: 1_000_000_000_000,
  authored: 2_000_000_000_000,
  overlay: 3_000_000_000_000,
};
const MAX_LOGICAL_HANDLE = 999_999_999_999;
const MAX_PICK_FILTER_VALUES = 128;
const MAX_VIEWPORT_DIMENSION = 16_384;
const MAX_DIAGNOSTICS = 64;

export async function mountAshaRendererEditorViewport(
  canvas: HTMLCanvasElement,
  options: AshaRendererEditorViewportOptions = {},
): Promise<AshaRendererEditorViewport> {
  const animatedMeshSource = options.animatedMeshManifest === undefined
    ? undefined
    : await loadRendererAnimatedMeshSource(
        options.animatedMeshManifest,
        options.resolveAnimatedMeshResource,
      );
  const bufferSource = options.bufferSource;
  const meshBufferSource = bufferSource === undefined
    ? undefined
    : {
        getBuffer: (handle: number) => ({
          handle: handle as never,
          bytes: bufferSource.borrow(handle),
        }),
        releaseBuffer: (handle: number) => bufferSource.release(handle),
      };
  const backend = mountAshaRendererEditorBackend(canvas, {
    ...(animatedMeshSource === undefined ? {} : { animatedMeshSource: animatedMeshSource as never }),
    ...(meshBufferSource === undefined ? {} : { meshBufferSource: meshBufferSource as never }),
    ...(options.clearColor === undefined ? {} : { clearColor: options.clearColor }),
    ...(options.pixelRatio === undefined ? {} : { pixelRatio: options.pixelRatio }),
  });
  const size = {
    width: Math.max(1, canvas.clientWidth || canvas.width || 800),
    height: Math.max(1, canvas.clientHeight || canvas.height || 450),
    pixelRatio: options.pixelRatio ?? globalThis.devicePixelRatio ?? 1,
  };
  return createAshaRendererEditorViewportWithBackend(backend, {
    ...(options.autoStart === undefined ? {} : { autoStart: options.autoStart }),
    ...(options.initialCamera === undefined ? {} : { initialCamera: options.initialCamera }),
    ...(options.initialGrid === undefined ? {} : { initialGrid: options.initialGrid }),
    size,
  });
}

/** Internal conformance seam; not exported from the package root. */
export function createAshaRendererEditorViewportWithBackend(
  backend: AshaRendererEditorViewportBackendPort,
  options: {
    readonly autoStart?: boolean;
    readonly initialCamera?: AshaRendererEditorViewportCamera;
    readonly initialGrid?: EditorGridDescriptor | null;
    readonly size?: AshaRendererEditorViewportSize;
  } = {},
): AshaRendererEditorViewport {
  const states = new Map<AshaRendererEditorViewportChannel, ChannelState>();
  for (const channel of CHANNELS) {
    states.set(channel, {
      channel,
      disposed: false,
      generation: 0,
      history: [],
      projection: new RenderProjection(),
    });
  }
  const diagnostics: AshaRendererEditorViewportDiagnostic[] = [];
  let status: AshaRendererEditorViewportStatus = 'mounted';
  const requestedCamera = options.initialCamera ?? defaultEditorCamera();
  const cameraIssue = validateCamera(requestedCamera);
  let camera = cameraIssue === null ? requestedCamera : defaultEditorCamera();
  if (cameraIssue !== null) {
    rememberDiagnostic(diagnostics, cameraIssue);
  }
  const requestedSize = options.size ?? { width: 800, height: 450, pixelRatio: 1 };
  const sizeIssue = validateSize(requestedSize);
  let size = sizeIssue === null ? requestedSize : { width: 800, height: 450, pixelRatio: 1 };
  if (sizeIssue !== null) {
    rememberDiagnostic(diagnostics, sizeIssue);
  }
  const requestedGrid = options.initialGrid ?? null;
  const gridIssue = validateGrid(requestedGrid);
  const initialGridDescriptor = gridIssue === null ? cloneGrid(requestedGrid) : null;
  if (gridIssue !== null) {
    rememberDiagnostic(diagnostics, gridIssue);
  }

  backend.resize(size);
  backend.setCamera(camera);
  backend.setGrid(initialGridDescriptor);

  const channelHandles = Object.fromEntries(CHANNELS.map((channel) => [
    channel,
    createChannelHandle(channel, () => status, states, backend, diagnostics),
  ])) as Record<AshaRendererEditorViewportChannel, AshaRendererEditorViewportChannelHandle>;

  const readout = (): AshaRendererEditorViewportReadout => {
    const channelSnapshots = CHANNELS.map((channel) => snapshotChannel(requireState(states, channel)));
    const grid = backend.gridReadout();
    const viewportHash = stableHash({
      camera,
      channels: channelSnapshots.map(({ channel, disposed, generation, hash }) => ({
        channel,
        disposed,
        generation,
        hash,
      })),
      size,
      status,
      grid,
    });
    return {
      kind: 'asha_renderer_editor_viewport_readout.v0',
      compatibilityVersion: ASHA_RENDERER_EDITOR_VIEWPORT_COMPATIBILITY_VERSION,
      status,
      camera,
      size,
      channels: channelSnapshots,
      channelPolicies: ASHA_RENDERER_EDITOR_VIEWPORT_CHANNEL_POLICIES,
      diagnostics: [...diagnostics],
      grid,
      viewportHash,
    };
  };

  const viewport: AshaRendererEditorViewport = {
    kind: 'asha_renderer_editor_viewport.v0',
    channels: channelHandles,
    camera: () => camera,
    grid: () => backend.gridReadout(),
    setCamera: (next) => {
      const issue = validateCamera(next);
      if (issue !== null || status === 'disposed') {
        const diagnostic = issue ?? viewportDisposedDiagnostic(null);
        rememberDiagnostic(diagnostics, diagnostic);
        return { applied: false, diagnostics: [diagnostic], hash: stableHash(camera) };
      }
      try {
        backend.setCamera(next);
        camera = next;
        return { applied: true, diagnostics: [], hash: stableHash(camera) };
      } catch (error) {
        const diagnostic = backendDiagnostic(null, error);
        rememberDiagnostic(diagnostics, diagnostic);
        return { applied: false, diagnostics: [diagnostic], hash: stableHash(camera) };
      }
    },
    resize: (next) => {
      const issue = validateSize(next);
      if (issue !== null || status === 'disposed') {
        const diagnostic = issue ?? viewportDisposedDiagnostic(null);
        rememberDiagnostic(diagnostics, diagnostic);
        return { applied: false, diagnostics: [diagnostic], size };
      }
      try {
        backend.resize(next);
        size = next;
        return { applied: true, diagnostics: [], size };
      } catch (error) {
        const diagnostic = backendDiagnostic(null, error);
        rememberDiagnostic(diagnostics, diagnostic);
        return { applied: false, diagnostics: [diagnostic], size };
      }
    },
    setGrid: (next) => {
      const issue = validateGrid(next);
      if (issue !== null || status === 'disposed') {
        const diagnostic = issue ?? viewportDisposedDiagnostic(null);
        rememberDiagnostic(diagnostics, diagnostic);
        const grid = backend.gridReadout();
        return { applied: false, diagnostics: [diagnostic], grid, hash: stableHash(grid) };
      }
      try {
        backend.setGrid(next);
        const grid = backend.gridReadout();
        return { applied: true, diagnostics: [], grid, hash: stableHash(grid) };
      } catch (error) {
        const diagnostic = backendDiagnostic(null, error);
        rememberDiagnostic(diagnostics, diagnostic);
        const grid = backend.gridReadout();
        return { applied: false, diagnostics: [diagnostic], grid, hash: stableHash(grid) };
      }
    },
    pick: (request) => pickViewport(status, size, states, backend, diagnostics, request),
    readout,
    renderOnce: (timeMs) => {
      if (status !== 'disposed') {
        backend.renderOnce(timeMs);
      }
    },
    start: () => {
      if (status !== 'disposed' && status !== 'running') {
        backend.start();
        status = 'running';
      }
    },
    stop: () => {
      if (status !== 'disposed' && status !== 'stopped') {
        backend.stop();
        status = 'stopped';
      }
    },
    dispose: () => {
      if (status === 'disposed') {
        return;
      }
      backend.stop();
      backend.dispose();
      status = 'disposed';
      for (const state of states.values()) {
        state.disposed = true;
      }
    },
  };

  if (options.autoStart !== false) {
    viewport.start();
  }
  return viewport;
}

function createChannelHandle(
  channel: AshaRendererEditorViewportChannel,
  viewportStatus: () => AshaRendererEditorViewportStatus,
  states: Map<AshaRendererEditorViewportChannel, ChannelState>,
  backend: AshaRendererEditorViewportBackendPort,
  diagnostics: AshaRendererEditorViewportDiagnostic[],
): AshaRendererEditorViewportChannelHandle {
  const commit = (mode: 'apply' | 'replace' | 'clear'): AshaRendererEditorViewportChannelReceipt => {
    const state = requireState(states, channel);
    if (viewportStatus() === 'disposed') {
      return rejectedChannelReceipt(state, diagnostics, viewportDisposedDiagnostic(channel));
    }
    if (state.disposed) {
      return rejectedChannelReceipt(state, diagnostics, {
        channel,
        code: 'channel_disposed',
        message: `renderer viewport channel ${channel} is disposed`,
        recoverable: false,
      });
    }
    const frame = mode === 'clear' ? { ops: [] as readonly RenderDiff[] } : pendingFrame;
    if (!hasRenderFrameOps(frame)) {
      return rejectedChannelReceipt(
        state,
        diagnostics,
        invalidFrameDiagnostic(channel, 'render frame ops must be an array'),
      );
    }
    const nextHistory = mode === 'apply' ? [...state.history, ...frame.ops] : [...frame.ops];
    const validation = validateChannelHistory(channel, frame, nextHistory);
    if ('diagnostic' in validation) {
      return rejectedChannelReceipt(state, diagnostics, validation.diagnostic);
    }
    try {
      backend.replaceChannel(channel, namespaceFrame(channel, { ops: nextHistory }));
    } catch (error) {
      return rejectedChannelReceipt(state, diagnostics, backendDiagnostic(channel, error));
    }
    state.history = nextHistory;
    state.projection = validation.projection;
    state.generation += 1;
    return acceptedChannelReceipt(state);
  };

  let pendingFrame: RenderFrameDiff = { ops: [] };
  return {
    channel,
    apply: (frame) => {
      pendingFrame = frame;
      return commit('apply');
    },
    replace: (frame) => {
      pendingFrame = frame;
      return commit('replace');
    },
    clear: () => commit('clear'),
    snapshot: () => snapshotChannel(requireState(states, channel)),
    dispose: () => {
      const state = requireState(states, channel);
      if (viewportStatus() === 'disposed' || state.disposed) {
        const diagnostic = viewportStatus() === 'disposed'
          ? viewportDisposedDiagnostic(channel)
          : {
              channel,
              code: 'channel_disposed' as const,
              message: `renderer viewport channel ${channel} is disposed`,
              recoverable: false,
            };
        return rejectedChannelReceipt(state, diagnostics, diagnostic);
      }
      const receipt = commit('clear');
      if (receipt.applied) {
        state.disposed = true;
      }
      return receipt;
    },
  };
}

function validateChannelHistory(
  channel: AshaRendererEditorViewportChannel,
  frame: RenderFrameDiff,
  history: readonly RenderDiff[],
): { readonly projection: RenderProjection } | { readonly diagnostic: AshaRendererEditorViewportDiagnostic } {
  if (frame.ops.length > ASHA_RENDERER_EDITOR_VIEWPORT_MAX_FRAME_OPS
    || history.length > ASHA_RENDERER_EDITOR_VIEWPORT_MAX_RETAINED_OPS) {
    return {
      diagnostic: {
        channel,
        code: 'frame_limit_exceeded',
        message: `renderer viewport frames are bounded to ${ASHA_RENDERER_EDITOR_VIEWPORT_MAX_FRAME_OPS} ops and ${ASHA_RENDERER_EDITOR_VIEWPORT_MAX_RETAINED_OPS} retained ops`,
        recoverable: true,
      },
    };
  }
  for (const op of frame.ops) {
    const handleIssue = validateDiffHandles(channel, op);
    if (handleIssue !== null) {
      return { diagnostic: handleIssue };
    }
    if (channel === 'overlay' && createdLayer(op) !== null && createdLayer(op) !== 'debug') {
      return {
        diagnostic: {
          channel,
          code: 'overlay_requires_debug_layer',
          message: 'overlay channel creates must use the debug render layer',
          recoverable: true,
        },
      };
    }
  }
  try {
    const projection = new RenderProjection();
    projection.applyFrame({ ops: history });
    return { projection };
  } catch (error) {
    return { diagnostic: invalidFrameDiagnostic(channel, error instanceof Error ? error.message : String(error)) };
  }
}

function validateDiffHandles(
  channel: AshaRendererEditorViewportChannel,
  op: RenderDiff,
): AshaRendererEditorViewportDiagnostic | null {
  const values: number[] = [];
  if ('handle' in op) {
    values.push(op.handle);
  }
  if ('parent' in op && op.parent !== null) {
    values.push(op.parent);
  }
  if (values.every((value) => Number.isSafeInteger(value) && value >= 0 && value <= MAX_LOGICAL_HANDLE)) {
    return null;
  }
  return {
    channel,
    code: 'invalid_handle',
    message: `render handles must be canonical integers from 0 through ${MAX_LOGICAL_HANDLE}`,
    recoverable: true,
  };
}

function createdLayer(op: RenderDiff): RenderLayer | null {
  if (op.op === 'create') {
    return op.node.layer;
  }
  if (op.op === 'createStaticMeshInstance'
    || op.op === 'createAnimatedMeshInstance'
    || op.op === 'createSprite'
    || op.op === 'createLight') {
    return 'scene';
  }
  return null;
}

function namespaceFrame(
  channel: AshaRendererEditorViewportChannel,
  frame: RenderFrameDiff,
): RenderFrameDiff {
  return { ops: frame.ops.map((op) => namespaceDiff(channel, op)) };
}

function namespaceDiff(channel: AshaRendererEditorViewportChannel, op: RenderDiff): RenderDiff {
  const handle = (value: RenderHandle): RenderHandle =>
    renderHandle(CHANNEL_HANDLE_OFFSETS[channel] + value);
  const parent = (value: RenderHandle | null): RenderHandle | null => value === null ? null : handle(value);
  switch (op.op) {
    case 'create':
      return { ...op, handle: handle(op.handle), parent: parent(op.parent) };
    case 'createStaticMeshInstance':
    case 'createAnimatedMeshInstance':
    case 'createSprite':
    case 'createLight':
      return { ...op, handle: handle(op.handle), parent: parent(op.parent) };
    case 'update':
    case 'destroy':
    case 'replaceMeshPayload':
    case 'setMaterialInstanceParameters':
    case 'setAnimatedMeshPlayback':
    case 'updateSprite':
    case 'updateLight':
      return { ...op, handle: handle(op.handle) };
    case 'defineMaterial':
    case 'defineTexture':
    case 'defineSpriteAtlas':
    case 'defineStaticMesh':
    case 'defineAnimatedMesh':
      return op;
  }
}

function pickViewport(
  status: AshaRendererEditorViewportStatus,
  size: AshaRendererEditorViewportSize,
  states: ReadonlyMap<AshaRendererEditorViewportChannel, ChannelState>,
  backend: AshaRendererEditorViewportBackendPort,
  diagnostics: AshaRendererEditorViewportDiagnostic[],
  request: AshaRendererEditorViewportPickRequest,
): AshaRendererEditorViewportPickReceipt {
  const issue = validatePickRequest(status, size, request);
  if (issue !== null) {
    rememberDiagnostic(diagnostics, issue);
    return { diagnostics: [issue], hint: null, kind: 'asha_renderer_editor_viewport_pick.v0' };
  }
  const channels = request.filter?.channels ?? CHANNELS;
  const backendHandles = request.filter?.handles === undefined
    ? undefined
    : channels.flatMap((channel) => request.filter?.handles?.map((handle) =>
        renderHandle(CHANNEL_HANDLE_OFFSETS[channel] + handle),
      ) ?? []);
  const normalizedX = (request.point[0] / size.width) * 2 - 1;
  const normalizedY = -((request.point[1] / size.height) * 2 - 1);
  const point: readonly [number, number] = [
    normalizedX === 0 ? 0 : normalizedX,
    normalizedY === 0 ? 0 : normalizedY,
  ];
  try {
    const receipt = backend.pick({
      point,
      ...(request.maxDistance === undefined ? {} : { maxDistance: request.maxDistance }),
      filter: {
        channels,
        ...(backendHandles === undefined ? {} : { handles: backendHandles }),
        ...(request.filter?.layers === undefined ? {} : { layers: request.filter.layers }),
        ...(request.filter?.tags === undefined ? {} : { tags: request.filter.tags }),
      },
    });
    if (receipt.diagnostics.length > 0) {
      const projected = receipt.diagnostics.map((entry) => ({
        channel: null,
        code: 'backend_rejected' as const,
        message: `${entry.code}: ${entry.message}`,
        recoverable: true,
      }));
      projected.forEach((entry) => rememberDiagnostic(diagnostics, entry));
      return { diagnostics: projected, hint: null, kind: 'asha_renderer_editor_viewport_pick.v0' };
    }
    if (receipt.hit === null) {
      return { diagnostics: [], hint: null, kind: 'asha_renderer_editor_viewport_pick.v0' };
    }
    const offset = CHANNEL_HANDLE_OFFSETS[receipt.hit.channel];
    const logicalHandle = receipt.hit.handle - offset;
    const state = states.get(receipt.hit.channel);
    const logicalRenderHandle = renderHandle(logicalHandle);
    const retained = state?.projection.snapshot().nodes.some((node) => node.handle === logicalRenderHandle) ?? false;
    if (state === undefined || state.disposed || !retained || !Number.isSafeInteger(logicalHandle)
      || logicalHandle < 0 || logicalHandle > MAX_LOGICAL_HANDLE) {
      const diagnostic = backendDiagnostic(receipt.hit.channel, 'backend returned an unrecognized namespaced handle');
      rememberDiagnostic(diagnostics, diagnostic);
      return { diagnostics: [diagnostic], hint: null, kind: 'asha_renderer_editor_viewport_pick.v0' };
    }
    return {
      diagnostics: [],
      hint: { ...receipt.hit, handle: logicalRenderHandle },
      kind: 'asha_renderer_editor_viewport_pick.v0',
    };
  } catch (error) {
    const diagnostic = backendDiagnostic(null, error);
    rememberDiagnostic(diagnostics, diagnostic);
    return { diagnostics: [diagnostic], hint: null, kind: 'asha_renderer_editor_viewport_pick.v0' };
  }
}

function validatePickRequest(
  status: AshaRendererEditorViewportStatus,
  size: AshaRendererEditorViewportSize,
  request: AshaRendererEditorViewportPickRequest,
): AshaRendererEditorViewportDiagnostic | null {
  if (status === 'disposed') {
    return viewportDisposedDiagnostic(null);
  }
  const [x, y] = request.point;
  const counts = [
    request.filter?.channels?.length ?? 0,
    request.filter?.handles?.length ?? 0,
    request.filter?.layers?.length ?? 0,
    request.filter?.tags?.length ?? 0,
  ];
  const invalidPoint = !Number.isFinite(x) || !Number.isFinite(y)
    || x < 0 || x > size.width || y < 0 || y > size.height;
  const invalidDistance = request.maxDistance !== undefined
    && (!Number.isFinite(request.maxDistance) || request.maxDistance <= 0);
  const invalidChannel = request.filter?.channels?.some((channel) => !CHANNELS.includes(channel)) ?? false;
  const invalidHandle = request.filter?.handles?.some((handle) =>
    !Number.isSafeInteger(handle) || handle < 0 || handle > MAX_LOGICAL_HANDLE,
  ) ?? false;
  if (invalidPoint || invalidDistance || invalidChannel || invalidHandle
    || counts.some((count) => count > MAX_PICK_FILTER_VALUES)) {
    return {
      channel: null,
      code: 'invalid_pick_request',
      message: 'pick point, distance, channel, handle, and filter bounds must be valid for the current viewport',
      recoverable: true,
    };
  }
  return null;
}

function validateCamera(
  camera: AshaRendererEditorViewportCamera,
): AshaRendererEditorViewportDiagnostic | null {
  const sourceValid = camera.source === 'stored_editor' || camera.source === 'runtime_authority';
  const vectors = [camera.pose.position, camera.basis.forward, camera.basis.right, camera.basis.up];
  const finite = vectors.every((vector) => vector.every(Number.isFinite))
    && Number.isFinite(camera.pose.yawDegrees)
    && Number.isFinite(camera.pose.pitchDegrees)
    && Number.isFinite(camera.projection.fovYDegrees)
    && Number.isFinite(camera.projection.near)
    && Number.isFinite(camera.projection.far);
  const basisValid = [camera.basis.forward, camera.basis.right, camera.basis.up]
    .every((vector) => Math.abs(Math.hypot(...vector) - 1) <= 0.01)
    && Math.abs(dot(camera.basis.forward, camera.basis.right)) <= 0.01
    && Math.abs(dot(camera.basis.forward, camera.basis.up)) <= 0.01
    && Math.abs(dot(camera.basis.right, camera.basis.up)) <= 0.01;
  if (!sourceValid || !finite || !basisValid || camera.projection.fovYDegrees <= 0
    || camera.projection.fovYDegrees >= 180 || camera.projection.near <= 0
    || camera.projection.far <= camera.projection.near) {
    return {
      channel: null,
      code: 'invalid_camera',
      message: 'editor viewport camera requires finite pose, orthonormal basis, and valid perspective bounds',
      recoverable: true,
    };
  }
  return null;
}

function validateSize(
  size: AshaRendererEditorViewportSize,
): AshaRendererEditorViewportDiagnostic | null {
  if (!Number.isSafeInteger(size.width) || !Number.isSafeInteger(size.height)
    || size.width <= 0 || size.height <= 0
    || size.width > MAX_VIEWPORT_DIMENSION || size.height > MAX_VIEWPORT_DIMENSION
    || !Number.isFinite(size.pixelRatio) || size.pixelRatio <= 0 || size.pixelRatio > 4) {
    return {
      channel: null,
      code: 'invalid_viewport_size',
      message: `viewport width and height must be integers from 1 through ${MAX_VIEWPORT_DIMENSION}; pixelRatio must be in (0, 4]`,
      recoverable: true,
    };
  }
  return null;
}

function validateGrid(
  descriptor: EditorGridDescriptor | null,
): AshaRendererEditorViewportDiagnostic | null {
  if (descriptor === null) return null;
  const finiteTuple = (values: readonly number[]): boolean => values.every(Number.isFinite);
  const normalizedColor = (values: readonly number[]): boolean =>
    finiteTuple(values) && values.every(value => value >= 0 && value <= 1);
  const coordinateSystemValid = descriptor.grid.coordinateSystem === 'rightHandedYUp';
  const gridValid = finiteTuple(descriptor.grid.origin)
    && finiteTuple(descriptor.grid.spacing)
    && descriptor.grid.spacing.every(value => value > 0);
  const colors = [
    descriptor.style.minorColor,
    descriptor.style.majorColor,
    descriptor.style.xAxisColor,
    descriptor.style.yAxisColor,
    descriptor.style.zAxisColor,
  ];
  const styleValid = colors.every(normalizedColor)
    && Number.isSafeInteger(descriptor.style.majorLineEvery)
    && descriptor.style.majorLineEvery > 0
    && Number.isFinite(descriptor.style.opacity)
    && descriptor.style.opacity >= 0
    && descriptor.style.opacity <= 1
    && Number.isFinite(descriptor.style.fadeStart)
    && Number.isFinite(descriptor.style.fadeEnd)
    && descriptor.style.fadeStart >= 0
    && descriptor.style.fadeEnd > descriptor.style.fadeStart;
  if (!coordinateSystemValid || !gridValid || !styleValid) {
    return {
      channel: null,
      code: 'invalid_grid',
      message: 'editor grid requires right-handed Y-up coordinates, finite positive spacing, normalized colors/opacity, and an increasing fade range',
      recoverable: true,
    };
  }
  return null;
}

function cloneGrid(descriptor: EditorGridDescriptor | null): EditorGridDescriptor | null {
  return descriptor === null ? null : structuredClone(descriptor);
}

function defaultEditorCamera(): AshaRendererEditorViewportCamera {
  return {
    source: 'stored_editor',
    pose: { position: [4, 4, 8], yawDegrees: 0, pitchDegrees: -20 },
    basis: {
      forward: [-0.408248, -0.408248, -0.816497],
      right: [0.894427, 0, -0.447214],
      up: [-0.182574, 0.912871, -0.365148],
    },
    projection: { fovYDegrees: 55, near: 0.05, far: 1000 },
  };
}

function snapshotChannel(state: ChannelState): AshaRendererEditorViewportChannelSnapshot {
  const projection = state.projection.snapshot();
  return {
    channel: state.channel,
    disposed: state.disposed,
    generation: state.generation,
    hash: stableHash({ channel: state.channel, history: state.history, projection }),
    retainedOpCount: state.history.length,
    projection,
  };
}

function acceptedChannelReceipt(state: ChannelState): AshaRendererEditorViewportChannelReceipt {
  const snapshot = snapshotChannel(state);
  return {
    applied: true,
    channel: state.channel,
    diagnostics: [],
    generation: state.generation,
    snapshotHash: snapshot.hash,
  };
}

function rejectedChannelReceipt(
  state: ChannelState,
  diagnostics: AshaRendererEditorViewportDiagnostic[],
  diagnostic: AshaRendererEditorViewportDiagnostic,
): AshaRendererEditorViewportChannelReceipt {
  rememberDiagnostic(diagnostics, diagnostic);
  return {
    applied: false,
    channel: state.channel,
    diagnostics: [diagnostic],
    generation: state.generation,
    snapshotHash: snapshotChannel(state).hash,
  };
}

function invalidFrameDiagnostic(
  channel: AshaRendererEditorViewportChannel,
  message: string,
): AshaRendererEditorViewportDiagnostic {
  return { channel, code: 'invalid_frame', message, recoverable: true };
}

function backendDiagnostic(
  channel: AshaRendererEditorViewportChannel | null,
  error: unknown,
): AshaRendererEditorViewportDiagnostic {
  return {
    channel,
    code: 'backend_rejected',
    message: error instanceof Error ? error.message : String(error),
    recoverable: true,
  };
}

function viewportDisposedDiagnostic(
  channel: AshaRendererEditorViewportChannel | null,
): AshaRendererEditorViewportDiagnostic {
  return {
    channel,
    code: 'viewport_disposed',
    message: 'renderer editor viewport is disposed',
    recoverable: false,
  };
}

function requireState(
  states: ReadonlyMap<AshaRendererEditorViewportChannel, ChannelState>,
  channel: AshaRendererEditorViewportChannel,
): ChannelState {
  const state = states.get(channel);
  if (state === undefined) {
    throw new Error(`renderer editor viewport channel ${channel} is unavailable`);
  }
  return state;
}

function rememberDiagnostic(
  diagnostics: AshaRendererEditorViewportDiagnostic[],
  diagnostic: AshaRendererEditorViewportDiagnostic,
): void {
  diagnostics.push(diagnostic);
  if (diagnostics.length > MAX_DIAGNOSTICS) {
    diagnostics.splice(0, diagnostics.length - MAX_DIAGNOSTICS);
  }
}

function dot(
  left: readonly [number, number, number],
  right: readonly [number, number, number],
): number {
  return left[0] * right[0] + left[1] * right[1] + left[2] * right[2];
}

type StableValue = string | number | boolean | null | readonly StableValue[] | { readonly [key: string]: StableValue | undefined };

function stableHash(value: unknown): string {
  return `fnv1a64:${fnv1a64(stableStringify(value as StableValue))}`;
}

function stableStringify(value: StableValue | undefined): string {
  if (value === undefined) return 'undefined';
  if (value === null || typeof value !== 'object') return JSON.stringify(value);
  if (isStableValueArray(value)) return `[${value.map((entry) => stableStringify(entry)).join(',')}]`;
  const record = value as { readonly [key: string]: StableValue | undefined };
  return `{${Object.keys(record).sort().map((key) =>
    `${JSON.stringify(key)}:${stableStringify(record[key])}`,
  ).join(',')}}`;
}

function hasRenderFrameOps(value: unknown): value is RenderFrameDiff {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  return Array.isArray((value as { readonly ops?: unknown }).ops);
}

function isStableValueArray(value: StableValue): value is readonly StableValue[] {
  return Array.isArray(value);
}

function fnv1a64(value: string): string {
  let hash = 0xcbf29ce484222325n;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= BigInt(value.charCodeAt(index));
    hash = BigInt.asUintN(64, hash * 0x100000001b3n);
  }
  return hash.toString(16).padStart(16, '0');
}
