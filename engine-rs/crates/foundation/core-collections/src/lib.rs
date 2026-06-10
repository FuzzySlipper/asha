//! Deterministic ordering helpers for the ASHA workspace.
//!
//! # Lane
//!
//! `rust-foundation` — `std`-only, zero external dependencies, no knowledge of
//! state, protocol, render, services, or TypeScript.
//!
//! # Design
//!
//! Determinism in ASHA depends on stable, sorted iteration order (the state
//! hash, snapshots, and replay all rely on it). These are small free functions
//! that make "keep this collection sorted and unique" a single, tested
//! operation instead of an ad-hoc `sort` + `dedup` re-implemented per crate.
//!
//! # Non-goals
//!
//! No new generic collection *types* or framework-style container abstractions —
//! `std`'s `Vec`, `BTreeMap`, and `BTreeSet` already give deterministic order.
//! This crate only offers boring helpers over slices and `Vec`s.

#![forbid(unsafe_code)]

/// Collect `items`, sort ascending, and remove duplicates. Deterministic
/// regardless of input order.
pub fn sorted_unique<T: Ord>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut v: Vec<T> = items.into_iter().collect();
    v.sort();
    v.dedup();
    v
}

/// Whether `slice` is in strictly ascending order — i.e. sorted *and*
/// duplicate-free. Useful as a debug/test invariant on ordered collections.
pub fn is_strictly_sorted<T: Ord>(slice: &[T]) -> bool {
    slice.windows(2).all(|w| w[0] < w[1])
}

/// Insert `item` into a strictly-sorted `vec`, preserving sorted order and
/// uniqueness. Returns `true` if inserted, `false` if `item` was already present.
///
/// `vec` must already be strictly sorted; the insertion uses binary search.
pub fn insert_sorted_unique<T: Ord>(vec: &mut Vec<T>, item: T) -> bool {
    match vec.binary_search(&item) {
        Ok(_) => false,
        Err(pos) => {
            vec.insert(pos, item);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_unique_sorts_and_dedups() {
        assert_eq!(sorted_unique([3, 1, 2, 3, 1]), vec![1, 2, 3]);
        assert_eq!(sorted_unique(Vec::<u8>::new()), Vec::<u8>::new());
    }

    #[test]
    fn sorted_unique_is_order_independent() {
        assert_eq!(
            sorted_unique([5, 1, 4, 2, 3]),
            sorted_unique([3, 2, 4, 1, 5])
        );
    }

    #[test]
    fn strictly_sorted_detects_order_and_duplicates() {
        assert!(is_strictly_sorted(&[1, 2, 3]));
        assert!(is_strictly_sorted::<u8>(&[]));
        assert!(is_strictly_sorted(&[7]));
        assert!(!is_strictly_sorted(&[1, 1, 2])); // duplicate
        assert!(!is_strictly_sorted(&[3, 1, 2])); // unsorted
    }

    #[test]
    fn insert_sorted_unique_keeps_invariant() {
        let mut v = vec![1, 3, 5];

        assert!(insert_sorted_unique(&mut v, 4));
        assert_eq!(v, vec![1, 3, 4, 5]);
        assert!(is_strictly_sorted(&v));

        assert!(insert_sorted_unique(&mut v, 0));
        assert_eq!(v, vec![0, 1, 3, 4, 5]);

        // Already present → no-op, reports false.
        assert!(!insert_sorted_unique(&mut v, 3));
        assert_eq!(v, vec![0, 1, 3, 4, 5]);
        assert!(is_strictly_sorted(&v));
    }
}
