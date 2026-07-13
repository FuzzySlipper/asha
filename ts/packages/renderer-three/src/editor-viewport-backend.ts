// Engine-owned Three.js realization for the backend-neutral editor viewport.

import * as THREE from 'three';
import type {
  CameraBasis,
  CameraPose,
  EntityId,
  PerspectiveProjection,
  RenderFrameDiff,
  RenderHandle,
  RenderLayer,
  TagId,
} from '@asha/contracts';
import type { AnimatedMeshAssetSource } from './animated-mesh.js';
import {
  pickProjectedObject,
  type AshaRendererBrowserSurfacePickFilter,
} from './browser-surface.js';
import { ThreeRenderer, type MeshBufferSource } from './three-renderer.js';

export type AshaRendererEditorBackendChannel = 'runtime' | 'authored' | 'overlay';

export interface AshaRendererEditorBackendCamera {
  readonly basis: CameraBasis;
  readonly pose: CameraPose;
  readonly projection: PerspectiveProjection;
}

export interface AshaRendererEditorBackendSize {
  readonly height: number;
  readonly pixelRatio: number;
  readonly width: number;
}

export interface AshaRendererEditorBackendPickFilter {
  readonly channels?: readonly AshaRendererEditorBackendChannel[];
  readonly handles?: readonly RenderHandle[];
  readonly layers?: readonly RenderLayer[];
  readonly tags?: readonly TagId[];
}

export interface AshaRendererEditorBackendPickRequest {
  readonly filter?: AshaRendererEditorBackendPickFilter;
  readonly maxDistance?: number;
  readonly point: readonly [number, number];
}

export interface AshaRendererEditorBackendPickHit {
  readonly channel: AshaRendererEditorBackendChannel;
  readonly distance: number;
  readonly handle: RenderHandle;
  readonly label: string | null;
  readonly layer: RenderLayer;
  readonly normal: readonly [number, number, number];
  readonly position: readonly [number, number, number];
  readonly sourceTrace: {
    readonly entity: EntityId;
    readonly kind: 'render_metadata_entity';
  } | null;
  readonly tags: readonly TagId[];
}

export interface AshaRendererEditorBackendPickReceipt {
  readonly diagnostics: readonly { readonly code: string; readonly message: string }[];
  readonly hit: AshaRendererEditorBackendPickHit | null;
}

export interface AshaRendererEditorBackendOptions {
  readonly animatedMeshSource?: AnimatedMeshAssetSource;
  readonly clearColor?: number;
  readonly meshBufferSource?: MeshBufferSource;
  readonly pixelRatio?: number;
}

export interface AshaRendererEditorBackend {
  readonly dispose: () => void;
  readonly pick: (request: AshaRendererEditorBackendPickRequest) => AshaRendererEditorBackendPickReceipt;
  readonly renderOnce: (timeMs?: number) => void;
  readonly replaceChannel: (channel: AshaRendererEditorBackendChannel, frame: RenderFrameDiff) => void;
  readonly resize: (size: AshaRendererEditorBackendSize) => void;
  readonly setCamera: (camera: AshaRendererEditorBackendCamera) => void;
  readonly snapshot: () => string;
  readonly start: () => void;
  readonly stop: () => void;
}

const CHANNEL_ORDER: readonly AshaRendererEditorBackendChannel[] = [
  'runtime',
  'authored',
  'overlay',
];

/** Engine-internal retained channel set used by the mounted WebGL backend. */
export class AshaRendererEditorProjectionChannels {
  readonly #options: AshaRendererEditorBackendOptions;
  readonly #renderers = new Map<AshaRendererEditorBackendChannel, ThreeRenderer>();

  constructor(options: AshaRendererEditorBackendOptions = {}) {
    this.#options = options;
    for (const channel of CHANNEL_ORDER) {
      this.#renderers.set(channel, createChannelRenderer(options));
    }
  }

  renderer(channel: AshaRendererEditorBackendChannel): ThreeRenderer {
    return requireChannelRenderer(this.#renderers, channel);
  }

  replace(channel: AshaRendererEditorBackendChannel, frame: RenderFrameDiff): void {
    const candidate = createChannelRenderer(this.#options);
    try {
      candidate.applyFrame(frame);
    } catch (error) {
      candidate.dispose();
      throw error;
    }
    const previous = this.renderer(channel);
    this.#renderers.set(channel, candidate);
    previous.dispose();
  }

  snapshot(): string {
    return CHANNEL_ORDER.map((channel) =>
      `[${channel}]\n${this.renderer(channel).snapshot()}`,
    ).join('\n');
  }

  dispose(): void {
    for (const renderer of this.#renderers.values()) {
      renderer.dispose();
    }
    this.#renderers.clear();
  }
}

export function mountAshaRendererEditorBackend(
  canvas: HTMLCanvasElement,
  options: AshaRendererEditorBackendOptions = {},
): AshaRendererEditorBackend {
  const channels = new AshaRendererEditorProjectionChannels(options);

  const webgl = new THREE.WebGLRenderer({ canvas, antialias: true });
  webgl.autoClear = false;
  webgl.setClearColor(options.clearColor ?? 0x101820, 1);
  const camera = new THREE.PerspectiveCamera(55, 1, 0.1, 1000);
  const raycaster = new THREE.Raycaster();
  const pickPoint = new THREE.Vector2();
  const lookTarget = new THREE.Vector3();
  let size: AshaRendererEditorBackendSize = {
    width: Math.max(1, canvas.clientWidth || canvas.width || 800),
    height: Math.max(1, canvas.clientHeight || canvas.height || 450),
    pixelRatio: options.pixelRatio ?? globalThis.devicePixelRatio ?? 1,
  };
  let animationFrame: number | null = null;
  let lastRenderTimeMs: number | null = null;
  let disposed = false;

  const resize = (next: AshaRendererEditorBackendSize): void => {
    requireActive(disposed);
    size = next;
    webgl.setPixelRatio(next.pixelRatio);
    webgl.setSize(next.width, next.height, false);
    camera.aspect = next.width / next.height;
    camera.updateProjectionMatrix();
  };

  const setCamera = (next: AshaRendererEditorBackendCamera): void => {
    requireActive(disposed);
    camera.position.set(...next.pose.position);
    camera.up.set(...next.basis.up);
    lookTarget.set(
      next.pose.position[0] + next.basis.forward[0],
      next.pose.position[1] + next.basis.forward[1],
      next.pose.position[2] + next.basis.forward[2],
    );
    camera.lookAt(lookTarget);
    camera.fov = next.projection.fovYDegrees;
    camera.near = next.projection.near;
    camera.far = next.projection.far;
    camera.updateProjectionMatrix();
  };

  const renderOnce = (timeMs = globalThis.performance?.now() ?? 0): void => {
    requireActive(disposed);
    const deltaSeconds = lastRenderTimeMs === null
      ? 0
      : Math.min(0.05, Math.max(0, (timeMs - lastRenderTimeMs) / 1000));
    lastRenderTimeMs = timeMs;
    webgl.clear(true, true, true);
    for (const channel of CHANNEL_ORDER) {
      const renderer = channels.renderer(channel);
      renderer.advanceAnimation(deltaSeconds);
      if (channel === 'overlay') {
        webgl.clearDepth();
      }
      webgl.render(renderer.scene, camera);
    }
  };

  const tick = (timeMs: number): void => {
    renderOnce(timeMs);
    animationFrame = globalThis.requestAnimationFrame(tick);
  };

  const start = (): void => {
    requireActive(disposed);
    if (animationFrame === null) {
      animationFrame = globalThis.requestAnimationFrame(tick);
    }
  };

  const stop = (): void => {
    if (animationFrame !== null) {
      globalThis.cancelAnimationFrame(animationFrame);
      animationFrame = null;
    }
  };

  resize(size);

  return {
    replaceChannel: (channel, frame) => {
      requireActive(disposed);
      channels.replace(channel, frame);
    },
    setCamera,
    resize,
    pick: (request) => pickAcrossChannels(channels, camera, raycaster, pickPoint, request),
    renderOnce,
    start,
    stop,
    snapshot: () => channels.snapshot(),
    dispose: () => {
      if (disposed) {
        return;
      }
      stop();
      disposed = true;
      channels.dispose();
      webgl.dispose();
    },
  };
}

function createChannelRenderer(options: AshaRendererEditorBackendOptions): ThreeRenderer {
  const renderer = new ThreeRenderer({
    ...(options.animatedMeshSource === undefined
      ? {}
      : { animatedMeshSource: options.animatedMeshSource }),
    ...(options.meshBufferSource === undefined
      ? {}
      : { meshBufferSource: options.meshBufferSource }),
  });
  const ambientLight = new THREE.HemisphereLight(0xffffff, 0x263238, 2.4);
  const keyLight = new THREE.DirectionalLight(0xffffff, 2.2);
  keyLight.position.set(5, 8, 6);
  renderer.scene.add(ambientLight, keyLight);
  return renderer;
}

function pickAcrossChannels(
  projectionChannels: AshaRendererEditorProjectionChannels,
  camera: THREE.PerspectiveCamera,
  raycaster: THREE.Raycaster,
  point: THREE.Vector2,
  request: AshaRendererEditorBackendPickRequest,
): AshaRendererEditorBackendPickReceipt {
  const requestedChannels = request.filter?.channels ?? CHANNEL_ORDER;
  let selected: AshaRendererEditorBackendPickHit | null = null;
  for (const channel of CHANNEL_ORDER) {
    if (!requestedChannels.includes(channel)) {
      continue;
    }
    const renderer = projectionChannels.renderer(channel);
    const filter: AshaRendererBrowserSurfacePickFilter = {
      ...(request.filter?.handles === undefined ? {} : { handles: request.filter.handles }),
      ...(request.filter?.layers === undefined ? {} : { layers: request.filter.layers }),
      ...(request.filter?.tags === undefined ? {} : { tags: request.filter.tags }),
    };
    const receipt = pickProjectedObject(renderer, camera, raycaster, point, {
      ray: { kind: 'viewport', point: request.point },
      ...(request.maxDistance === undefined ? {} : { maxDistance: request.maxDistance }),
      ...(Object.keys(filter).length === 0 ? {} : { filter }),
    });
    if (receipt.diagnostics.length > 0) {
      return { diagnostics: receipt.diagnostics, hit: null };
    }
    if (receipt.hit !== null && (selected === null || receipt.hit.distance < selected.distance)) {
      selected = { ...receipt.hit, channel };
    }
  }
  return { diagnostics: [], hit: selected };
}

function requireChannelRenderer(
  renderers: ReadonlyMap<AshaRendererEditorBackendChannel, ThreeRenderer>,
  channel: AshaRendererEditorBackendChannel,
): ThreeRenderer {
  const renderer = renderers.get(channel);
  if (renderer === undefined) {
    throw new Error(`editor viewport backend channel ${channel} is unavailable`);
  }
  return renderer;
}

function requireActive(disposed: boolean): void {
  if (disposed) {
    throw new Error('editor viewport backend is disposed');
  }
}
