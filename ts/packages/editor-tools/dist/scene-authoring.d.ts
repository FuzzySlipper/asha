import type { AssetReference, FlatSceneDocument, SceneNodeId, SceneNodeRecord, SceneTransform, SceneValidationCode, SceneValidationReport } from '@asha/contracts';
/**
 * One proposed scene edit, expressed over generated contract types. A proposal is
 * the thing a UI submits to authority; authority accepts or rejects it. There is
 * no proposal that mutates state directly.
 */
export type SceneEditProposal = {
    readonly op: 'addNode';
    readonly node: SceneNodeRecord;
} | {
    readonly op: 'reparent';
    readonly node: SceneNodeId;
    readonly newParent: SceneNodeId | null;
    readonly childOrder: number;
} | {
    readonly op: 'setTransform';
    readonly node: SceneNodeId;
    readonly transform: SceneTransform;
} | {
    readonly op: 'setMetadata';
    readonly node: SceneNodeId;
    readonly label: string | null;
    readonly tags: readonly string[];
};
/** Optional authored fields shared by the add-node builders. */
export interface NodeAuthoring {
    readonly parent?: SceneNodeId | null;
    readonly childOrder?: number;
    readonly label?: string | null;
    readonly tags?: readonly string[];
    readonly transform?: SceneTransform;
}
/** Propose adding a static-mesh node bound to a catalog mesh asset. */
export declare function proposeAddStaticMesh(id: SceneNodeId, asset: AssetReference, authoring?: NodeAuthoring): SceneEditProposal;
/** Propose adding a sprite node bound to a catalog sprite asset. */
export declare function proposeAddSprite(id: SceneNodeId, asset: AssetReference, authoring?: NodeAuthoring): SceneEditProposal;
/** Propose adding an empty grouping/transform node. */
export declare function proposeAddGroup(id: SceneNodeId, authoring?: NodeAuthoring): SceneEditProposal;
/** Propose reparenting (or grouping) a node under a new parent at a sibling index. */
export declare function proposeReparent(node: SceneNodeId, newParent: SceneNodeId | null, childOrder?: number): SceneEditProposal;
/** Propose replacing a node's initial transform. */
export declare function proposeSetTransform(node: SceneNodeId, transform: SceneTransform): SceneEditProposal;
/** Propose replacing a node's debug label and tags (never authority semantics). */
export declare function proposeSetMetadata(node: SceneNodeId, label: string | null, tags?: readonly string[]): SceneEditProposal;
/**
 * Apply a proposal to a copy of `doc`, producing a UI-local **draft** for preview
 * and for handing to authority validation. Pure — returns a new document and never
 * mutates the input. The draft is not authority truth; only a validated, replayed
 * proposal becomes truth.
 *
 * Returns the unchanged document (structurally copied) when the proposal targets a
 * node that is absent — authority will reject it; the draft never invents the node.
 */
export declare function applyProposalToDraft(doc: FlatSceneDocument, proposal: SceneEditProposal): FlatSceneDocument;
/** One classified validation issue, lifted from the authoritative Rust report. */
export interface ProposalIssue {
    readonly code: SceneValidationCode;
    readonly node: SceneNodeId | null;
    readonly detail: string;
}
/** Whether authority accepted the proposal, plus any classified rejection reasons. */
export interface ProposalFeedback {
    readonly accepted: boolean;
    readonly issues: readonly ProposalIssue[];
}
/**
 * Turn an authoritative `SceneValidationReport` (produced by Rust validation of a
 * proposal's draft) into an accept/reject readout the UI can display. The UI never
 * decides acceptance — it reflects this.
 */
export declare function summarizeValidation(report: SceneValidationReport): ProposalFeedback;
//# sourceMappingURL=scene-authoring.d.ts.map