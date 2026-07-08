import { consumerCompatibilityDiagnostic, } from './manifest-types.js';
export function validateAshaConsumerCompatibility(manifest, metadata) {
    const diagnostics = [];
    const contracts = requireSurface(metadata.contracts, 'contracts', diagnostics);
    const runtimeBridge = requireSurface(metadata.runtimeBridge, 'runtimeBridge', diagnostics);
    const devtoolsProtocol = requireProtocol(metadata.devtoolsProtocol, 'devtoolsProtocol', diagnostics);
    const publishArtifact = requireProtocol(metadata.publishArtifact, 'publishArtifact', diagnostics);
    if (contracts !== null) {
        compareVersion(manifest.asha.contractsVersion, contracts.packageVersion, 'asha.contracts_version', diagnostics);
    }
    if (runtimeBridge !== null) {
        compareVersion(manifest.asha.runtimeBridgeVersion, runtimeBridge.packageVersion, 'asha.runtime_bridge_version', diagnostics);
    }
    if (devtoolsProtocol !== null) {
        compareVersion(manifest.asha.devtoolsProtocolVersion, devtoolsProtocol.compatibilityVersion, 'asha.devtools_protocol_version', diagnostics);
    }
    if (publishArtifact !== null) {
        compareVersion(manifest.asha.publishArtifactFormatVersion, publishArtifact.compatibilityVersion, 'asha.publish_artifact_format_version', diagnostics);
    }
    if (diagnostics.length > 0 || contracts === null || runtimeBridge === null || devtoolsProtocol === null || publishArtifact === null) {
        return { ok: false, diagnostics };
    }
    return {
        ok: true,
        metadata: { contracts, runtimeBridge, devtoolsProtocol, publishArtifact },
        diagnostics: [],
    };
}
function requireSurface(surface, path, diagnostics) {
    if (surface === undefined || surface.compatibilityVersion.length === 0 || surface.packageVersion.length === 0) {
        diagnostics.push(consumerCompatibilityDiagnostic('missing_metadata', path, `missing ${path} compatibility metadata`));
        return null;
    }
    return surface;
}
function requireProtocol(protocol, path, diagnostics) {
    if (protocol === undefined || protocol.compatibilityVersion.length === 0) {
        diagnostics.push(consumerCompatibilityDiagnostic('missing_metadata', path, `missing ${path} compatibility metadata`));
        return null;
    }
    return protocol;
}
function compareVersion(manifestVersion, metadataVersion, path, diagnostics) {
    if (manifestVersion !== metadataVersion) {
        diagnostics.push(consumerCompatibilityDiagnostic('incompatible_version', path, `manifest declares "${manifestVersion}" but ASHA metadata provides "${metadataVersion}"`));
    }
}
//# sourceMappingURL=manifest-compatibility.js.map