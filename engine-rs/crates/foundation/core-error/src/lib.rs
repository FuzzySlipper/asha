//! Shared error envelope for the ASHA workspace.
//!
//! # Lane
//!
//! `rust-foundation` — `std`-only, zero external dependencies, no knowledge of
//! state, protocol, render, services, or TypeScript.
//!
//! # Design
//!
//! [`AshaError`] is a small, boring error envelope: a broad [`ErrorCategory`]
//! plus a human-readable message. It exists so crates that don't need a bespoke
//! typed error can share one consistent shape, and so orchestration/tools can
//! switch on a stable category without parsing prose.
//!
//! # Non-goals
//!
//! This crate does **not** model domain-specific failures (validation reasons,
//! decode errors, etc.) — those stay as rich typed enums in their owning crates
//! (e.g. `sim-validator::ValidationError`). It carries no backtrace, no error
//! source chain, and no `From` conversions for foreign error types, to keep the
//! foundation dependency-free and unopinionated.

#![forbid(unsafe_code)]

/// A broad, stable classification of a failure. Tools and orchestration can
/// match on this without reading the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorCategory {
    /// The input was malformed or violated an invariant.
    Invalid,
    /// A referenced thing does not exist.
    NotFound,
    /// The operation conflicts with current state (duplicate, already exists).
    Conflict,
    /// The operation is not supported (yet) in this configuration.
    Unsupported,
    /// An unexpected internal failure (a bug, not a user/input problem).
    Internal,
}

impl ErrorCategory {
    /// A short, stable, lowercase label suitable for logs and routing.
    pub fn label(self) -> &'static str {
        match self {
            ErrorCategory::Invalid => "invalid",
            ErrorCategory::NotFound => "not-found",
            ErrorCategory::Conflict => "conflict",
            ErrorCategory::Unsupported => "unsupported",
            ErrorCategory::Internal => "internal",
        }
    }
}

/// A small shared error: a category plus a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AshaError {
    category: ErrorCategory,
    message: String,
}

impl AshaError {
    /// Construct an error with an explicit category and message.
    pub fn new(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Invalid, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::NotFound, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Conflict, message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Unsupported, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Internal, message)
    }

    pub fn category(&self) -> ErrorCategory {
        self.category
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for AshaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.category.label(), self.message)
    }
}

impl std::error::Error for AshaError {}

/// Convenience alias for fallible operations that use [`AshaError`].
pub type AshaResult<T> = Result<T, AshaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_set_category_and_message() {
        let e = AshaError::not_found("entity 7");
        assert_eq!(e.category(), ErrorCategory::NotFound);
        assert_eq!(e.message(), "entity 7");
    }

    #[test]
    fn display_is_category_then_message() {
        assert_eq!(
            AshaError::conflict("already exists").to_string(),
            "conflict: already exists"
        );
        assert_eq!(AshaError::invalid("bad").to_string(), "invalid: bad");
    }

    #[test]
    fn category_labels_are_stable_and_distinct() {
        let cats = [
            ErrorCategory::Invalid,
            ErrorCategory::NotFound,
            ErrorCategory::Conflict,
            ErrorCategory::Unsupported,
            ErrorCategory::Internal,
        ];
        let mut labels: Vec<&str> = cats.iter().map(|c| c.label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), count, "labels must be unique");
    }

    #[test]
    fn usable_as_std_error() {
        fn take(_e: &dyn std::error::Error) {}
        take(&AshaError::internal("boom"));
    }

    #[test]
    fn result_alias_roundtrips() {
        fn fallible(ok: bool) -> AshaResult<u32> {
            if ok {
                Ok(3)
            } else {
                Err(AshaError::invalid("nope"))
            }
        }
        assert_eq!(fallible(true).unwrap(), 3);
        assert_eq!(
            fallible(false).unwrap_err().category(),
            ErrorCategory::Invalid
        );
    }
}
