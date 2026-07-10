import type {
  VoxelConversionMeshAssetRegistrationRequest,
  VoxelConversionMeshSourceImportReceipt,
  VoxelConversionMeshSourceImportRequest,
  VoxelConversionSourceRef,
} from '@asha/contracts';
import type { NativeAddon } from '@asha/native-bridge';

const QUAD_SOURCE = {
  assetId: 'mesh/quad',
  assetKind: 'mesh',
  assetVersion: 1,
  sourceHash: 'sha256:quad',
  meshPrimitive: null,
} satisfies VoxelConversionSourceRef;

export const VOXEL_CONVERSION_MESH_ASSET_REGISTRATION_REQUEST = {
  source: QUAD_SOURCE,
  meshAsset: {
    assetId: 'mesh/quad',
    sourcePath: 'assets/mesh/quad.mesh.json',
    positions: [[0, 0, 0], [1, 0, 0], [0, 1, 0]] as const,
    normals: [] as const,
    indices: [0, 1, 2] as const,
    groups: [{ materialSlot: 0, start: 0, count: 3 }],
    materialSlots: [{ sourceMaterialSlot: 0, sourceMaterialId: 'mat/a' }],
  },
} satisfies VoxelConversionMeshAssetRegistrationRequest;

export const VOXEL_CONVERSION_MESH_SOURCE_IMPORT_REQUEST = {
  sourceAssetId: 'mesh/imported-wall',
  assetVersion: 3,
  sourcePath: 'assets/mesh/imported-wall.glb',
  format: 'glb',
  sourceBytes: [103, 108, 84, 70, 2, 0, 0, 0, 12, 0, 0, 0],
  meshPrimitive: null,
} satisfies VoxelConversionMeshSourceImportRequest;

export function importedMeshReceipt(
  request: VoxelConversionMeshSourceImportRequest,
): VoxelConversionMeshSourceImportReceipt {
  const source = {
    assetId: request.sourceAssetId,
    assetKind: 'mesh',
    assetVersion: request.assetVersion,
    sourceHash: 'sha256:imported-wall',
    meshPrimitive: request.meshPrimitive,
  };
  const materialSlots = [{ sourceMaterialSlot: 0, sourceMaterialId: 'material/wall' }];
  const groups = [{
    groupId: 'group:0:material-slot:0',
    label: 'Wall',
    materialSlot: 0,
    start: 0,
    count: 6,
    bounds: { min: [0, 0, 0] as const, max: [2, 2, 0] as const },
  }];
  return {
    source,
    imported: true,
    sourcePath: request.sourcePath,
    format: request.format,
    sourceByteCount: request.sourceBytes.length,
    meshAsset: {
      assetId: request.sourceAssetId,
      sourcePath: request.sourcePath,
      positions: [[0, 0, 0], [2, 0, 0], [2, 2, 0], [0, 2, 0]],
      normals: [],
      indices: [0, 1, 2, 0, 2, 3],
      groups: [{ materialSlot: 0, start: 0, count: 6 }],
      materialSlots,
    },
    sourceBounds: { min: [0, 0, 0], max: [2, 2, 0] },
    vertexCount: 4,
    triangleCount: 2,
    groups,
    materialSlots,
    diagnostics: [],
    evidence: [{
      kind: 'source_snapshot',
      uri: `asha://voxel-conversion/source/${request.sourceAssetId}`,
      contentHash: source.sourceHash,
    }],
  };
}

export function createNativeVoxelMeshSourceHandlers(
  calls: string[],
): Pick<NativeAddon, 'registerVoxelConversionMeshAsset' | 'importVoxelConversionMeshSource'> {
  return {
    registerVoxelConversionMeshAsset: (_handle, requestJson) => {
      calls.push(`voxelMeshAssetRegister:${requestJson}`);
      const request = JSON.parse(requestJson) as VoxelConversionMeshAssetRegistrationRequest;
      return JSON.stringify({
        source: request.source,
        registered: true,
        materialSlots: request.meshAsset.materialSlots,
        diagnostics: [],
        evidence: [{
          kind: 'source_snapshot',
          uri: `asha://voxel-conversion/source/${request.meshAsset.assetId}`,
          contentHash: request.source.sourceHash,
        }],
      });
    },
    importVoxelConversionMeshSource: (_handle, requestJson) => {
      calls.push(`voxelMeshSourceImport:${requestJson}`);
      const request = JSON.parse(requestJson) as VoxelConversionMeshSourceImportRequest;
      return JSON.stringify(importedMeshReceipt(request));
    },
  };
}
