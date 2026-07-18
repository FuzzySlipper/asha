use super::*;

pub(super) fn extend_round_trip_coverage(coverage: &mut BTreeSet<String>) {
    coverage.extend([
        variant_coverage_key("assets", "StoredAssetVersionRequirement", "any"),
        variant_coverage_key("assets", "StoredAssetVersionRequirement", "exact"),
        variant_coverage_key("assets", "StoredAssetVersionRequirement", "atLeast"),
        interface_coverage_key("assets", "StoredAssetReference"),
        interface_coverage_key("assets", "StoredMaterialAuthority"),
        interface_coverage_key("assets", "StoredMaterialStyle"),
        interface_coverage_key("assets", "StoredMaterialDefinition"),
        interface_coverage_key("assets", "StoredCatalogEntry"),
        interface_coverage_key("assets", "StoredAssetCatalog"),
    ]);
}

#[test]
fn stored_asset_catalog_samples_match_generated_ir_shapes() {
    let assets = module("assets");
    for (tag, value) in [
        ("any", json!({ "req": "any" })),
        ("exact", json!({ "req": "exact", "value": 2 })),
        ("atLeast", json!({ "req": "atLeast", "value": 1 })),
    ] {
        compare_object_to_variant(&assets, "StoredAssetVersionRequirement", tag, &value).unwrap();
    }

    let reference = json!({
        "id": "texture/demo-wall",
        "version": { "req": "exact", "value": 2 },
        "hash": "sha256:texture"
    });
    let authority = json!({
        "solid": true,
        "collidable": true,
        "occludes": true,
        "structuralClass": "structural"
    });
    let rgba = json!({ "r": 1.0, "g": 0.5, "b": 0.25, "a": 1.0 });
    let style = json!({
        "color": rgba,
        "texture": reference,
        "roughness": 0.8,
        "textureTint": rgba,
        "emissionColor": rgba,
        "emissive": 0.0,
        "uvStrategy": "planar"
    });
    let material = json!({ "authority": authority, "style": style });
    let entry = json!({
        "id": "material/demo-wall",
        "version": 1,
        "hash": "sha256:material",
        "sourcePath": "assets/demo-wall.material.json",
        "label": "Demo wall",
        "dependencies": [reference],
        "material": material
    });
    let catalog = json!({ "entries": [entry] });

    compare_object_to_interface(&assets, "StoredAssetReference", &reference).unwrap();
    compare_object_to_interface(&assets, "StoredMaterialAuthority", &authority).unwrap();
    compare_object_to_interface(&assets, "StoredMaterialStyle", &style).unwrap();
    compare_object_to_interface(&assets, "StoredMaterialDefinition", &material).unwrap();
    compare_object_to_interface(&assets, "StoredCatalogEntry", &entry).unwrap();
    compare_object_to_interface(&assets, "StoredAssetCatalog", &catalog).unwrap();
}
