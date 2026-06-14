//! Classified import diagnostics.
//!
//! The importer never silently drops an unsupported source feature or a missing
//! resource: every such case is a typed [`ImportDiagnostic`] with a stable code, a
//! source locus, and a suggested remedy, so an agent can route on the variant
//! rather than parse prose. Errors are fatal (the import is refused); warnings are
//! advisory (the import proceeds with the noted caveat).

/// Severity of an import diagnostic. Only `Error` refuses the import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportSeverity {
    Warning,
    Error,
}

impl ImportSeverity {
    pub fn label(self) -> &'static str {
        match self {
            ImportSeverity::Warning => "warning",
            ImportSeverity::Error => "error",
        }
    }
}

/// A stable, machine-routable import diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportCode {
    /// The source declared a schema version the importer does not support.
    UnsupportedSchema,
    /// The source could not be parsed as the documented format.
    MalformedSource,
    /// A recognised-but-unsupported source feature was present (e.g. animations).
    UnsupportedFeature,
    /// Only triangle-list topology is supported; the index count is not a multiple of 3.
    UnsupportedTopology,
    /// A vertex attribute stream length disagrees with the vertex count.
    AttributeLengthMismatch,
    /// A material referenced a texture that the source does not provide.
    MissingTexture,
    /// Two declared assets resolved to the same generated asset id.
    DuplicateAssetId,
    /// A mesh group referenced a material slot that no material declares.
    GroupSlotUnbound,
    /// The generated static-mesh descriptor failed border validation.
    InvalidDescriptor,
    /// A re-import would overwrite existing output whose source fingerprint changed.
    SourceFingerprintChanged,
}

impl ImportCode {
    pub fn label(self) -> &'static str {
        match self {
            ImportCode::UnsupportedSchema => "unsupportedSchema",
            ImportCode::MalformedSource => "malformedSource",
            ImportCode::UnsupportedFeature => "unsupportedFeature",
            ImportCode::UnsupportedTopology => "unsupportedTopology",
            ImportCode::AttributeLengthMismatch => "attributeLengthMismatch",
            ImportCode::MissingTexture => "missingTexture",
            ImportCode::DuplicateAssetId => "duplicateAssetId",
            ImportCode::GroupSlotUnbound => "groupSlotUnbound",
            ImportCode::InvalidDescriptor => "invalidDescriptor",
            ImportCode::SourceFingerprintChanged => "sourceFingerprintChanged",
        }
    }
}

/// One classified import diagnostic.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDiagnostic {
    pub severity: ImportSeverity,
    pub code: ImportCode,
    /// The source locus (a source path, field, or asset id) the diagnostic points at.
    pub locus: String,
    /// A human-readable, agent-legible message.
    pub message: String,
    /// A suggested remedy.
    pub remedy: String,
}

impl ImportDiagnostic {
    pub fn error(
        code: ImportCode,
        locus: impl Into<String>,
        message: impl Into<String>,
        remedy: impl Into<String>,
    ) -> Self {
        ImportDiagnostic {
            severity: ImportSeverity::Error,
            code,
            locus: locus.into(),
            message: message.into(),
            remedy: remedy.into(),
        }
    }

    pub fn warning(
        code: ImportCode,
        locus: impl Into<String>,
        message: impl Into<String>,
        remedy: impl Into<String>,
    ) -> Self {
        ImportDiagnostic {
            severity: ImportSeverity::Warning,
            code,
            locus: locus.into(),
            message: message.into(),
            remedy: remedy.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == ImportSeverity::Error
    }

    /// Deterministic one-line rendering for golden fixtures and CLI output.
    pub fn render(&self) -> String {
        format!(
            "{} [{}] {}: {} (remedy: {})",
            self.severity.label(),
            self.code.label(),
            self.locus,
            self.message,
            self.remedy
        )
    }
}
