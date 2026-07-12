use napi_derive::napi;
use runtime_bridge_api::{
    FpsBridgeBoundsCapability, FpsBridgeHealth, FpsBridgePolicyBinding, FpsBridgeRole,
    FpsBridgeStoredEntityDefinition, FpsBridgeTransformCapability, FpsBridgeWeaponMount,
    FpsEncounterDirectorSnapshot, FpsEncounterLifecycleInput, FpsEncounterStateReadout,
    FpsEncounterTransitionRequest, FpsEncounterTransitionResult, FpsPrimaryFireRequest,
    FpsPrimaryFireResult, FpsRuntimeSessionLoadRequest, FpsRuntimeSessionRestartRequest,
    FpsRuntimeSessionSnapshot, GameExtensionWeaponEffectInvocationRequest,
    GameRuleEffectIntentRequest, RuntimeBridge, RuntimeBridgeError, RuntimeBridgeErrorKind,
};

use crate::{
    game_extension_json, game_rule_json, parse_game_rule_catalog, parse_game_rule_module_manifests,
    parse_game_rule_resolution_request, parse_weapon_effect_hook_request, to_napi, u64_input,
    with_bridge, NativeVec3,
};

#[napi(object)]
pub struct NativeFpsTransformCapability {
    pub translation: NativeVec3,
    pub rotation: Vec<f64>,
    pub scale: NativeVec3,
}

#[napi(object)]
pub struct NativeFpsBoundsCapability {
    pub min: NativeVec3,
    pub max: NativeVec3,
}

#[napi(object)]
pub struct NativeFpsHealth {
    pub current: u32,
    pub max: u32,
}

#[napi(object)]
pub struct NativeFpsWeaponMount {
    pub weapon_id: String,
    pub damage: u32,
    pub range_units: u32,
    pub ammo: u32,
    pub cooldown_ticks_after_fire: u32,
}

#[napi(object)]
pub struct NativeFpsPolicyBinding {
    pub binding_id: String,
    pub policy_id: String,
    pub view_kind: String,
    pub view_version: String,
    pub allowed_intents: Vec<String>,
    pub runtime_moment: String,
}

#[napi(object)]
pub struct NativeFpsStoredEntityDefinition {
    pub entity: i64,
    pub stable_id: String,
    pub display_name: String,
    pub source_path: String,
    pub tags: Vec<String>,
    pub role: String,
    pub transform: Option<NativeFpsTransformCapability>,
    pub bounds: Option<NativeFpsBoundsCapability>,
    pub render_visible: Option<bool>,
    pub static_collider: Option<bool>,
    pub health: Option<NativeFpsHealth>,
    pub weapon: Option<NativeFpsWeaponMount>,
    pub policy_binding: Option<NativeFpsPolicyBinding>,
}

#[napi(object)]
pub struct NativeFpsLifecycleStatus {
    pub state: String,
    pub entity: Option<i64>,
    pub tick: Option<i64>,
}

#[napi(object)]
pub struct NativeFpsEntityHealthReadout {
    pub entity: i64,
    pub current: u32,
    pub max: u32,
}

#[napi(object)]
pub struct NativeFpsPolicyBindingReadout {
    pub entity: i64,
    pub binding_id: String,
    pub policy_id: String,
    pub view_kind: String,
    pub view_version: String,
    pub allowed_intents: Vec<String>,
    pub runtime_moment: String,
}

#[napi(object)]
pub struct NativeFpsReplayEvidence {
    pub replay_unit: String,
    pub entity_hash: String,
    pub health_hash: String,
    pub record_hash: String,
}

#[napi(object)]
pub struct NativeFpsReadSetEvidence {
    pub view_kind: String,
    pub owner: String,
    pub read_set: Vec<String>,
}

#[napi(object)]
pub struct NativeFpsRuntimeSessionSnapshot {
    pub backend: String,
    pub authority_surface: String,
    pub project_bundle: String,
    pub session_epoch: i64,
    pub lifecycle_status: NativeFpsLifecycleStatus,
    pub player_entity: i64,
    pub enemy_entity: i64,
    pub health: Vec<NativeFpsEntityHealthReadout>,
    pub policy_bindings: Vec<NativeFpsPolicyBindingReadout>,
    pub replay_records: Vec<NativeFpsReplayEvidence>,
    pub read_sets: Vec<NativeFpsReadSetEvidence>,
    pub entity_hash: String,
    pub health_hash: String,
    pub replay_hash: String,
}

#[napi(object)]
pub struct NativeFpsPrimaryFireResult {
    pub backend: String,
    pub authority_surface: String,
    pub mutation_owner: String,
    pub workspace_trace: Vec<String>,
    pub shooter: i64,
    pub target: Option<i64>,
    pub target_health_before: Option<NativeFpsHealth>,
    pub target_health_after: Option<NativeFpsHealth>,
    pub lifecycle_status: NativeFpsLifecycleStatus,
    pub target_render_visible: Option<bool>,
    pub entity_hash: String,
    pub health_hash: String,
    pub replay_hash: String,
}

#[napi(object)]
pub struct NativeGameExtensionWeaponEffectInvocationResult {
    pub hook_receipt_json: String,
    pub replay_evidence_json: String,
    pub primary_fire: Option<NativeFpsPrimaryFireResult>,
}

#[napi(object)]
pub struct NativeFpsEncounterLifecycleInput {
    pub outcome_kind: String,
    pub terminal: bool,
    pub enemy_dead: bool,
    pub player_dead: bool,
    pub lifecycle_hash: String,
}

#[napi(object)]
pub struct NativeFpsEncounterTransitionRequest {
    pub preset_id: String,
    pub action: String,
    pub lifecycle: NativeFpsEncounterLifecycleInput,
}

#[napi(object)]
pub struct NativeFpsEncounterStateReadout {
    pub preset_id: String,
    pub status: String,
    pub spawned_enemy_ids: Vec<String>,
    pub defeated_enemy_ids: Vec<String>,
    pub revision: i64,
    pub last_transition: String,
}

#[napi(object)]
pub struct NativeFpsEncounterDirectorSnapshot {
    pub backend: String,
    pub authority_surface: String,
    pub mutation_owner: String,
    pub workspace_trace: Vec<String>,
    pub state: NativeFpsEncounterStateReadout,
    pub lifecycle: NativeFpsEncounterLifecycleInput,
    pub read_sets: Vec<NativeFpsReadSetEvidence>,
    pub encounter_hash: String,
    pub replay_hash: String,
}

#[napi(object)]
pub struct NativeFpsEncounterTransitionResult {
    pub backend: String,
    pub authority_surface: String,
    pub mutation_owner: String,
    pub workspace_trace: Vec<String>,
    pub accepted: bool,
    pub rejection_reason: Option<String>,
    pub event_kind: Option<String>,
    pub state: NativeFpsEncounterStateReadout,
    pub lifecycle: NativeFpsEncounterLifecycleInput,
    pub encounter_hash: String,
    pub replay_hash: String,
}

fn native_hash(value: u64) -> String {
    format!("fnv1a64:{value:016x}")
}

fn native_fps_role(value: &str) -> napi::Result<FpsBridgeRole> {
    match value {
        "player" => Ok(FpsBridgeRole::Player),
        "enemy" => Ok(FpsBridgeRole::Enemy),
        "neutral" => Ok(FpsBridgeRole::Neutral),
        other => Err(to_napi(RuntimeBridgeError::new(
            RuntimeBridgeErrorKind::InvalidInput,
            format!("unknown FPS role '{other}'"),
        ))),
    }
}

fn optional_native_fps_role(
    value: Option<String>,
    field: &str,
) -> napi::Result<Option<FpsBridgeRole>> {
    match value {
        Some(role) => native_fps_role(role.as_str()).map(Some).map_err(|_| {
            to_napi(RuntimeBridgeError::new(
                RuntimeBridgeErrorKind::InvalidInput,
                format!("{field} must be player, enemy, or neutral"),
            ))
        }),
        None => Ok(None),
    }
}

fn native_fps_lifecycle_status(
    value: runtime_bridge_api::FpsBridgeLifecycleStatus,
) -> NativeFpsLifecycleStatus {
    match value {
        runtime_bridge_api::FpsBridgeLifecycleStatus::Active => NativeFpsLifecycleStatus {
            state: "active".into(),
            entity: None,
            tick: None,
        },
        runtime_bridge_api::FpsBridgeLifecycleStatus::EnemyDefeated { entity, tick } => {
            NativeFpsLifecycleStatus {
                state: "enemy_defeated".into(),
                entity: Some(entity as i64),
                tick: Some(tick as i64),
            }
        }
    }
}

fn bridge_fps_transform(
    value: NativeFpsTransformCapability,
    field: &str,
) -> napi::Result<FpsBridgeTransformCapability> {
    if value.rotation.len() != 4 || value.rotation.iter().any(|v| !v.is_finite()) {
        return Err(to_napi(RuntimeBridgeError::new(
            RuntimeBridgeErrorKind::InvalidInput,
            format!("{field}.rotation must be a finite quaternion"),
        )));
    }
    let translation = value.translation.to_vec3(&format!("{field}.translation"))?;
    let scale = value.scale.to_vec3(&format!("{field}.scale"))?;
    Ok(FpsBridgeTransformCapability {
        translation: [translation.x, translation.y, translation.z],
        rotation: [
            value.rotation[0] as f32,
            value.rotation[1] as f32,
            value.rotation[2] as f32,
            value.rotation[3] as f32,
        ],
        scale: [scale.x, scale.y, scale.z],
    })
}

fn bridge_fps_bounds(
    value: NativeFpsBoundsCapability,
    field: &str,
) -> napi::Result<FpsBridgeBoundsCapability> {
    let min = value.min.to_vec3(&format!("{field}.min"))?;
    let max = value.max.to_vec3(&format!("{field}.max"))?;
    Ok(FpsBridgeBoundsCapability {
        min: [min.x, min.y, min.z],
        max: [max.x, max.y, max.z],
    })
}

fn bridge_fps_definitions(
    definitions: Vec<NativeFpsStoredEntityDefinition>,
) -> napi::Result<Vec<FpsBridgeStoredEntityDefinition>> {
    definitions
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let field = format!("definitions[{index}]");
            Ok(FpsBridgeStoredEntityDefinition {
                entity: u64_input(value.entity, &format!("{field}.entity"))?,
                stable_id: value.stable_id,
                display_name: value.display_name,
                source_path: value.source_path,
                tags: value.tags,
                role: native_fps_role(&value.role)?,
                transform: value
                    .transform
                    .map(|transform| bridge_fps_transform(transform, &format!("{field}.transform")))
                    .transpose()?,
                bounds: value
                    .bounds
                    .map(|bounds| bridge_fps_bounds(bounds, &format!("{field}.bounds")))
                    .transpose()?,
                render_visible: value.render_visible,
                static_collider: value.static_collider,
                health: value.health.map(|health| FpsBridgeHealth {
                    current: health.current,
                    max: health.max,
                }),
                weapon: value.weapon.map(|weapon| FpsBridgeWeaponMount {
                    weapon_id: weapon.weapon_id,
                    damage: weapon.damage,
                    range_units: weapon.range_units,
                    ammo: weapon.ammo,
                    cooldown_ticks_after_fire: weapon.cooldown_ticks_after_fire,
                }),
                policy_binding: value.policy_binding.map(|binding| FpsBridgePolicyBinding {
                    binding_id: binding.binding_id,
                    policy_id: binding.policy_id,
                    view_kind: binding.view_kind,
                    view_version: binding.view_version,
                    allowed_intents: binding.allowed_intents,
                    runtime_moment: binding.runtime_moment,
                }),
            })
        })
        .collect()
}

impl From<FpsRuntimeSessionSnapshot> for NativeFpsRuntimeSessionSnapshot {
    fn from(value: FpsRuntimeSessionSnapshot) -> Self {
        Self {
            backend: value.backend,
            authority_surface: value.authority_surface,
            project_bundle: value.project_bundle,
            session_epoch: value.session_epoch as i64,
            lifecycle_status: native_fps_lifecycle_status(value.lifecycle_status),
            player_entity: value.player_entity as i64,
            enemy_entity: value.enemy_entity as i64,
            health: value
                .health
                .into_iter()
                .map(|health| NativeFpsEntityHealthReadout {
                    entity: health.entity as i64,
                    current: health.current,
                    max: health.max,
                })
                .collect(),
            policy_bindings: value
                .policy_bindings
                .into_iter()
                .map(|binding| NativeFpsPolicyBindingReadout {
                    entity: binding.entity as i64,
                    binding_id: binding.binding_id,
                    policy_id: binding.policy_id,
                    view_kind: binding.view_kind,
                    view_version: binding.view_version,
                    allowed_intents: binding.allowed_intents,
                    runtime_moment: binding.runtime_moment,
                })
                .collect(),
            replay_records: value
                .replay_records
                .into_iter()
                .map(|record| NativeFpsReplayEvidence {
                    replay_unit: record.replay_unit,
                    entity_hash: native_hash(record.entity_hash),
                    health_hash: native_hash(record.health_hash),
                    record_hash: native_hash(record.record_hash),
                })
                .collect(),
            read_sets: value
                .read_sets
                .into_iter()
                .map(|read_set| NativeFpsReadSetEvidence {
                    view_kind: read_set.view_kind,
                    owner: read_set.owner,
                    read_set: read_set.read_set,
                })
                .collect(),
            entity_hash: native_hash(value.entity_hash),
            health_hash: native_hash(value.health_hash),
            replay_hash: native_hash(value.replay_hash),
        }
    }
}

impl From<FpsPrimaryFireResult> for NativeFpsPrimaryFireResult {
    fn from(value: FpsPrimaryFireResult) -> Self {
        Self {
            backend: value.backend,
            authority_surface: value.authority_surface,
            mutation_owner: value.mutation_owner,
            workspace_trace: value.workspace_trace,
            shooter: value.shooter as i64,
            target: value.target.map(|target| target as i64),
            target_health_before: value.target_health_before.map(|health| NativeFpsHealth {
                current: health.current,
                max: health.max,
            }),
            target_health_after: value.target_health_after.map(|health| NativeFpsHealth {
                current: health.current,
                max: health.max,
            }),
            lifecycle_status: native_fps_lifecycle_status(value.lifecycle_status),
            target_render_visible: value.target_render_visible,
            entity_hash: native_hash(value.entity_hash),
            health_hash: native_hash(value.health_hash),
            replay_hash: native_hash(value.replay_hash),
        }
    }
}

impl From<NativeFpsEncounterLifecycleInput> for FpsEncounterLifecycleInput {
    fn from(value: NativeFpsEncounterLifecycleInput) -> Self {
        Self {
            outcome_kind: value.outcome_kind,
            terminal: value.terminal,
            enemy_dead: value.enemy_dead,
            player_dead: value.player_dead,
            lifecycle_hash: value.lifecycle_hash,
        }
    }
}

impl From<FpsEncounterLifecycleInput> for NativeFpsEncounterLifecycleInput {
    fn from(value: FpsEncounterLifecycleInput) -> Self {
        Self {
            outcome_kind: value.outcome_kind,
            terminal: value.terminal,
            enemy_dead: value.enemy_dead,
            player_dead: value.player_dead,
            lifecycle_hash: value.lifecycle_hash,
        }
    }
}

impl From<FpsEncounterStateReadout> for NativeFpsEncounterStateReadout {
    fn from(value: FpsEncounterStateReadout) -> Self {
        Self {
            preset_id: value.preset_id,
            status: value.status,
            spawned_enemy_ids: value.spawned_enemy_ids,
            defeated_enemy_ids: value.defeated_enemy_ids,
            revision: value.revision as i64,
            last_transition: value.last_transition,
        }
    }
}

fn native_fps_read_sets(
    read_sets: Vec<runtime_bridge_api::FpsReadSetEvidence>,
) -> Vec<NativeFpsReadSetEvidence> {
    read_sets
        .into_iter()
        .map(|read_set| NativeFpsReadSetEvidence {
            view_kind: read_set.view_kind,
            owner: read_set.owner,
            read_set: read_set.read_set,
        })
        .collect()
}

impl From<FpsEncounterDirectorSnapshot> for NativeFpsEncounterDirectorSnapshot {
    fn from(value: FpsEncounterDirectorSnapshot) -> Self {
        Self {
            backend: value.backend,
            authority_surface: value.authority_surface,
            mutation_owner: value.mutation_owner,
            workspace_trace: value.workspace_trace,
            state: value.state.into(),
            lifecycle: value.lifecycle.into(),
            read_sets: native_fps_read_sets(value.read_sets),
            encounter_hash: native_hash(value.encounter_hash),
            replay_hash: native_hash(value.replay_hash),
        }
    }
}

impl From<FpsEncounterTransitionResult> for NativeFpsEncounterTransitionResult {
    fn from(value: FpsEncounterTransitionResult) -> Self {
        Self {
            backend: value.backend,
            authority_surface: value.authority_surface,
            mutation_owner: value.mutation_owner,
            workspace_trace: value.workspace_trace,
            accepted: value.accepted,
            rejection_reason: value.rejection_reason,
            event_kind: value.event_kind,
            state: value.state.into(),
            lifecycle: value.lifecycle.into(),
            encounter_hash: native_hash(value.encounter_hash),
            replay_hash: native_hash(value.replay_hash),
        }
    }
}

#[napi]
pub fn load_fps_runtime_session(
    handle: i64,
    project_bundle: String,
    definitions: Vec<NativeFpsStoredEntityDefinition>,
    game_rule_modules_json: String,
) -> napi::Result<NativeFpsRuntimeSessionSnapshot> {
    let definitions = bridge_fps_definitions(definitions)?;
    let game_rule_modules = parse_game_rule_module_manifests(&game_rule_modules_json)?;
    with_bridge(handle, |bridge| {
        bridge
            .load_fps_runtime_session(FpsRuntimeSessionLoadRequest {
                project_bundle,
                definitions,
                game_rule_modules,
            })
            .map(NativeFpsRuntimeSessionSnapshot::from)
            .map_err(to_napi)
    })
}

#[napi]
pub fn read_fps_runtime_session(handle: i64) -> napi::Result<NativeFpsRuntimeSessionSnapshot> {
    with_bridge(handle, |bridge| {
        bridge
            .read_fps_runtime_session()
            .map(NativeFpsRuntimeSessionSnapshot::from)
            .map_err(to_napi)
    })
}

#[napi]
pub fn apply_fps_primary_fire(
    handle: i64,
    tick: i64,
    origin: NativeVec3,
    direction: NativeVec3,
    shooter_role: Option<String>,
    target_role: Option<String>,
) -> napi::Result<NativeFpsPrimaryFireResult> {
    let tick = u64_input(tick, "tick")?;
    let origin = origin.to_vec3("origin")?;
    let direction = direction.to_vec3("direction")?;
    let shooter_role = optional_native_fps_role(shooter_role, "shooterRole")?;
    let target_role = optional_native_fps_role(target_role, "targetRole")?;
    with_bridge(handle, |bridge| {
        bridge
            .apply_fps_primary_fire(FpsPrimaryFireRequest {
                tick,
                origin: [
                    f64::from(origin.x),
                    f64::from(origin.y),
                    f64::from(origin.z),
                ],
                direction: [
                    f64::from(direction.x),
                    f64::from(direction.y),
                    f64::from(direction.z),
                ],
                shooter_role,
                target_role,
            })
            .map(NativeFpsPrimaryFireResult::from)
            .map_err(to_napi)
    })
}

#[napi]
pub fn invoke_game_extension_weapon_effect(
    handle: i64,
    hook_json: String,
    tick: i64,
    origin: NativeVec3,
    direction: NativeVec3,
    shooter_role: Option<String>,
    target_role: Option<String>,
) -> napi::Result<NativeGameExtensionWeaponEffectInvocationResult> {
    let hook = parse_weapon_effect_hook_request(&hook_json)?;
    let tick = u64_input(tick, "tick")?;
    let origin = origin.to_vec3("origin")?;
    let direction = direction.to_vec3("direction")?;
    let shooter_role = optional_native_fps_role(shooter_role, "shooterRole")?;
    let target_role = optional_native_fps_role(target_role, "targetRole")?;
    with_bridge(handle, |bridge| {
        let result = bridge
            .invoke_game_extension_weapon_effect(GameExtensionWeaponEffectInvocationRequest {
                hook,
                primary_fire: FpsPrimaryFireRequest {
                    tick,
                    origin: [
                        f64::from(origin.x),
                        f64::from(origin.y),
                        f64::from(origin.z),
                    ],
                    direction: [
                        f64::from(direction.x),
                        f64::from(direction.y),
                        f64::from(direction.z),
                    ],
                    shooter_role,
                    target_role,
                },
            })
            .map_err(to_napi)?;
        Ok(NativeGameExtensionWeaponEffectInvocationResult {
            hook_receipt_json: game_extension_json(&result.hook_receipt)?,
            replay_evidence_json: game_extension_json(&result.replay_evidence)?,
            primary_fire: result.primary_fire.map(NativeFpsPrimaryFireResult::from),
        })
    })
}

#[napi]
pub fn validate_game_rule_catalog(handle: i64, catalog_json: String) -> napi::Result<String> {
    let catalog = parse_game_rule_catalog(&catalog_json)?;
    with_bridge(handle, |bridge| {
        let receipt = bridge
            .validate_game_rule_catalog(catalog)
            .map_err(to_napi)?;
        game_rule_json(&receipt)
    })
}

#[napi]
pub fn submit_game_rule_effect_intent(
    handle: i64,
    catalog_json: String,
    request_json: String,
) -> napi::Result<String> {
    let catalog = parse_game_rule_catalog(&catalog_json)?;
    let request = parse_game_rule_resolution_request(&request_json)?;
    with_bridge(handle, |bridge| {
        let receipt = bridge
            .submit_game_rule_effect_intent(GameRuleEffectIntentRequest { catalog, request })
            .map_err(to_napi)?;
        game_rule_json(&receipt)
    })
}

#[napi]
pub fn read_game_rule_runtime_readout(handle: i64) -> napi::Result<String> {
    with_bridge(handle, |bridge| {
        let readout = bridge.read_game_rule_runtime_readout().map_err(to_napi)?;
        game_rule_json(&readout)
    })
}

#[napi]
pub fn restart_fps_runtime_session(
    handle: i64,
    expected_epoch: i64,
) -> napi::Result<NativeFpsRuntimeSessionSnapshot> {
    let expected_epoch = u64_input(expected_epoch, "expected_epoch")?;
    with_bridge(handle, |bridge| {
        bridge
            .restart_fps_runtime_session(FpsRuntimeSessionRestartRequest { expected_epoch })
            .map(NativeFpsRuntimeSessionSnapshot::from)
            .map_err(to_napi)
    })
}

#[napi]
pub fn read_fps_encounter_director(
    handle: i64,
    lifecycle: NativeFpsEncounterLifecycleInput,
) -> napi::Result<NativeFpsEncounterDirectorSnapshot> {
    with_bridge(handle, |bridge| {
        bridge
            .read_fps_encounter_director(lifecycle.into())
            .map(NativeFpsEncounterDirectorSnapshot::from)
            .map_err(to_napi)
    })
}

#[napi]
pub fn apply_fps_encounter_transition(
    handle: i64,
    request: NativeFpsEncounterTransitionRequest,
) -> napi::Result<NativeFpsEncounterTransitionResult> {
    with_bridge(handle, |bridge| {
        bridge
            .apply_fps_encounter_transition(FpsEncounterTransitionRequest {
                preset_id: request.preset_id,
                action: request.action,
                lifecycle: request.lifecycle.into(),
            })
            .map(NativeFpsEncounterTransitionResult::from)
            .map_err(to_napi)
    })
}
