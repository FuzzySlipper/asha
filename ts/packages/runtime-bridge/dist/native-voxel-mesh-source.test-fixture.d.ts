import type { VoxelConversionMeshSourceImportReceipt, VoxelConversionMeshSourceImportRequest } from '@asha/contracts';
import type { NativeAddon } from '@asha/native-bridge';
export declare const VOXEL_CONVERSION_MESH_ASSET_REGISTRATION_REQUEST: {
    source: {
        assetId: string;
        assetKind: string;
        assetVersion: number;
        sourceHash: string;
        meshPrimitive: null;
    };
    meshAsset: {
        assetId: string;
        sourcePath: string;
        positions: readonly [readonly [0, 0, 0], readonly [1, 0, 0], readonly [0, 1, 0]];
        normals: readonly [];
        indices: readonly [0, 1, 2];
        groups: {
            materialSlot: number;
            start: number;
            count: number;
        }[];
        materialSlots: {
            sourceMaterialSlot: number;
            sourceMaterialId: string;
        }[];
    };
};
export declare const VOXEL_CONVERSION_MESH_SOURCE_IMPORT_REQUEST: {
    sourceAssetId: string;
    assetVersion: number;
    sourcePath: string;
    format: "glb";
    sourceBytes: number[];
    meshPrimitive: null;
};
export declare function importedMeshReceipt(request: VoxelConversionMeshSourceImportRequest): VoxelConversionMeshSourceImportReceipt;
export declare function createNativeVoxelMeshSourceHandlers(calls: string[]): Pick<NativeAddon, 'registerVoxelConversionMeshAsset' | 'importVoxelConversionMeshSource'>;
//# sourceMappingURL=native-voxel-mesh-source.test-fixture.d.ts.map