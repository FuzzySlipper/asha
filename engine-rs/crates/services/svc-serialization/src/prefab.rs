//! Durable prefab registry model and fail-closed content validation.
//!
//! Prefabs are stored ProjectBundle content. They describe reusable authored
//! composition and stable local part roles; they do not instantiate runtime
//! entities. Runtime expansion and provenance belong to the project-bundle rule
//! lane.

use core_assets::{AssetId, AssetKind};
use core_ids::{PrefabId, PrefabInstanceId, PrefabPartId};
use std::collections::{BTreeMap, BTreeSet};

pub const PREFAB_REGISTRY_SCHEMA_VERSION: u32 = 1;
pub const PREFAB_DEFINITION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrefabTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl PrefabTransform {
    pub const IDENTITY: Self = Self {
        translation: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    };

    fn is_valid(self) -> bool {
        self.translation
            .into_iter()
            .chain(self.rotation)
            .chain(self.scale)
            .all(f32::is_finite)
            && self.scale.into_iter().all(|axis| axis != 0.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefabPartSource {
    Scene { asset: String },
    EntityDefinition { stable_id: String },
    VoxelObject { asset: String },
}

impl PrefabPartSource {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Scene { .. } => "scene",
            Self::EntityDefinition { .. } => "entityDefinition",
            Self::VoxelObject { .. } => "voxelObject",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefabPart {
    pub id: PrefabPartId,
    pub namespace: String,
    pub display_name: String,
    pub parent: Option<PrefabPartId>,
    pub transform: PrefabTransform,
    pub source: PrefabPartSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefabPartRoleBinding {
    pub role: String,
    pub part: PrefabPartId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefabOverrideValue {
    Transform { transform: PrefabTransform },
    EntityDefinition { stable_id: String },
    Asset { asset: String },
    Material { asset: String },
    Activation { active: bool },
}

impl PrefabOverrideValue {
    pub fn field(&self) -> &'static str {
        match self {
            Self::Transform { .. } => "transform",
            Self::EntityDefinition { .. } => "entityDefinition",
            Self::Asset { .. } => "asset",
            Self::Material { .. } => "material",
            Self::Activation { .. } => "activation",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefabOverride {
    pub target_role: String,
    pub value: PrefabOverrideValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefabVariantDelta {
    /// Stable authored key used to select this variant from a base prefab.
    pub variant_id: String,
    pub base: PrefabId,
    pub removed_roles: Vec<String>,
    pub overrides: Vec<PrefabOverride>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefabDefinition {
    pub id: PrefabId,
    pub schema_version: u32,
    pub display_name: String,
    pub parts: Vec<PrefabPart>,
    pub part_roles: Vec<PrefabPartRoleBinding>,
    pub variant: Option<PrefabVariantDelta>,
}

impl PrefabDefinition {
    pub fn canonicalize(&mut self) {
        self.parts.sort_by_key(|part| part.id.raw());
        self.part_roles.sort_by(|a, b| a.role.cmp(&b.role));
        if let Some(variant) = &mut self.variant {
            variant.removed_roles.sort();
            variant.overrides.sort_by(|a, b| {
                (a.target_role.as_str(), a.value.field())
                    .cmp(&(b.target_role.as_str(), b.value.field()))
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefabRegistry {
    pub schema_version: u32,
    pub definitions: Vec<PrefabDefinition>,
}

impl PrefabRegistry {
    pub fn canonical(&self) -> Self {
        let mut registry = self.clone();
        for definition in &mut registry.definitions {
            definition.canonicalize();
        }
        registry
            .definitions
            .sort_by_key(|definition| definition.id.raw());
        registry
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefabInstanceRecord {
    pub instance: PrefabInstanceId,
    pub prefab: PrefabId,
    pub seed: u64,
    pub transform: PrefabTransform,
    pub overrides: Vec<PrefabOverride>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefabPartReference {
    pub prefab: PrefabId,
    pub role: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrefabRegistryValidationContext {
    pub asset_ids: BTreeSet<String>,
    pub entity_definition_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrefabDiagnosticCode {
    UnsupportedRegistrySchema,
    UnsupportedDefinitionSchema,
    DuplicatePrefabId,
    MissingDisplayName,
    DuplicatePartId,
    InvalidPartNamespace,
    DuplicatePartNamespace,
    MissingParentPart,
    PartHierarchyCycle,
    InvalidPartTransform,
    UnknownAsset,
    AssetKindMismatch,
    UnknownEntityDefinition,
    InvalidPartRole,
    DuplicatePartRole,
    DanglingPartRole,
    MissingBasePrefab,
    InvalidVariantId,
    DuplicateVariantId,
    VariantCycle,
    VariantDepthExceeded,
    VariantDefinesParts,
    UnknownRemovedRole,
    DuplicateRemovedRole,
    UnsafePartRemoval,
    InvalidOverrideTarget,
    DuplicateOverride,
    InvalidOverrideValue,
    DeletedRoleReferenced,
}

impl PrefabDiagnosticCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedRegistrySchema => "unsupportedRegistrySchema",
            Self::UnsupportedDefinitionSchema => "unsupportedDefinitionSchema",
            Self::DuplicatePrefabId => "duplicatePrefabId",
            Self::MissingDisplayName => "missingDisplayName",
            Self::DuplicatePartId => "duplicatePartId",
            Self::InvalidPartNamespace => "invalidPartNamespace",
            Self::DuplicatePartNamespace => "duplicatePartNamespace",
            Self::MissingParentPart => "missingParentPart",
            Self::PartHierarchyCycle => "partHierarchyCycle",
            Self::InvalidPartTransform => "invalidPartTransform",
            Self::UnknownAsset => "unknownAsset",
            Self::AssetKindMismatch => "assetKindMismatch",
            Self::UnknownEntityDefinition => "unknownEntityDefinition",
            Self::InvalidPartRole => "invalidPartRole",
            Self::DuplicatePartRole => "duplicatePartRole",
            Self::DanglingPartRole => "danglingPartRole",
            Self::MissingBasePrefab => "missingBasePrefab",
            Self::InvalidVariantId => "invalidVariantId",
            Self::DuplicateVariantId => "duplicateVariantId",
            Self::VariantCycle => "variantCycle",
            Self::VariantDepthExceeded => "variantDepthExceeded",
            Self::VariantDefinesParts => "variantDefinesParts",
            Self::UnknownRemovedRole => "unknownRemovedRole",
            Self::DuplicateRemovedRole => "duplicateRemovedRole",
            Self::UnsafePartRemoval => "unsafePartRemoval",
            Self::InvalidOverrideTarget => "invalidOverrideTarget",
            Self::DuplicateOverride => "duplicateOverride",
            Self::InvalidOverrideValue => "invalidOverrideValue",
            Self::DeletedRoleReferenced => "deletedRoleReferenced",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefabDiagnostic {
    pub code: PrefabDiagnosticCode,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrefabValidationReport {
    pub diagnostics: Vec<PrefabDiagnostic>,
}

impl PrefabValidationReport {
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }

    fn push(
        &mut self,
        code: PrefabDiagnosticCode,
        path: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.diagnostics.push(PrefabDiagnostic {
            code,
            path: path.into(),
            message: message.into(),
        });
    }

    fn canonicalize(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            (a.path.as_str(), a.code.as_str(), a.message.as_str()).cmp(&(
                b.path.as_str(),
                b.code.as_str(),
                b.message.as_str(),
            ))
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedPrefabRegistry(PrefabRegistry);

impl ValidatedPrefabRegistry {
    pub fn new(
        registry: PrefabRegistry,
        context: &PrefabRegistryValidationContext,
    ) -> Result<Self, PrefabValidationReport> {
        let report = validate_prefab_registry(&registry, context);
        if report.is_valid() {
            Ok(Self(registry.canonical()))
        } else {
            Err(report)
        }
    }

    pub fn as_registry(&self) -> &PrefabRegistry {
        &self.0
    }

    pub fn into_registry(self) -> PrefabRegistry {
        self.0
    }
}

pub fn validate_prefab_registry(
    registry: &PrefabRegistry,
    context: &PrefabRegistryValidationContext,
) -> PrefabValidationReport {
    let mut report = PrefabValidationReport::default();
    if registry.schema_version != PREFAB_REGISTRY_SCHEMA_VERSION {
        report.push(
            PrefabDiagnosticCode::UnsupportedRegistrySchema,
            "schemaVersion",
            format!("expected prefab registry schema {PREFAB_REGISTRY_SCHEMA_VERSION}"),
        );
    }

    let mut definitions = BTreeMap::new();
    for (index, definition) in registry.definitions.iter().enumerate() {
        let path = format!("definitions[{index}]");
        if definitions.insert(definition.id, definition).is_some() {
            report.push(
                PrefabDiagnosticCode::DuplicatePrefabId,
                format!("{path}.id"),
                format!("duplicate prefab id {}", definition.id.raw()),
            );
        }
        validate_definition(definition, context, &path, &mut report);
    }

    validate_variants(&definitions, context, &mut report);
    report.canonicalize();
    report
}

fn validate_definition(
    definition: &PrefabDefinition,
    context: &PrefabRegistryValidationContext,
    path: &str,
    report: &mut PrefabValidationReport,
) {
    if definition.schema_version != PREFAB_DEFINITION_SCHEMA_VERSION {
        report.push(
            PrefabDiagnosticCode::UnsupportedDefinitionSchema,
            format!("{path}.schemaVersion"),
            format!("expected prefab definition schema {PREFAB_DEFINITION_SCHEMA_VERSION}"),
        );
    }
    if definition.display_name.trim().is_empty() {
        report.push(
            PrefabDiagnosticCode::MissingDisplayName,
            format!("{path}.displayName"),
            "display name must not be blank",
        );
    }
    if definition.variant.is_some()
        && (!definition.parts.is_empty() || !definition.part_roles.is_empty())
    {
        report.push(
            PrefabDiagnosticCode::VariantDefinesParts,
            path,
            "Wave 1 variants are deltas and may not define new parts or roles",
        );
    }

    let mut parts = BTreeMap::new();
    let mut namespaces = BTreeSet::new();
    for (index, part) in definition.parts.iter().enumerate() {
        let part_path = format!("{path}.parts[{index}]");
        if parts.insert(part.id, part).is_some() {
            report.push(
                PrefabDiagnosticCode::DuplicatePartId,
                format!("{part_path}.id"),
                format!("duplicate part id {}", part.id.raw()),
            );
        }
        if !is_scoped_key(&part.namespace) {
            report.push(
                PrefabDiagnosticCode::InvalidPartNamespace,
                format!("{part_path}.namespace"),
                "part namespace must be slash-scoped lowercase kebab-case",
            );
        } else if !namespaces.insert(part.namespace.as_str()) {
            report.push(
                PrefabDiagnosticCode::DuplicatePartNamespace,
                format!("{part_path}.namespace"),
                format!("duplicate part namespace {}", part.namespace),
            );
        }
        if !part.transform.is_valid() {
            report.push(
                PrefabDiagnosticCode::InvalidPartTransform,
                format!("{part_path}.transform"),
                "transform values must be finite and scale axes non-zero",
            );
        }
        validate_source(
            &part.source,
            context,
            &format!("{part_path}.source"),
            report,
        );
    }

    for (index, part) in definition.parts.iter().enumerate() {
        if let Some(parent) = part.parent {
            if !parts.contains_key(&parent) {
                report.push(
                    PrefabDiagnosticCode::MissingParentPart,
                    format!("{path}.parts[{index}].parent"),
                    format!("unknown parent part {}", parent.raw()),
                );
            }
        }
    }
    validate_part_cycles(&parts, path, report);

    let mut roles = BTreeSet::new();
    for (index, binding) in definition.part_roles.iter().enumerate() {
        let role_path = format!("{path}.partRoles[{index}]");
        if !is_scoped_key(&binding.role) {
            report.push(
                PrefabDiagnosticCode::InvalidPartRole,
                format!("{role_path}.role"),
                "part role must be slash-scoped lowercase kebab-case",
            );
        }
        if !roles.insert(binding.role.as_str()) {
            report.push(
                PrefabDiagnosticCode::DuplicatePartRole,
                format!("{role_path}.role"),
                format!("duplicate part role {}", binding.role),
            );
        }
        if !parts.contains_key(&binding.part) {
            report.push(
                PrefabDiagnosticCode::DanglingPartRole,
                format!("{role_path}.part"),
                format!("role targets unknown part {}", binding.part.raw()),
            );
        }
    }
}

fn validate_source(
    source: &PrefabPartSource,
    context: &PrefabRegistryValidationContext,
    path: &str,
    report: &mut PrefabValidationReport,
) {
    match source {
        PrefabPartSource::Scene { asset } => {
            validate_asset(asset, AssetKind::Scene, context, path, report)
        }
        PrefabPartSource::VoxelObject { asset } => {
            validate_asset(asset, AssetKind::VoxelObject, context, path, report)
        }
        PrefabPartSource::EntityDefinition { stable_id } => {
            if !context.entity_definition_ids.contains(stable_id) {
                report.push(
                    PrefabDiagnosticCode::UnknownEntityDefinition,
                    path,
                    format!("unknown EntityDefinition {stable_id}"),
                );
            }
        }
    }
}

fn validate_asset(
    asset: &str,
    expected: AssetKind,
    context: &PrefabRegistryValidationContext,
    path: &str,
    report: &mut PrefabValidationReport,
) {
    let Ok(id) = AssetId::parse(asset) else {
        report.push(
            PrefabDiagnosticCode::AssetKindMismatch,
            path,
            format!("malformed {expected} asset id {asset}"),
        );
        return;
    };
    if id.kind() != expected {
        report.push(
            PrefabDiagnosticCode::AssetKindMismatch,
            path,
            format!("expected {expected} asset, found {}", id.kind()),
        );
    } else if !context.asset_ids.contains(asset) {
        report.push(
            PrefabDiagnosticCode::UnknownAsset,
            path,
            format!("unknown asset {asset}"),
        );
    }
}

fn validate_part_cycles(
    parts: &BTreeMap<PrefabPartId, &PrefabPart>,
    path: &str,
    report: &mut PrefabValidationReport,
) {
    for start in parts.keys() {
        let mut seen = BTreeSet::new();
        let mut cursor = Some(*start);
        while let Some(id) = cursor {
            if !seen.insert(id) {
                report.push(
                    PrefabDiagnosticCode::PartHierarchyCycle,
                    format!("{path}.parts"),
                    format!("part hierarchy cycle includes {}", id.raw()),
                );
                break;
            }
            cursor = parts.get(&id).and_then(|part| part.parent);
        }
    }
}

fn validate_variants(
    definitions: &BTreeMap<PrefabId, &PrefabDefinition>,
    context: &PrefabRegistryValidationContext,
    report: &mut PrefabValidationReport,
) {
    let mut variant_ids_by_base = BTreeMap::<PrefabId, BTreeSet<&str>>::new();
    for definition in definitions.values() {
        let Some(variant) = &definition.variant else {
            continue;
        };
        let path = format!("prefab[{}].variant", definition.id.raw());
        if !is_scoped_key(&variant.variant_id) {
            report.push(
                PrefabDiagnosticCode::InvalidVariantId,
                format!("{path}.variantId"),
                "variant id must be slash-scoped lowercase kebab-case",
            );
        } else if !variant_ids_by_base
            .entry(variant.base)
            .or_default()
            .insert(variant.variant_id.as_str())
        {
            report.push(
                PrefabDiagnosticCode::DuplicateVariantId,
                format!("{path}.variantId"),
                format!(
                    "duplicate variant id `{}` for base prefab {}",
                    variant.variant_id,
                    variant.base.raw()
                ),
            );
        }
        let Some(base) = definitions.get(&variant.base).copied() else {
            report.push(
                PrefabDiagnosticCode::MissingBasePrefab,
                format!("{path}.base"),
                format!("unknown base prefab {}", variant.base.raw()),
            );
            continue;
        };
        if base.id == definition.id {
            report.push(
                PrefabDiagnosticCode::VariantCycle,
                &path,
                "variant may not base itself",
            );
            continue;
        }
        if base.variant.is_some() {
            let code = if variant_chain_reaches(base, definition.id, definitions) {
                PrefabDiagnosticCode::VariantCycle
            } else {
                PrefabDiagnosticCode::VariantDepthExceeded
            };
            report.push(code, &path, "Wave 1 permits exactly one variant level");
            continue;
        }
        validate_variant_delta(variant, base, context, &path, report);
    }
}

fn variant_chain_reaches(
    start: &PrefabDefinition,
    target: PrefabId,
    definitions: &BTreeMap<PrefabId, &PrefabDefinition>,
) -> bool {
    let mut cursor = Some(start);
    let mut seen = BTreeSet::new();
    while let Some(definition) = cursor {
        if definition.id == target {
            return true;
        }
        if !seen.insert(definition.id) {
            return true;
        }
        cursor = definition
            .variant
            .as_ref()
            .and_then(|variant| definitions.get(&variant.base).copied());
    }
    false
}

fn validate_variant_delta(
    variant: &PrefabVariantDelta,
    base: &PrefabDefinition,
    context: &PrefabRegistryValidationContext,
    path: &str,
    report: &mut PrefabValidationReport,
) {
    let roles: BTreeMap<&str, PrefabPartId> = base
        .part_roles
        .iter()
        .map(|binding| (binding.role.as_str(), binding.part))
        .collect();
    let parts: BTreeMap<PrefabPartId, &PrefabPart> =
        base.parts.iter().map(|part| (part.id, part)).collect();
    let mut removed = BTreeSet::new();
    for (index, role) in variant.removed_roles.iter().enumerate() {
        if !removed.insert(role.as_str()) {
            report.push(
                PrefabDiagnosticCode::DuplicateRemovedRole,
                format!("{path}.removedRoles[{index}]"),
                format!("role {role} is removed more than once"),
            );
        }
        if !roles.contains_key(role.as_str()) {
            report.push(
                PrefabDiagnosticCode::UnknownRemovedRole,
                format!("{path}.removedRoles[{index}]"),
                format!("unknown base role {role}"),
            );
        }
    }

    let removed_parts: BTreeSet<PrefabPartId> = removed
        .iter()
        .filter_map(|role| roles.get(role).copied())
        .collect();
    for removed_part in &removed_parts {
        for binding in &base.part_roles {
            if binding.part == *removed_part && !removed.contains(binding.role.as_str()) {
                report.push(
                    PrefabDiagnosticCode::UnsafePartRemoval,
                    format!("{path}.removedRoles"),
                    format!(
                        "removing part {} through one role would leave retained role {} dangling",
                        removed_part.raw(),
                        binding.role
                    ),
                );
            }
        }
        if parts
            .values()
            .any(|part| part.parent == Some(*removed_part) && !removed_parts.contains(&part.id))
        {
            report.push(
                PrefabDiagnosticCode::UnsafePartRemoval,
                format!("{path}.removedRoles"),
                format!(
                    "removing part {} would leave a retained child dangling",
                    removed_part.raw()
                ),
            );
        }
    }

    let mut targets = BTreeSet::new();
    for (index, item) in variant.overrides.iter().enumerate() {
        let item_path = format!("{path}.overrides[{index}]");
        let Some(part_id) = roles.get(item.target_role.as_str()).copied() else {
            report.push(
                PrefabDiagnosticCode::InvalidOverrideTarget,
                format!("{item_path}.targetRole"),
                format!("unknown base role {}", item.target_role),
            );
            continue;
        };
        if removed_parts.contains(&part_id) {
            report.push(
                PrefabDiagnosticCode::DeletedRoleReferenced,
                &item_path,
                format!(
                    "override role {} resolves to removed part {}",
                    item.target_role,
                    part_id.raw()
                ),
            );
        }
        if !targets.insert((item.target_role.as_str(), item.value.field())) {
            report.push(
                PrefabDiagnosticCode::DuplicateOverride,
                &item_path,
                format!(
                    "duplicate {} override for {}",
                    item.value.field(),
                    item.target_role
                ),
            );
        }
        let Some(part) = parts.get(&part_id) else {
            continue;
        };
        match &item.value {
            PrefabOverrideValue::Transform { transform } if !transform.is_valid() => report.push(
                PrefabDiagnosticCode::InvalidOverrideValue,
                &item_path,
                "override transform is invalid",
            ),
            PrefabOverrideValue::EntityDefinition { stable_id } => {
                if !matches!(part.source, PrefabPartSource::EntityDefinition { .. })
                    || !context.entity_definition_ids.contains(stable_id)
                {
                    report.push(
                        PrefabDiagnosticCode::InvalidOverrideValue,
                        &item_path,
                        "EntityDefinition override requires an entity-definition part and known id",
                    );
                }
            }
            PrefabOverrideValue::Asset { asset } => {
                let expected = match part.source {
                    PrefabPartSource::Scene { .. } => Some(AssetKind::Scene),
                    PrefabPartSource::VoxelObject { .. } => Some(AssetKind::VoxelObject),
                    PrefabPartSource::EntityDefinition { .. } => None,
                };
                match expected {
                    Some(kind) => validate_asset(asset, kind, context, &item_path, report),
                    None => report.push(
                        PrefabDiagnosticCode::InvalidOverrideValue,
                        &item_path,
                        "asset override cannot target an EntityDefinition part",
                    ),
                }
            }
            PrefabOverrideValue::Material { asset } => {
                if matches!(part.source, PrefabPartSource::EntityDefinition { .. }) {
                    report.push(
                        PrefabDiagnosticCode::InvalidOverrideValue,
                        &item_path,
                        "material override requires a Scene or VoxelObject part",
                    );
                } else {
                    validate_asset(asset, AssetKind::Material, context, &item_path, report);
                }
            }
            PrefabOverrideValue::Activation { .. } => {}
            PrefabOverrideValue::Transform { .. } => {}
        }
    }
}

fn is_scoped_key(value: &str) -> bool {
    !value.is_empty() && value.split('/').all(is_kebab_segment)
}

fn is_kebab_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.starts_with('-')
        && !segment.ends_with('-')
        && !segment.contains("--")
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> PrefabRegistryValidationContext {
        PrefabRegistryValidationContext {
            asset_ids: [
                "scene/factory-cell",
                "voxel-object/assembler-body",
                "material/steel",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            entity_definition_ids: ["machine.assembler"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }

    fn base_definition() -> PrefabDefinition {
        PrefabDefinition {
            id: PrefabId::new(1),
            schema_version: 1,
            display_name: "Assembler".into(),
            parts: vec![
                PrefabPart {
                    id: PrefabPartId::new(20),
                    namespace: "body".into(),
                    display_name: "Body".into(),
                    parent: None,
                    transform: PrefabTransform::IDENTITY,
                    source: PrefabPartSource::VoxelObject {
                        asset: "voxel-object/assembler-body".into(),
                    },
                },
                PrefabPart {
                    id: PrefabPartId::new(10),
                    namespace: "controller".into(),
                    display_name: "Controller".into(),
                    parent: Some(PrefabPartId::new(20)),
                    transform: PrefabTransform::IDENTITY,
                    source: PrefabPartSource::EntityDefinition {
                        stable_id: "machine.assembler".into(),
                    },
                },
            ],
            part_roles: vec![
                PrefabPartRoleBinding {
                    role: "visual-body".into(),
                    part: PrefabPartId::new(20),
                },
                PrefabPartRoleBinding {
                    role: "gameplay-root".into(),
                    part: PrefabPartId::new(10),
                },
            ],
            variant: None,
        }
    }

    #[test]
    fn valid_registry_is_accepted_and_canonicalized() {
        let registry = PrefabRegistry {
            schema_version: 1,
            definitions: vec![base_definition()],
        };
        let validated = ValidatedPrefabRegistry::new(registry, &context()).unwrap();
        assert_eq!(validated.as_registry().definitions[0].parts[0].id.raw(), 10);
        assert_eq!(
            validated.as_registry().definitions[0].part_roles[0].role,
            "gameplay-root"
        );
    }

    #[test]
    fn display_rename_does_not_change_stable_part_reference() {
        let mut definition = base_definition();
        let reference = PrefabPartReference {
            prefab: definition.id,
            role: definition.part_roles[0].role.clone(),
        };
        definition.display_name = "Renamed Machine".into();
        definition.parts[0].display_name = "Renamed Part".into();
        assert_eq!(reference.prefab, definition.id);
        assert_eq!(reference.role, definition.part_roles[0].role);
    }

    #[test]
    fn invalid_registry_collects_errors_and_never_constructs_validated_state() {
        let mut definition = base_definition();
        definition.parts[1].namespace = "body".into();
        definition.part_roles.push(PrefabPartRoleBinding {
            role: "gameplay-root".into(),
            part: PrefabPartId::new(999),
        });
        if let PrefabPartSource::VoxelObject { asset } = &mut definition.parts[0].source {
            *asset = "voxel-object/missing".into();
        }
        let registry = PrefabRegistry {
            schema_version: 1,
            definitions: vec![definition],
        };
        let report = ValidatedPrefabRegistry::new(registry, &context()).unwrap_err();
        let codes: BTreeSet<_> = report.diagnostics.iter().map(|d| d.code).collect();
        assert!(codes.contains(&PrefabDiagnosticCode::DuplicatePartNamespace));
        assert!(codes.contains(&PrefabDiagnosticCode::DuplicatePartRole));
        assert!(codes.contains(&PrefabDiagnosticCode::DanglingPartRole));
        assert!(codes.contains(&PrefabDiagnosticCode::UnknownAsset));
    }

    #[test]
    fn variants_are_one_level_and_cannot_override_removed_roles() {
        let base = base_definition();
        let variant = PrefabDefinition {
            id: PrefabId::new(2),
            schema_version: 1,
            display_name: "Assembler Variant".into(),
            parts: vec![],
            part_roles: vec![],
            variant: Some(PrefabVariantDelta {
                variant_id: "damaged".into(),
                base: base.id,
                removed_roles: vec!["gameplay-root".into()],
                overrides: vec![PrefabOverride {
                    target_role: "gameplay-root".into(),
                    value: PrefabOverrideValue::Transform {
                        transform: PrefabTransform::IDENTITY,
                    },
                }],
            }),
        };
        let report = validate_prefab_registry(
            &PrefabRegistry {
                schema_version: 1,
                definitions: vec![base, variant],
            },
            &context(),
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == PrefabDiagnosticCode::DeletedRoleReferenced));
    }

    #[test]
    fn material_and_activation_overrides_are_typed_and_source_checked() {
        let base = base_definition();
        let valid = PrefabDefinition {
            id: PrefabId::new(2),
            schema_version: 1,
            display_name: "Dormant steel assembler".into(),
            parts: vec![],
            part_roles: vec![],
            variant: Some(PrefabVariantDelta {
                variant_id: "damaged".into(),
                base: base.id,
                removed_roles: vec![],
                overrides: vec![
                    PrefabOverride {
                        target_role: "visual-body".into(),
                        value: PrefabOverrideValue::Material {
                            asset: "material/steel".into(),
                        },
                    },
                    PrefabOverride {
                        target_role: "visual-body".into(),
                        value: PrefabOverrideValue::Activation { active: false },
                    },
                ],
            }),
        };
        ValidatedPrefabRegistry::new(
            PrefabRegistry {
                schema_version: 1,
                definitions: vec![base.clone(), valid],
            },
            &context(),
        )
        .expect("material and activation overrides are valid for a voxel part");

        let invalid = PrefabDefinition {
            id: PrefabId::new(3),
            schema_version: 1,
            display_name: "Invalid material target".into(),
            parts: vec![],
            part_roles: vec![],
            variant: Some(PrefabVariantDelta {
                variant_id: "damaged".into(),
                base: base.id,
                removed_roles: vec![],
                overrides: vec![PrefabOverride {
                    target_role: "gameplay-root".into(),
                    value: PrefabOverrideValue::Material {
                        asset: "material/steel".into(),
                    },
                }],
            }),
        };
        let report = validate_prefab_registry(
            &PrefabRegistry {
                schema_version: 1,
                definitions: vec![base, invalid],
            },
            &context(),
        );
        assert!(report
            .diagnostics
            .iter()
            .any(|item| item.code == PrefabDiagnosticCode::InvalidOverrideValue));
    }

    #[test]
    fn variant_removal_rejects_retained_aliases_and_alias_overrides() {
        let mut base = base_definition();
        base.part_roles.push(PrefabPartRoleBinding {
            role: "controller-alias".into(),
            part: PrefabPartId::new(10),
        });
        let variant = PrefabDefinition {
            id: PrefabId::new(2),
            schema_version: 1,
            display_name: "Assembler Variant".into(),
            parts: vec![],
            part_roles: vec![],
            variant: Some(PrefabVariantDelta {
                variant_id: "damaged".into(),
                base: base.id,
                removed_roles: vec!["gameplay-root".into()],
                overrides: vec![PrefabOverride {
                    target_role: "controller-alias".into(),
                    value: PrefabOverrideValue::Transform {
                        transform: PrefabTransform::IDENTITY,
                    },
                }],
            }),
        };

        let report = validate_prefab_registry(
            &PrefabRegistry {
                schema_version: 1,
                definitions: vec![base, variant],
            },
            &context(),
        );
        let codes: BTreeSet<_> = report.diagnostics.iter().map(|item| item.code).collect();
        assert!(codes.contains(&PrefabDiagnosticCode::UnsafePartRemoval));
        assert!(codes.contains(&PrefabDiagnosticCode::DeletedRoleReferenced));
    }

    #[test]
    fn variant_cycles_and_depth_fail_closed() {
        let variant = |id, base| PrefabDefinition {
            id: PrefabId::new(id),
            schema_version: 1,
            display_name: format!("Variant {id}"),
            parts: vec![],
            part_roles: vec![],
            variant: Some(PrefabVariantDelta {
                variant_id: format!("variant-{id}"),
                base: PrefabId::new(base),
                removed_roles: vec![],
                overrides: vec![],
            }),
        };
        let cycle_report = validate_prefab_registry(
            &PrefabRegistry {
                schema_version: 1,
                definitions: vec![variant(1, 2), variant(2, 1)],
            },
            &context(),
        );
        assert!(cycle_report
            .diagnostics
            .iter()
            .any(|d| d.code == PrefabDiagnosticCode::VariantCycle));

        let depth_report = validate_prefab_registry(
            &PrefabRegistry {
                schema_version: 1,
                definitions: vec![base_definition(), variant(2, 1), variant(3, 2)],
            },
            &context(),
        );
        assert!(depth_report
            .diagnostics
            .iter()
            .any(|d| d.code == PrefabDiagnosticCode::VariantDepthExceeded));
    }

    #[test]
    fn wrong_kind_assets_and_unknown_entity_definitions_are_classified() {
        let mut definition = base_definition();
        definition.parts[0].source = PrefabPartSource::VoxelObject {
            asset: "scene/factory-cell".into(),
        };
        definition.parts[1].source = PrefabPartSource::EntityDefinition {
            stable_id: "machine.missing".into(),
        };
        let report = validate_prefab_registry(
            &PrefabRegistry {
                schema_version: 1,
                definitions: vec![definition],
            },
            &context(),
        );
        let codes: BTreeSet<_> = report.diagnostics.iter().map(|d| d.code).collect();
        assert!(codes.contains(&PrefabDiagnosticCode::AssetKindMismatch));
        assert!(codes.contains(&PrefabDiagnosticCode::UnknownEntityDefinition));
    }
}
