//! Renderer resource diagnostics: turn an observational
//! [`RendererResourceReport`] into a summary plus leak/fallback hints.

use protocol_diagnostics::{
    DiagnosticCode, DiagnosticReport, DiagnosticReportSet, DiagnosticSourceRef, RemedyAction,
    RendererResourceReport, SuggestedRemedy,
};

/// Emit observational diagnostics for a renderer resource report: always an
/// `Info` summary, a `Warning` when created-vs-disposed accounting suggests a
/// leak, and a `Warning` when fallback materials are in use. The report is an
/// observation — these diagnostics never imply the renderer has authority.
pub fn resource_diagnostics(report: &RendererResourceReport) -> DiagnosticReportSet {
    let mut set = DiagnosticReportSet::new();

    set.push(DiagnosticReport::new(
        DiagnosticCode::RendererResourceSummary,
        "renderer",
        DiagnosticSourceRef::empty(),
        format!(
            "handles={} geometries={} materials={} sprites={} created={} disposed={}",
            report.live_handles,
            report.geometries,
            report.materials,
            report.sprite_instances,
            report.resources_created,
            report.resources_disposed
        ),
    ));

    if report.suspects_leak() {
        set.push(
            DiagnosticReport::new(
                DiagnosticCode::SuspectedResourceLeak,
                "renderer",
                DiagnosticSourceRef::empty(),
                format!(
                    "renderer created {} resources but disposed {} ({} outstanding)",
                    report.resources_created,
                    report.resources_disposed,
                    report.outstanding_resources()
                ),
            )
            .with_remedy(SuggestedRemedy::new(
                RemedyAction::Inspect,
                "check geometry/material disposal on destroy",
            )),
        );
    }

    if report.fallback_materials > 0 {
        set.push(
            DiagnosticReport::new(
                DiagnosticCode::FallbackUsed,
                "renderer",
                DiagnosticSourceRef::empty(),
                format!(
                    "{} fallback material(s)/texture(s) currently substituted",
                    report.fallback_materials
                ),
            )
            .with_remedy(SuggestedRemedy::new(
                RemedyAction::ProvideAsset,
                "provide the real materials, or accept the fallbacks",
            )),
        );
    }

    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_report_is_just_a_summary() {
        let report = RendererResourceReport {
            live_handles: 2,
            geometries: 1,
            materials: 1,
            sprite_instances: 1,
            sprites_updated_last_tick: 1,
            resources_created: 4,
            resources_disposed: 4,
            fallback_materials: 0,
        };
        let set = resource_diagnostics(&report);
        assert_eq!(set.reports.len(), 1);
        assert_eq!(set.reports[0].code, DiagnosticCode::RendererResourceSummary);
        assert!(!set.blocks_load());
    }

    #[test]
    fn leak_and_fallback_add_warnings() {
        let report = RendererResourceReport {
            live_handles: 3,
            geometries: 2,
            materials: 2,
            sprite_instances: 0,
            sprites_updated_last_tick: 0,
            resources_created: 10,
            resources_disposed: 7,
            fallback_materials: 1,
        };
        let set = resource_diagnostics(&report);
        assert!(set
            .reports
            .iter()
            .any(|r| r.code == DiagnosticCode::SuspectedResourceLeak));
        assert!(set
            .reports
            .iter()
            .any(|r| r.code == DiagnosticCode::FallbackUsed));
        assert!(!set.blocks_load());
    }
}
