export function consumeRuntimeSessionAnimationIntent(session, projection, surface) {
    const intent = session.readAnimationIntent();
    projection.applyFrame(intent.frame);
    surface.applyFrame(intent.frame);
    return {
        projectionStatus: projection.playback(intent.instanceHandle).status,
        surfaceStatus: surface.animatedMeshPlayback(intent.instanceHandle).status,
    };
}
//# sourceMappingURL=renderer-host-runtime-session-consumer-proof.js.map