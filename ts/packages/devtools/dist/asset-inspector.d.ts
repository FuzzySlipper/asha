import type { AssetKind, Catalog, CatalogEntry, CatalogValidationReport, CollisionMaterial, FallbackDecision, LockIssueCode, LockValidationReport, RenderMaterial } from '@asha/contracts';
/** One catalog entry as displayed: identity, kind, and resolved dependency ids. */
export interface CatalogEntryView {
    readonly id: string;
    readonly kind: AssetKind;
    readonly version: number;
    readonly label: string | null;
    readonly hasMaterial: boolean;
    /** Dependency asset ids, in declared order. */
    readonly dependencies: readonly string[];
    /** Dependency ids that are not present in the catalog (classified, not dropped). */
    readonly missingDependencies: readonly string[];
}
/** A classified catalog readout derived from a generated validation report. */
export interface CatalogStructuralIssue {
    readonly code: CatalogValidationReport['errors'][number]['code'];
    readonly id: string | null;
    readonly detail: string;
    /** Non-empty only for dependency-cycle. */
    readonly cyclePath: readonly string[];
}
export interface CatalogModel {
    readonly entries: readonly CatalogEntryView[];
    /** Adjacency: asset id → dependency ids present in the catalog. */
    readonly dependencyEdges: ReadonlyMap<string, readonly string[]>;
    /** Cycles detected over present dependencies (each path starts at its lowest id). */
    readonly cycles: readonly (readonly string[])[];
    readonly structuralIssues: readonly CatalogStructuralIssue[];
}
/**
 * Build the catalog inspector model: per-entry views, the dependency DAG over
 * present assets, detected cycles, and classified structural issues from a
 * generated validation report (when one is supplied).
 */
export declare function buildCatalogModel(catalog: Catalog, validation?: CatalogValidationReport): CatalogModel;
/** A lock finding's display severity. `new-in-catalog` is informational. */
export type LockDriftSeverity = 'info' | 'warning' | 'drift';
export interface LockFindingView {
    readonly id: string;
    readonly code: LockIssueCode;
    readonly severity: LockDriftSeverity;
    readonly detail: string;
}
export interface LockDriftModel {
    readonly findings: readonly LockFindingView[];
    /** True when any finding is more than informational — a save must not silently relock. */
    readonly hasDrift: boolean;
}
/** Build the lock-drift inspector model from a generated lock validation report. */
export declare function buildLockDriftModel(report: LockValidationReport): LockDriftModel;
/**
 * The material inspector view. The two projections are exposed as separate read
 * objects so a UI cannot present (or edit) them as one mixed material: the pure
 * render path consumes only `render`, authority consumes only `collision`.
 */
export interface MaterialInspection {
    readonly render: RenderMaterial;
    readonly collision: CollisionMaterial;
}
/** Inspect a catalog entry's material projection, or null for a non-material asset. */
export declare function inspectMaterial(entry: CatalogEntry): MaterialInspection | null;
export interface FallbackReadout {
    readonly outcome: FallbackDecision['outcome'];
    readonly reason: string;
    /** The concrete placeholder, present only when a fallback is actually used. */
    readonly visual: string | null;
    /** True only for the `useFallback` outcome — a missing asset is being substituted. */
    readonly fallbackUsed: boolean;
}
/** Classify a fallback decision for display (never authorizes a substitution). */
export declare function classifyFallback(decision: FallbackDecision): FallbackReadout;
export interface ImpactReport {
    readonly changed: string;
    /** Catalog entries that depend (transitively) on the changed asset. */
    readonly dependents: readonly string[];
    /** True when the changed id is not present in the catalog. */
    readonly unknownAsset: boolean;
}
/**
 * Report which catalog entries are impacted by a change to `changedId` — every
 * asset that transitively depends on it. Pure; reads the catalog's declared
 * dependency edges and never mutates.
 */
export declare function impactOfChangedAsset(catalog: Catalog, changedId: string): ImpactReport;
//# sourceMappingURL=asset-inspector.d.ts.map