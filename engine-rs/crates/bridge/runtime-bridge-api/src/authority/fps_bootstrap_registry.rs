use super::*;

const FPS_BOOTSTRAP_REGISTRY_SCHEMA_VERSION: u32 = 1;

impl EngineBridge {
    pub(super) fn fps_bootstrap_resolution_context(
        registry: &FpsBootstrapResolutionRegistry,
    ) -> BridgeResult<core_scene::BootstrapResolutionContext> {
        if registry.schema_version != FPS_BOOTSTRAP_REGISTRY_SCHEMA_VERSION {
            return Err(registry_error(format!(
                "schemaVersion must be {FPS_BOOTSTRAP_REGISTRY_SCHEMA_VERSION}, got {}",
                registry.schema_version
            )));
        }

        let entity_definition_ids =
            unique_identifier_set("entityDefinitionIds", &registry.entity_definition_ids)?;
        let spawn_marker_ids = unique_identifier_set("spawnMarkerIds", &registry.spawn_marker_ids)?;
        let catalog_ids = unique_identifier_set("catalogIds", &registry.catalog_ids)?;

        let mut prefab_ids = BTreeSet::new();
        for (index, prefab_id) in registry.prefab_ids.iter().copied().enumerate() {
            if prefab_id == 0 {
                return Err(registry_error(format!(
                    "prefabIds[{index}] must be a positive prefab identity"
                )));
            }
            if !prefab_ids.insert(prefab_id) {
                return Err(registry_error(format!(
                    "prefabIds[{index}] duplicates prefab {prefab_id}"
                )));
            }
        }

        let mut generator_presets = BTreeSet::new();
        for (index, preset) in registry.generator_presets.iter().enumerate() {
            validate_registry_identifier(
                &format!("generatorPresets[{index}].providerId"),
                &preset.provider_id,
            )?;
            validate_registry_identifier(
                &format!("generatorPresets[{index}].presetId"),
                &preset.preset_id,
            )?;
            let identity = (preset.provider_id.clone(), preset.preset_id.clone());
            if !generator_presets.insert(identity) {
                return Err(registry_error(format!(
                    "generatorPresets[{index}] duplicates provider/preset {}/{}",
                    preset.provider_id, preset.preset_id
                )));
            }
        }

        Ok(core_scene::BootstrapResolutionContext {
            entity_definition_ids,
            prefab_ids,
            spawn_marker_ids,
            generator_presets,
            catalog_ids,
        })
    }
}

fn unique_identifier_set(label: &str, values: &[String]) -> BridgeResult<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        validate_registry_identifier(&format!("{label}[{index}]"), value)?;
        if !result.insert(value.clone()) {
            return Err(registry_error(format!(
                "{label}[{index}] duplicates identity {value}"
            )));
        }
    }
    Ok(result)
}

fn validate_registry_identifier(path: &str, value: &str) -> BridgeResult<()> {
    let valid = !value.is_empty()
        && value.trim() == value
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/' | b':' | b'@')
        });
    if valid {
        Ok(())
    } else {
        Err(registry_error(format!(
            "{path} must be a non-empty canonical identifier"
        )))
    }
}

fn registry_error(detail: String) -> RuntimeBridgeError {
    RuntimeBridgeError::new(
        RuntimeBridgeErrorKind::InvalidInput,
        format!("FPS bootstrap resolution registry rejected: {detail}"),
    )
}
