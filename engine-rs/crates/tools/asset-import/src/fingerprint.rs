//! Deterministic content fingerprints for source files and generated artifacts.
//!
//! Uses FNV-1a (64-bit), the workspace's standard deterministic hash, rendered as a
//! 16-char lowercase-hex string — the form [`core_assets::AssetHash`] accepts. The
//! seam is algorithm-agnostic; if a stronger digest is wanted later, only this
//! module changes.

use core_assets::AssetHash;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64-bit hash of arbitrary bytes.
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// A 16-char lowercase-hex fingerprint of `bytes`.
pub fn fingerprint_hex(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a_64(bytes))
}

/// A fingerprint as a validated [`AssetHash`] (always valid: 16 lowercase-hex chars).
pub fn fingerprint_hash(bytes: &[u8]) -> AssetHash {
    AssetHash::parse(&fingerprint_hex(bytes)).expect("16-char lowercase hex is a valid AssetHash")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_deterministic_and_well_formed() {
        let a = fingerprint_hex(b"hello");
        let b = fingerprint_hex(b"hello");
        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
        assert!(a
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)));
        // Distinct input → distinct hash (overwhelmingly).
        assert_ne!(fingerprint_hex(b"hello"), fingerprint_hex(b"world"));
    }

    #[test]
    fn parses_as_an_asset_hash() {
        let h = fingerprint_hash(b"content");
        assert_eq!(h.as_str().len(), 16);
    }
}
