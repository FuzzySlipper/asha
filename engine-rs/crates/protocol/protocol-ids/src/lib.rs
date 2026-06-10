//! Branded border IDs for the ASHA generated-contract boundary.
//!
//! # Lane
//!
//! `contract-steward` — owns the *shape* of identifiers that cross the Rust ↔
//! TypeScript border. May depend only on `core-ids`.
//!
//! # Border ownership
//!
//! The authority core defines its identifiers in `core-ids` as `Copy` newtypes
//! over `u64`. Those types are an internal Rust concern. This crate declares the
//! subset of those identifiers that are *promised to the TypeScript side* and
//! the canonical brand name each one carries in generated contracts.
//!
//! Keeping that promise in one ordered, machine-readable table ([`BORDER_IDS`])
//! is what makes Phase 2 codegen deterministic: the generator walks this table
//! rather than guessing which `core-ids` types are public API.
//!
//! # Forbidden convenience logic
//!
//! This crate must not parse, validate, allocate, or remap IDs. It does not own
//! a registry of live IDs and it performs no conversion beyond naming. Anything
//! that *does* something with an ID belongs in the authority core, not at the
//! border.

#![forbid(unsafe_code)]

/// The authority-core representation a branded border ID maps onto.
///
/// Today every ASHA identifier is a 64-bit unsigned integer; the enum exists so
/// that codegen has an explicit repr to switch on rather than an implied one,
/// and so a future non-`u64` brand is an additive change here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdRepr {
    /// Wraps a 64-bit unsigned integer in the authority core.
    U64,
}

/// Declares one branded identifier that crosses the generated-contract border.
///
/// The `brand` is the exact name used on *both* sides of the border: the
/// `core-ids` Rust newtype and the generated TypeScript branded type. The
/// `brand_names_match_core_ids_types` test proves the two never drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderId {
    /// Brand name, identical in Rust and generated TypeScript (e.g. `"EntityId"`).
    pub brand: &'static str,
    /// Underlying authority-core representation.
    pub repr: IdRepr,
}

/// Canonical, ordered set of branded IDs the protocol layer exposes to TS.
///
/// Order is significant: codegen emits TypeScript brands in this order so the
/// generated file is byte-stable across runs. Append new brands at the end.
pub const BORDER_IDS: &[BorderId] = &[
    BorderId {
        brand: "EntityId",
        repr: IdRepr::U64,
    },
    BorderId {
        brand: "SubjectId",
        repr: IdRepr::U64,
    },
    BorderId {
        brand: "ProcessId",
        repr: IdRepr::U64,
    },
    BorderId {
        brand: "ModeId",
        repr: IdRepr::U64,
    },
    BorderId {
        brand: "SignalId",
        repr: IdRepr::U64,
    },
    BorderId {
        brand: "TagId",
        repr: IdRepr::U64,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use core_ids::{EntityId, ModeId, ProcessId, SignalId, SubjectId, TagId};

    /// The `Debug` output of a `core-ids` newtype is `"<Brand>(<raw>)"`; this
    /// returns the leading brand portion so a test can compare it to
    /// [`BorderId::brand`] without depending on the raw value.
    fn debug_brand(s: &str) -> &str {
        s.split('(').next().unwrap()
    }

    #[test]
    fn border_id_table_is_unique_and_ordered_as_written() {
        let brands: Vec<&str> = BORDER_IDS.iter().map(|b| b.brand).collect();
        assert_eq!(
            brands,
            [
                "EntityId",
                "SubjectId",
                "ProcessId",
                "ModeId",
                "SignalId",
                "TagId"
            ],
        );
        let mut deduped = brands.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(deduped.len(), brands.len(), "brand names must be unique");
    }

    /// Proves each declared brand still matches the name of its `core-ids`
    /// newtype. If someone renames `EntityId` in `core-ids`, this fails and
    /// forces the border table to be updated in lockstep.
    #[test]
    fn brand_names_match_core_ids_types() {
        assert_eq!(debug_brand(&format!("{:?}", EntityId::new(0))), "EntityId");
        assert_eq!(
            debug_brand(&format!("{:?}", SubjectId::new(0))),
            "SubjectId"
        );
        assert_eq!(
            debug_brand(&format!("{:?}", ProcessId::new(0))),
            "ProcessId"
        );
        assert_eq!(debug_brand(&format!("{:?}", ModeId::new(0))), "ModeId");
        assert_eq!(debug_brand(&format!("{:?}", SignalId::new(0))), "SignalId");
        assert_eq!(debug_brand(&format!("{:?}", TagId::new(0))), "TagId");

        let declared: Vec<&str> = BORDER_IDS.iter().map(|b| b.brand).collect();
        for ty in [
            "EntityId",
            "SubjectId",
            "ProcessId",
            "ModeId",
            "SignalId",
            "TagId",
        ] {
            assert!(declared.contains(&ty), "{ty} must appear in BORDER_IDS");
        }
    }

    #[test]
    fn every_border_id_is_u64_today() {
        assert!(BORDER_IDS.iter().all(|b| b.repr == IdRepr::U64));
    }
}
