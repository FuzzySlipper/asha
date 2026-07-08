export const ASHA_GAME_WORKSPACE_COMPATIBILITY = {
    contracts: { compatibilityVersion: 'contracts.v0', packageVersion: '0.1.0' },
    runtimeBridge: { compatibilityVersion: 'runtime-bridge.v0', packageVersion: '0.1.0' },
    devtoolsProtocol: { compatibilityVersion: 'devtools-protocol.v0' },
    publishArtifact: { compatibilityVersion: 'publish-artifact.v0' },
};
export function manifestDiagnostic(code, path, message) {
    return { code, path, message };
}
export function consumerCompatibilityDiagnostic(code, path, message) {
    return { code, path, message };
}
//# sourceMappingURL=manifest-types.js.map