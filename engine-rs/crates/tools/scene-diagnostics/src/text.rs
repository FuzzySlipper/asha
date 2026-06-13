//! Deterministic, greppable text rendering of diagnostics for goldens and
//! devtools/CLI readback. Pure: output depends only on the inputs and is stable
//! across runs.

use core::fmt::Write;

use protocol_diagnostics::{
    DiagnosticReport, DiagnosticReportSet, DiagnosticSourceRef, RendererResourceReport, SourceTrace,
};

/// Render a report set: a header line with aggregate policy, then one line per
/// report in order.
pub fn report_set_to_text(set: &DiagnosticReportSet) -> String {
    let mut s = String::new();
    let max = set.max_severity().map(|sev| sev.as_str()).unwrap_or("none");
    let _ = writeln!(
        s,
        "diagnostics count={} maxSeverity={} blocksLoad={}",
        set.reports.len(),
        max,
        set.blocks_load()
    );
    for report in &set.reports {
        let _ = writeln!(s, "{}", report_to_line(report));
    }
    s
}

/// One report as a single stable line.
pub fn report_to_line(report: &DiagnosticReport) -> String {
    let mut line = format!(
        "[{}] {}/{} ref={}{} :: {}",
        report.severity.as_str(),
        report.scope.as_str(),
        report.code.as_str(),
        report.reference,
        source_suffix(&report.source),
        report.message
    );
    if let Some(remedy) = &report.remedy {
        let _ = write!(
            line,
            " | remedy={}: {}",
            remedy.action.as_str(),
            remedy.detail
        );
    }
    line
}

/// The populated source-ref fields, in a fixed order, as ` key=value` pairs.
fn source_suffix(src: &DiagnosticSourceRef) -> String {
    let mut s = String::new();
    if let Some(node) = src.scene_node_id {
        let _ = write!(s, " sceneNode={node}");
    }
    if let Some(entity) = src.runtime_entity_id {
        let _ = write!(s, " entity={entity}");
    }
    if let Some(asset) = &src.asset_id {
        let _ = write!(s, " asset={asset}");
    }
    if let Some(chunk) = src.chunk_coord {
        let _ = write!(s, " chunk={:?}", chunk);
    }
    if let Some(handle) = src.render_handle {
        let _ = write!(s, " handle={handle}");
    }
    if let Some(path) = &src.bundle_path {
        let _ = write!(s, " bundlePath={path}");
    }
    s
}

/// Render a batch of source traces, one line each.
pub fn traces_to_text(traces: &[SourceTrace]) -> String {
    let mut s = String::new();
    for t in traces {
        let _ = writeln!(
            s,
            "trace handle={} sceneNode={} entity={} asset={} resolved={}{}",
            t.render_handle,
            opt_num(t.scene_node_id),
            opt_num(t.runtime_entity_id),
            t.asset_id.as_deref().unwrap_or("-"),
            t.asset_resolved,
            if t.is_broken() { " BROKEN" } else { "" }
        );
    }
    s
}

/// Render a renderer resource report as a single stable line.
pub fn resource_report_to_text(report: &RendererResourceReport) -> String {
    format!(
        "resources handles={} geometries={} materials={} sprites={} spritesUpdated={} created={} disposed={} outstanding={} fallbacks={} leak={}\n",
        report.live_handles,
        report.geometries,
        report.materials,
        report.sprite_instances,
        report.sprites_updated_last_tick,
        report.resources_created,
        report.resources_disposed,
        report.outstanding_resources(),
        report.fallback_materials,
        report.suspects_leak()
    )
}

fn opt_num(v: Option<u64>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol_diagnostics::{DiagnosticCode, DiagnosticReport, DiagnosticReportSet};

    #[test]
    fn report_line_is_stable_and_greppable() {
        let report = DiagnosticReport::new(
            DiagnosticCode::CorruptBundleArtifact,
            "chunks/0_0_0.snap",
            DiagnosticSourceRef::empty()
                .with_chunk([0, 0, 0])
                .with_bundle_path("chunks/0_0_0.snap"),
            "durable artifact failed its content hash",
        );
        let line = report_to_line(&report);
        assert_eq!(
            line,
            "[fatal] worldBundle/corruptBundleArtifact ref=chunks/0_0_0.snap chunk=[0, 0, 0] bundlePath=chunks/0_0_0.snap :: durable artifact failed its content hash"
        );
    }

    #[test]
    fn report_set_header_summarizes_policy() {
        let mut set = DiagnosticReportSet::new();
        set.push(DiagnosticReport::new(
            DiagnosticCode::StaleAsset,
            "mesh/a",
            DiagnosticSourceRef::empty().with_asset("mesh/a"),
            "older",
        ));
        let text = report_set_to_text(&set);
        assert!(text.starts_with("diagnostics count=1 maxSeverity=warning blocksLoad=false\n"));
    }

    #[test]
    fn empty_set_reports_none_severity() {
        let text = report_set_to_text(&DiagnosticReportSet::new());
        assert_eq!(
            text,
            "diagnostics count=0 maxSeverity=none blocksLoad=false\n"
        );
    }
}
