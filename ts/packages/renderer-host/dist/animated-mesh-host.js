import { MapAnimatedMeshAssetSource, ThreeRenderer, loadAnimatedMeshGlbResource, } from '@asha/renderer-three/backend';
export class AshaRendererHostError extends Error {
    diagnostics;
    constructor(diagnostics) {
        super(diagnostics.map((diagnostic) => diagnostic.message).join('; '));
        this.name = 'AshaRendererHostError';
        this.diagnostics = diagnostics;
    }
}
export const ASHA_RENDERER_HOST_KENNEY_ANIMATED_MESH_RESOURCE = {
    asset: 'mesh-animation/kenney-retro-character-medium',
    resourceUrl: new URL('../assets/kenney-retro-character-medium.glb', import.meta.url).href,
    contentHash: 'sha256:c71255a41c0373f0d2ef52593369d5fd9d2f6220ae548aff8cd6bf5edb403674',
    clipIds: ['idle', 'run', 'jump'],
    licenseUrl: new URL('../assets/LICENSE.Kenney-Animated-Characters-Retro.txt', import.meta.url).href,
};
export const ASHA_RENDERER_HOST_ANIMATED_MESH_FIXTURE_MANIFEST = {
    kind: 'asha_renderer_animated_mesh_resources.v0',
    resources: [ASHA_RENDERER_HOST_KENNEY_ANIMATED_MESH_RESOURCE],
};
export async function createAshaRendererAnimatedMeshProjection(options) {
    const source = await loadRendererAnimatedMeshSource(options.manifest, options.resolveResource);
    const renderer = new ThreeRenderer({ animatedMeshSource: source });
    return createProjectionController(renderer);
}
export async function loadRendererAnimatedMeshSource(manifest, resolver = resolveAnimatedMeshWithFetch) {
    validateManifest(manifest);
    const resources = await Promise.all(manifest.resources.map(async (descriptor) => {
        let data;
        try {
            data = await resolver(descriptor);
        }
        catch (cause) {
            throw hostError('animated_mesh_resource_unavailable', descriptor.asset, null, cause);
        }
        const actualHash = `sha256:${await sha256Hex(data)}`;
        if (actualHash !== descriptor.contentHash) {
            throw hostError('animated_mesh_content_hash_mismatch', descriptor.asset, null, `expected ${descriptor.contentHash}, received ${actualHash}`);
        }
        const resource = await loadAnimatedMeshGlbResource(descriptor.asset, data, descriptor.contentHash).catch((cause) => {
            throw hostError('animated_mesh_resource_unavailable', descriptor.asset, null, cause);
        });
        const availableClips = new Set(resource.clips.map((clip) => clip.name.toLowerCase()));
        const missingClip = descriptor.clipIds.find((clip) => !availableClips.has(clip.toLowerCase()));
        if (missingClip !== undefined) {
            throw hostError('animated_mesh_clip_unavailable', descriptor.asset, null, `missing clip ${missingClip}`);
        }
        return resource;
    }));
    return new MapAnimatedMeshAssetSource(resources);
}
export function animationPlaybackReadout(handle, readout) {
    if (readout === undefined) {
        return {
            handle,
            asset: null,
            status: 'unavailable',
            selectedClip: null,
            mixerTimeSeconds: 0,
            actionTimeSeconds: null,
            commandSelected: false,
            running: false,
            paused: false,
            loop: null,
            speed: null,
            weight: null,
            poseSample: null,
            diagnostics: [diagnostic('animated_mesh_handle_unavailable', null, handle, `animated mesh handle ${handle} is unavailable`)],
            projectionOnly: true,
            controllerClips: [],
        };
    }
    return {
        handle,
        asset: readout.asset,
        status: readout.status,
        selectedClip: readout.currentClip,
        mixerTimeSeconds: readout.mixerTimeSeconds,
        actionTimeSeconds: readout.actionTimeSeconds,
        commandSelected: readout.commandSelected,
        running: readout.running,
        paused: readout.paused,
        loop: readout.loop,
        speed: readout.speed,
        weight: readout.weight,
        poseSample: readout.poseSample,
        diagnostics: readout.diagnostics.map((code) => diagnostic(animationDiagnosticCode(code), readout.asset, handle, code)),
        projectionOnly: true,
        controllerClips: readout.controllerClips,
    };
}
function createProjectionController(renderer) {
    return {
        kind: 'asha_renderer_animated_mesh_projection.v0',
        applyFrame: (frame) => applyRendererOperation(() => renderer.applyFrame(frame)),
        advance: (deltaSeconds) => applyRendererOperation(() => renderer.advanceAnimation(deltaSeconds)),
        playback: (handle) => animationPlaybackReadout(handle, renderer.animatedMeshPlayback(handle)),
        snapshot: () => renderer.snapshot(),
        hasAnimationTarget: (handle) => renderer.has(handle),
        setAnimationControllerWeights: (handle, clips) => {
            renderer.setAnimationControllerWeights(handle, clips);
        },
        hasAnimationClips: (handle, clipIds) => renderer.hasAnimationControllerClips(handle, clipIds),
        clearAnimationControllerWeights: (handle) => renderer.clearAnimationControllerWeights(handle),
    };
}
function applyRendererOperation(operation) {
    try {
        operation();
        return { applied: true, diagnostics: [] };
    }
    catch (cause) {
        return {
            applied: false,
            diagnostics: [diagnostic('animated_mesh_frame_rejected', null, null, errorMessage(cause))],
        };
    }
}
async function resolveAnimatedMeshWithFetch(descriptor) {
    const response = await fetch(descriptor.resourceUrl);
    if (!response.ok) {
        throw new Error(`resource request failed with HTTP ${response.status}`);
    }
    return response.arrayBuffer();
}
function validateManifest(manifest) {
    if (manifest.kind !== 'asha_renderer_animated_mesh_resources.v0' || manifest.resources.length === 0) {
        throw hostError('animated_mesh_manifest_invalid', null, null, 'animated mesh resource manifest is empty or unsupported');
    }
    const assets = new Set();
    for (const resource of manifest.resources) {
        const validHash = /^sha256:[0-9a-f]{64}$/u.test(resource.contentHash);
        const validClips = resource.clipIds.length > 0 && new Set(resource.clipIds).size === resource.clipIds.length;
        if (resource.asset.length === 0 || resource.resourceUrl.length === 0 || !validHash || !validClips || assets.has(resource.asset)) {
            throw hostError('animated_mesh_manifest_invalid', resource.asset || null, null, 'animated mesh resource descriptor is invalid or duplicated');
        }
        assets.add(resource.asset);
    }
}
async function sha256Hex(data) {
    if (globalThis.crypto?.subtle === undefined) {
        throw hostError('animated_mesh_resource_unavailable', null, null, 'Web Crypto SHA-256 is unavailable');
    }
    const digest = await globalThis.crypto.subtle.digest('SHA-256', data);
    return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}
function animationDiagnosticCode(code) {
    switch (code) {
        case 'animation_not_started':
        case 'animation_paused':
        case 'animation_stopped':
            return code;
        default:
            return 'animated_mesh_frame_rejected';
    }
}
function hostError(code, asset, handle, cause) {
    return new AshaRendererHostError([diagnostic(code, asset, handle, errorMessage(cause))]);
}
function diagnostic(code, asset, handle, message) {
    return { code, message, asset, handle };
}
function errorMessage(cause) {
    return cause instanceof Error ? cause.message : String(cause);
}
//# sourceMappingURL=animated-mesh-host.js.map