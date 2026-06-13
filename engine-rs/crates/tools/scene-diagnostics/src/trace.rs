//! Render projection source traces: render handle → scene node → runtime entity
//! → asset ref, and the broken-trace / fallback diagnostics they warrant.
//!
//! The renderer's live handle registry lives in TypeScript (`renderer-three`),
//! so the Rust side works over a caller-assembled [`ProjectionRecord`]: the
//! observational facts about one render object. This keeps the trace logic and
//! its goldens deterministic and authority-free.

use protocol_diagnostics::{
    DiagnosticCode, DiagnosticReport, DiagnosticReportSet, DiagnosticSourceRef, RemedyAction,
    SourceTrace, SuggestedRemedy,
};

/// The observational facts about one render object, assembled by whatever holds
/// the render handle registry. Every projection hop is optional; `asset_resolved`
/// records whether `asset_id` resolved against the catalog, and `fallback_used`
/// records whether a fallback material/texture was substituted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionRecord {
    pub render_handle: u64,
    pub scene_node_id: Option<u64>,
    pub runtime_entity_id: Option<u64>,
    pub asset_id: Option<String>,
    pub asset_resolved: bool,
    pub fallback_used: bool,
}

impl ProjectionRecord {
    /// A complete, healthy projection record (every hop present, asset resolved).
    pub fn complete(
        render_handle: u64,
        scene_node_id: u64,
        runtime_entity_id: u64,
        asset_id: impl Into<String>,
    ) -> Self {
        Self {
            render_handle,
            scene_node_id: Some(scene_node_id),
            runtime_entity_id: Some(runtime_entity_id),
            asset_id: Some(asset_id.into()),
            asset_resolved: true,
            fallback_used: false,
        }
    }

    /// The [`SourceTrace`] this record projects to.
    pub fn to_trace(&self) -> SourceTrace {
        SourceTrace {
            render_handle: self.render_handle,
            scene_node_id: self.scene_node_id,
            runtime_entity_id: self.runtime_entity_id,
            asset_id: self.asset_id.clone(),
            asset_resolved: self.asset_resolved,
        }
    }
}

/// Build the source traces for a batch of projection records, in input order.
pub fn build_source_traces(records: &[ProjectionRecord]) -> Vec<SourceTrace> {
    records.iter().map(ProjectionRecord::to_trace).collect()
}

/// Emit diagnostics for broken source traces and substituted fallbacks. A
/// render object that cannot be traced back to a scene node, or that draws an
/// unresolved asset, gets a [`DiagnosticCode::MissingSourceTrace`]; a substituted
/// fallback gets a [`DiagnosticCode::FallbackUsed`]. Both are `Warning`s (the
/// object still renders, degraded). Read-only.
pub fn source_trace_diagnostics(records: &[ProjectionRecord]) -> DiagnosticReportSet {
    let mut set = DiagnosticReportSet::new();
    for record in records {
        let trace = record.to_trace();
        if trace.is_broken() {
            let mut src = DiagnosticSourceRef::empty().with_render_handle(record.render_handle);
            if let Some(node) = record.scene_node_id {
                src = src.with_scene_node(node);
            }
            if let Some(asset) = &record.asset_id {
                src = src.with_asset(asset.clone());
            }
            let why = if record.scene_node_id.is_none() {
                "no scene node".to_string()
            } else {
                format!(
                    "asset `{}` did not resolve",
                    record.asset_id.as_deref().unwrap_or("?")
                )
            };
            set.push(
                DiagnosticReport::new(
                    DiagnosticCode::MissingSourceTrace,
                    format!("handle:{}", record.render_handle),
                    src,
                    format!(
                        "render handle {} cannot be fully traced to authority: {why}",
                        record.render_handle
                    ),
                )
                .with_remedy(SuggestedRemedy::new(
                    RemedyAction::Inspect,
                    "attach scene-node / asset source metadata to the render object",
                )),
            );
        }
        if record.fallback_used {
            let mut src = DiagnosticSourceRef::empty().with_render_handle(record.render_handle);
            if let Some(asset) = &record.asset_id {
                src = src.with_asset(asset.clone());
            }
            set.push(
                DiagnosticReport::new(
                    DiagnosticCode::FallbackUsed,
                    format!("handle:{}", record.render_handle),
                    src,
                    format!(
                        "render handle {} drew a fallback material/texture for `{}`",
                        record.render_handle,
                        record.asset_id.as_deref().unwrap_or("?")
                    ),
                )
                .with_remedy(SuggestedRemedy::new(
                    RemedyAction::AcceptFallback,
                    "provide the real asset, or accept the fallback",
                )),
            );
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol_diagnostics::DiagnosticSeverity;

    #[test]
    fn complete_record_traces_cleanly() {
        let rec = ProjectionRecord::complete(42, 7, 123, "mesh/belt-straight");
        let trace = rec.to_trace();
        assert!(!trace.is_broken());
        assert!(source_trace_diagnostics(&[rec]).is_empty());
    }

    #[test]
    fn untraceable_handle_warns() {
        let rec = ProjectionRecord {
            render_handle: 43,
            scene_node_id: None,
            runtime_entity_id: None,
            asset_id: None,
            asset_resolved: false,
            fallback_used: false,
        };
        let set = source_trace_diagnostics(&[rec]);
        assert_eq!(set.reports.len(), 1);
        assert_eq!(set.reports[0].code, DiagnosticCode::MissingSourceTrace);
        assert_eq!(set.reports[0].severity, DiagnosticSeverity::Warning);
        assert!(!set.blocks_load());
    }

    #[test]
    fn unresolved_asset_warns_and_fallback_is_separate() {
        let rec = ProjectionRecord {
            render_handle: 44,
            scene_node_id: Some(8),
            runtime_entity_id: Some(456),
            asset_id: Some("sprite/hard-hat".to_string()),
            asset_resolved: false,
            fallback_used: true,
        };
        let set = source_trace_diagnostics(&[rec]);
        assert!(set
            .reports
            .iter()
            .any(|r| r.code == DiagnosticCode::MissingSourceTrace));
        assert!(set
            .reports
            .iter()
            .any(|r| r.code == DiagnosticCode::FallbackUsed));
    }
}
