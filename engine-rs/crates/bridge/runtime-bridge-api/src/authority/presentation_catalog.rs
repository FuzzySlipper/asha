use super::*;

pub(super) fn built_in_presentation_catalog() -> Catalog {
    Catalog::from_entries(vec![
        CatalogEntry::new(
            AssetId::parse("audio/asha-primary-fire-pulse")
                .expect("built-in audio asset id is valid"),
            1,
        )
        .with_hash(
            AssetHash::parse("9de44d49edeab1dba3c78b42a602d8d1c5dcf92f752638995adda894a5b3ccba")
                .expect("built-in audio asset hash is valid lowercase hex"),
        ),
        CatalogEntry::new(
            AssetId::parse("sprite/asha-primary-fire-spark")
                .expect("built-in particle sprite id is valid"),
            1,
        )
        .with_hash(
            AssetHash::parse("0541e102a0dc20342819a3fb9024de73f3249269fed374b68c6aa8fc5dd2f5c1")
                .expect("built-in particle sprite hash is valid lowercase hex"),
        ),
    ])
}
