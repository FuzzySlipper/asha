//! Asset-catalog diagnostics: map `core-catalog` validation and asset-lock
//! drift into stable diagnostic reports.

use core_catalog::{
    validate, AssetLock, Catalog, CatalogValidationError, LockIssue, LockValidationReport,
};
use protocol_diagnostics::{
    DiagnosticCode, DiagnosticReport, DiagnosticReportSet, DiagnosticSeverity, DiagnosticSourceRef,
    RemedyAction, SuggestedRemedy,
};

/// Emit diagnostics for a catalog by running `core-catalog` validation and
/// classifying each error. Read-only.
pub fn catalog_diagnostics(catalog: &Catalog) -> DiagnosticReportSet {
    let mut set = DiagnosticReportSet::new();
    for error in validate(catalog).errors {
        set.push(map_catalog_error(&error));
    }
    set
}

fn map_catalog_error(error: &CatalogValidationError) -> DiagnosticReport {
    match error {
        CatalogValidationError::DuplicateAssetId { id } => DiagnosticReport::new(
            DiagnosticCode::DuplicateAssetId,
            id.as_str(),
            DiagnosticSourceRef::empty().with_asset(id.as_str()),
            format!("two catalog entries share asset id `{}`", id.as_str()),
        )
        .with_remedy(SuggestedRemedy::new(
            RemedyAction::FixReference,
            "give each catalog entry a unique id",
        )),
        CatalogValidationError::MaterialPayloadMissing { id } => structural(
            id.as_str(),
            format!("material `{}` is missing its material payload", id.as_str()),
        ),
        CatalogValidationError::MaterialPayloadOnNonMaterial { id, kind } => structural(
            id.as_str(),
            format!(
                "`{}` is a `{kind}` but carries a material payload",
                id.as_str()
            ),
        ),
        CatalogValidationError::EmptySourcePath { id } => structural(
            id.as_str(),
            format!("`{}` has an empty source path", id.as_str()),
        ),
        CatalogValidationError::WrongKindReference {
            from,
            slot,
            expected,
            actual,
            reference,
        } => DiagnosticReport::new(
            DiagnosticCode::WrongKindAssetRef,
            from.as_str(),
            DiagnosticSourceRef::empty().with_asset(reference.as_str()),
            format!(
                "`{}` slot `{slot}` expects a `{expected}` but references `{}` (a `{actual}`)",
                from.as_str(),
                reference.as_str()
            ),
        )
        .with_remedy(SuggestedRemedy::new(
            RemedyAction::FixReference,
            "point the typed slot at an asset of the expected kind",
        )),
        CatalogValidationError::UnknownDependency { from, dependency } => DiagnosticReport::new(
            DiagnosticCode::MissingAsset,
            dependency.as_str(),
            DiagnosticSourceRef::empty().with_asset(dependency.as_str()),
            format!(
                "`{}` depends on `{}` which is not present in the catalog",
                from.as_str(),
                dependency.as_str()
            ),
        )
        .with_remedy(SuggestedRemedy::new(
            RemedyAction::ProvideAsset,
            "add the missing dependency to the catalog",
        )),
        CatalogValidationError::DependencyCycle { path } => {
            let chain = path
                .iter()
                .map(|id| id.as_str().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            let first = path.first().map(|id| id.as_str()).unwrap_or("");
            DiagnosticReport::new(
                DiagnosticCode::AssetCycle,
                first,
                DiagnosticSourceRef::empty().with_asset(first),
                format!("asset dependency cycle: {chain}"),
            )
            .with_remedy(SuggestedRemedy::new(
                RemedyAction::BreakCycle,
                "remove one dependency edge in the reported cycle",
            ))
        }
    }
}

fn structural(asset: &str, message: String) -> DiagnosticReport {
    DiagnosticReport::new(
        DiagnosticCode::CatalogStructuralError,
        asset,
        DiagnosticSourceRef::empty().with_asset(asset),
        message,
    )
    .with_remedy(SuggestedRemedy::new(
        RemedyAction::Inspect,
        "fix the catalog entry's structural payload",
    ))
}

/// Emit diagnostics for asset-lock drift: re-validate `lock` against the current
/// `catalog` and classify each finding. Missing/wrong-kind are errors; version,
/// hash, and dependency drift are warnings. Read-only.
pub fn lock_diagnostics(lock: &AssetLock, catalog: &Catalog) -> DiagnosticReportSet {
    let report: LockValidationReport = core_catalog::validate_lock(lock, catalog);
    let mut set = DiagnosticReportSet::new();
    for finding in &report.findings {
        let id = finding.id.as_str();
        let src = DiagnosticSourceRef::empty().with_asset(id);
        let r = match &finding.issue {
            LockIssue::Missing => DiagnosticReport::new(
                DiagnosticCode::MissingAsset,
                id,
                src,
                format!("locked asset `{id}` is absent from the current catalog"),
            )
            .with_remedy(SuggestedRemedy::new(
                RemedyAction::ProvideAsset,
                "restore the asset or re-lock the bundle",
            )),
            LockIssue::WrongKind { locked, current } => DiagnosticReport::new(
                DiagnosticCode::WrongKindAssetRef,
                id,
                src,
                format!("`{id}` was locked as `{locked}` but the catalog now has `{current}`"),
            ),
            LockIssue::StaleVersion { locked, current } => DiagnosticReport::new(
                DiagnosticCode::StaleAsset,
                id,
                src,
                format!("`{id}` lock version {locked} differs from catalog version {current}"),
            )
            .with_remedy(SuggestedRemedy::new(
                RemedyAction::RefreshCache,
                "re-lock the bundle if the new version is intended",
            )),
            LockIssue::StaleHash { .. } => DiagnosticReport::new(
                DiagnosticCode::StaleAsset,
                id,
                src,
                format!("`{id}` content hash differs from the lock"),
            ),
            LockIssue::DependencyDrift { added, removed } => DiagnosticReport::new(
                DiagnosticCode::StaleAsset,
                id,
                src,
                format!(
                    "`{id}` dependencies drifted (+{} -{})",
                    added.len(),
                    removed.len()
                ),
            ),
            // A catalog asset the lock never pinned is informational, not drift.
            LockIssue::NewInCatalog => DiagnosticReport::new(
                DiagnosticCode::StaleAsset,
                id,
                src,
                format!("`{id}` is in the catalog but was not in the lock"),
            )
            .with_severity(DiagnosticSeverity::Info),
        };
        set.push(r);
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_assets::{AssetHash, AssetId, AssetReference, AssetVersionReq};
    use core_catalog::{generate_lock, CatalogEntry};

    fn id(s: &str) -> AssetId {
        AssetId::parse(s).unwrap()
    }
    fn dep(s: &str) -> AssetReference {
        AssetReference::new(id(s), AssetVersionReq::Any, None)
    }

    fn catalog() -> Catalog {
        let tex = CatalogEntry::new(id("texture/atlas-a"), 1)
            .with_hash(AssetHash::parse("aa01").unwrap());
        let mesh = CatalogEntry::new(id("mesh/fixture-a"), 1)
            .with_hash(AssetHash::parse("cc03").unwrap())
            .with_dependencies(vec![dep("texture/atlas-a")]);
        Catalog::from_entries(vec![tex, mesh])
    }

    #[test]
    fn valid_catalog_emits_nothing() {
        assert!(catalog_diagnostics(&catalog()).is_empty());
    }

    #[test]
    fn unknown_dependency_and_cycle_are_classified() {
        let mut c = catalog();
        c.entries.push(
            CatalogEntry::new(id("mesh/fixture-b"), 1)
                .with_dependencies(vec![dep("mesh/does-not-exist")]),
        );
        let set = catalog_diagnostics(&c);
        assert!(set
            .reports
            .iter()
            .any(|r| r.code == DiagnosticCode::MissingAsset));
    }

    #[test]
    fn lock_drift_maps_to_stale_warnings() {
        let c = catalog();
        let lock = generate_lock(&c);
        let mut drifted = c.clone();
        drifted.entries[1].version = 9; // mesh/fixture-a version bump
        let set = lock_diagnostics(&lock, &drifted);
        let stale = set
            .reports
            .iter()
            .find(|r| r.code == DiagnosticCode::StaleAsset)
            .expect("stale reported");
        assert_eq!(stale.severity, DiagnosticSeverity::Warning);
        assert!(!set.blocks_load());
    }

    #[test]
    fn missing_locked_asset_is_an_error() {
        let c = catalog();
        let lock = generate_lock(&c);
        let mut changed = c.clone();
        changed.entries.remove(1); // drop mesh/fixture-a
        let set = lock_diagnostics(&lock, &changed);
        assert!(set
            .reports
            .iter()
            .any(|r| r.code == DiagnosticCode::MissingAsset
                && r.severity == DiagnosticSeverity::Error));
    }
}
