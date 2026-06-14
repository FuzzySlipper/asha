// @asha/editor-tools — proposal-only scene authoring controls (#2380).
//
// These builders turn authoring intent (add a static-mesh node, add a sprite,
// group/reparent, set an initial transform, edit metadata/tags) into typed
// **proposals** built from generated `@asha/contracts` scene types. They are pure
// and they NEVER submit, validate, or mutate authority — Rust owns scene
// validation and the runtime facade owns submission.
//
// `applyProposalToDraft` produces a UI-local *draft* document for preview and for
// handing to authority validation. The draft is explicitly NOT authority truth: a
// proposal is accepted only when Rust validation passes and authority replays it.
// `summarizeValidation` turns the authoritative `SceneValidationReport` (from Rust)
// into an accept/reject readout — the UI reflects that, it does not decide it.
const IDENTITY_TRANSFORM = {
    translation: [0, 0, 0],
    rotation: [0, 0, 0, 1],
    scale: [1, 1, 1],
};
function newRecord(id, kind, authoring) {
    return {
        id,
        parent: authoring.parent ?? null,
        childOrder: authoring.childOrder ?? 0,
        label: authoring.label ?? null,
        tags: authoring.tags ?? [],
        transform: authoring.transform ?? IDENTITY_TRANSFORM,
        kind,
    };
}
/** Propose adding a static-mesh node bound to a catalog mesh asset. */
export function proposeAddStaticMesh(id, asset, authoring = {}) {
    return { op: 'addNode', node: newRecord(id, { kind: 'staticMesh', asset }, authoring) };
}
/** Propose adding a sprite node bound to a catalog sprite asset. */
export function proposeAddSprite(id, asset, authoring = {}) {
    return { op: 'addNode', node: newRecord(id, { kind: 'sprite', asset }, authoring) };
}
/** Propose adding an empty grouping/transform node. */
export function proposeAddGroup(id, authoring = {}) {
    return { op: 'addNode', node: newRecord(id, { kind: 'emptyGroup' }, authoring) };
}
/** Propose reparenting (or grouping) a node under a new parent at a sibling index. */
export function proposeReparent(node, newParent, childOrder = 0) {
    return { op: 'reparent', node, newParent, childOrder };
}
/** Propose replacing a node's initial transform. */
export function proposeSetTransform(node, transform) {
    return { op: 'setTransform', node, transform };
}
/** Propose replacing a node's debug label and tags (never authority semantics). */
export function proposeSetMetadata(node, label, tags = []) {
    return { op: 'setMetadata', node, label, tags };
}
// ── Draft application (UI-local preview, not authority) ───────────────────────────
/**
 * Apply a proposal to a copy of `doc`, producing a UI-local **draft** for preview
 * and for handing to authority validation. Pure — returns a new document and never
 * mutates the input. The draft is not authority truth; only a validated, replayed
 * proposal becomes truth.
 *
 * Returns the unchanged document (structurally copied) when the proposal targets a
 * node that is absent — authority will reject it; the draft never invents the node.
 */
export function applyProposalToDraft(doc, proposal) {
    const nodes = doc.nodes.map((n) => ({ ...n }));
    const indexOf = (id) => nodes.findIndex((n) => n.id === id);
    switch (proposal.op) {
        case 'addNode': {
            // A draft does not dedupe ids — a duplicate is a validation concern, surfaced
            // by authority rather than silently swallowed here.
            nodes.push({ ...proposal.node });
            break;
        }
        case 'reparent': {
            const at = indexOf(proposal.node);
            if (at >= 0) {
                nodes[at] = { ...nodes[at], parent: proposal.newParent, childOrder: proposal.childOrder };
            }
            break;
        }
        case 'setTransform': {
            const at = indexOf(proposal.node);
            if (at >= 0) {
                nodes[at] = { ...nodes[at], transform: proposal.transform };
            }
            break;
        }
        case 'setMetadata': {
            const at = indexOf(proposal.node);
            if (at >= 0) {
                nodes[at] = { ...nodes[at], label: proposal.label, tags: [...proposal.tags] };
            }
            break;
        }
    }
    return { ...doc, nodes };
}
function describeValidationError(error) {
    switch (error.code) {
        case 'duplicate-node-id':
            return `node ${error.node} duplicates an existing id`;
        case 'unknown-parent':
            return `node ${error.node} names absent parent ${error.parent}`;
        case 'cycle':
            return `parent cycle: ${error.cyclePath.map((id) => id).join(' → ')}`;
        case 'invalid-transform':
            return `node ${error.node} has an invalid transform${error.transformReason ? ` (${error.transformReason})` : ''}`;
        case 'asset-kind-mismatch':
            return `node ${error.node} expected ${error.expectedKind}, found ${error.actualKind}`;
    }
}
/**
 * Turn an authoritative `SceneValidationReport` (produced by Rust validation of a
 * proposal's draft) into an accept/reject readout the UI can display. The UI never
 * decides acceptance — it reflects this.
 */
export function summarizeValidation(report) {
    const issues = report.errors.map((error) => ({
        code: error.code,
        node: error.node,
        detail: describeValidationError(error),
    }));
    return { accepted: issues.length === 0, issues };
}
//# sourceMappingURL=scene-authoring.js.map