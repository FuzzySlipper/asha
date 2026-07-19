//! Artifact classification for the project-bundle manifest.
//!
//! Every file in a project bundle is one of three classes (scene-capability-02,
//! "Recommended model"):
//!
//! * **durable** — required to load or diagnose the project (scene/current
//!   authority, asset lock, edits/snapshots, generator metadata).
//! * **generated** — reproducible from seed/version/params + edits.
//! * **cache** — disposable acceleration data (meshed geometry, collision
//!   projections, renderer-handle caches); deleting it never changes loaded
//!   authority.
//!
//! The `role` says *what* an artifact is; the `class` says how durable it is.

use crate::hash::BundleHash;

/// How durable an artifact is. Drives load requirements and cache disposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactClass {
    /// Required to load or diagnose the project.
    Durable,
    /// Reproducible from seed/version/params + edits.
    Generated,
    /// Disposable acceleration data; deletion never affects authority.
    Cache,
}

impl ArtifactClass {
    /// The on-disk discriminant.
    pub fn tag(self) -> &'static str {
        match self {
            ArtifactClass::Durable => "durable",
            ArtifactClass::Generated => "generated",
            ArtifactClass::Cache => "cache",
        }
    }

    /// Parse the on-disk discriminant.
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "durable" => Some(ArtifactClass::Durable),
            "generated" => Some(ArtifactClass::Generated),
            "cache" => Some(ArtifactClass::Cache),
            _ => None,
        }
    }

    /// Whether an artifact of this class must be present for an authority load.
    /// Cache artifacts are optional; durable and generated artifacts participate
    /// in the load plan (generated may be regenerated, but the manifest still
    /// lists it as a load input until regeneration is wired).
    pub fn is_load_required(self) -> bool {
        !matches!(self, ArtifactClass::Cache)
    }
}

/// What an artifact *is*. Stable string-tagged so the manifest, rather than a
/// host-side path convention, owns the complete runtime source closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactRole {
    /// The flat canonical scene document (`core-scene`).
    SceneDocument,
    /// The asset dependency lock (`core-assets` references resolved to versions).
    AssetLock,
    /// Durable reusable prefab definitions and their stable local part roles.
    PrefabRegistry,
    /// One typed ProjectContent document. The document body carries its closed
    /// kind and stable document id; the manifest only owns closure membership.
    ProjectContent,
    /// Durable stored EntityDefinition catalog input.
    EntityDefinitionCatalog,
    /// Durable stored material/catalog input.
    MaterialCatalog,
    /// A canonical stored voxel-volume resource used by authority, collision,
    /// and render projection.
    VoxelVolumeAsset,
    /// A future-extensible typed resource family. On disk this is encoded as
    /// `resource:<kind>`; unlike [`ArtifactRole::Other`], it is explicitly part
    /// of the understood resource namespace and may participate in a v2 load.
    Resource(String),
    /// A persisted current-authority session-state snapshot.
    SessionStateSnapshot,
    /// A voxel chunk snapshot (`rule-voxel-edit` persistence).
    VoxelChunkSnapshot,
    /// A voxel edit/replay log.
    VoxelEditLog,
    /// A durable voxel edit history/cursor timeline.
    VoxelEditHistory,
    /// A stored semantic annotation layer over a target voxel-volume asset.
    VoxelAnnotationLayer,
    /// Replay records / diagnostics for a session.
    ReplayRecord,
    /// Terrain generation metadata (seed/version/params).
    GeneratedMetadata,
    /// Meshed geometry / collision / renderer-handle cache.
    Cache,
    /// A role this build does not name; carried verbatim.
    Other(String),
}

impl ArtifactRole {
    /// The on-disk discriminant.
    pub fn tag(&self) -> &str {
        match self {
            ArtifactRole::SceneDocument => "sceneDocument",
            ArtifactRole::AssetLock => "assetLock",
            ArtifactRole::PrefabRegistry => "prefabRegistry",
            ArtifactRole::ProjectContent => "projectContent",
            ArtifactRole::EntityDefinitionCatalog => "entityDefinitionCatalog",
            ArtifactRole::MaterialCatalog => "materialCatalog",
            ArtifactRole::VoxelVolumeAsset => "voxelVolumeAsset",
            ArtifactRole::Resource(kind) => kind,
            ArtifactRole::SessionStateSnapshot => "sessionStateSnapshot",
            ArtifactRole::VoxelChunkSnapshot => "voxelChunkSnapshot",
            ArtifactRole::VoxelEditLog => "voxelEditLog",
            ArtifactRole::VoxelEditHistory => "voxelEditHistory",
            ArtifactRole::VoxelAnnotationLayer => "voxelAnnotationLayer",
            ArtifactRole::ReplayRecord => "replayRecord",
            ArtifactRole::GeneratedMetadata => "generatedMetadata",
            ArtifactRole::Cache => "cache",
            ArtifactRole::Other(s) => s,
        }
    }

    /// Parse the on-disk discriminant. Unknown tags become [`ArtifactRole::Other`].
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "sceneDocument" => ArtifactRole::SceneDocument,
            "assetLock" => ArtifactRole::AssetLock,
            "prefabRegistry" => ArtifactRole::PrefabRegistry,
            "projectContent" => ArtifactRole::ProjectContent,
            "entityDefinitionCatalog" => ArtifactRole::EntityDefinitionCatalog,
            "materialCatalog" => ArtifactRole::MaterialCatalog,
            "voxelVolumeAsset" => ArtifactRole::VoxelVolumeAsset,
            "sessionStateSnapshot" => ArtifactRole::SessionStateSnapshot,
            "voxelChunkSnapshot" => ArtifactRole::VoxelChunkSnapshot,
            "voxelEditLog" => ArtifactRole::VoxelEditLog,
            "voxelEditHistory" => ArtifactRole::VoxelEditHistory,
            "voxelAnnotationLayer" => ArtifactRole::VoxelAnnotationLayer,
            "replayRecord" => ArtifactRole::ReplayRecord,
            "generatedMetadata" => ArtifactRole::GeneratedMetadata,
            "cache" => ArtifactRole::Cache,
            other if other.starts_with("resource:") && other.len() > "resource:".len() => {
                ArtifactRole::Resource(other.to_string())
            }
            other => ArtifactRole::Other(other.to_string()),
        }
    }

    /// Whether this role names a manifest-understood runtime input rather than
    /// an opaque legacy extension.
    pub fn is_known_runtime_role(&self) -> bool {
        match self {
            ArtifactRole::Other(_) => false,
            ArtifactRole::Resource(tag) => tag.strip_prefix("resource:").is_some_and(|kind| {
                !kind.is_empty()
                    && kind
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '-')
            }),
            _ => true,
        }
    }
}

/// One row of the manifest's artifact table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactEntry {
    /// Bundle-relative path (forward slashes; canonical directory layout).
    pub path: String,
    pub class: ArtifactClass,
    pub role: ArtifactRole,
    /// Content hash for drift/replay diagnostics. Required for durable artifacts;
    /// optional (`None`) for cache artifacts that may be absent/rebuilt.
    pub content_hash: Option<BundleHash>,
}

impl ArtifactEntry {
    /// A durable artifact with its content hash computed from `bytes`.
    pub fn durable(path: impl Into<String>, role: ArtifactRole, bytes: &[u8]) -> Self {
        ArtifactEntry {
            path: path.into(),
            class: ArtifactClass::Durable,
            role,
            content_hash: Some(BundleHash::of(bytes)),
        }
    }

    /// A generated artifact with its content hash computed from `bytes`.
    pub fn generated(path: impl Into<String>, role: ArtifactRole, bytes: &[u8]) -> Self {
        ArtifactEntry {
            path: path.into(),
            class: ArtifactClass::Generated,
            role,
            content_hash: Some(BundleHash::of(bytes)),
        }
    }

    /// A cache artifact (no required hash; disposable).
    pub fn cache(path: impl Into<String>, role: ArtifactRole) -> Self {
        ArtifactEntry {
            path: path.into(),
            class: ArtifactClass::Cache,
            role,
            content_hash: None,
        }
    }
}
