//! Public, statically linked conformance runner for downstream gameplay modules.

#![forbid(unsafe_code)]

use asha_gameplay_module_sdk::{
    gameplay_module_payload_hash, GameplayContractRef, GameplayEventEnvelope,
    GameplayModuleBindingDiagnosticCode, GameplayModuleBindingRegistry, GameplayModuleStateScope,
    GameplayReadSelectorCapability, GameplayReadViewKind, GameplayStaticComposition,
    GameplayStaticCompositionError,
};
use core_ids::{RuntimeSessionId, SceneId, SceneNodeId};
use core_scene::{encode, SceneMetadata, SceneNode, SceneNodeKind, SceneTree};
use rule_gameplay_fabric::{
    verify_reaction_frame, FrozenGameplayViews, GameplayFabricCoordinator, GameplayFrozenRead,
    GameplayFrozenReadSet, GameplayModuleFact, GameplayModuleStateError, GameplayModuleStateStore,
    GameplayOwnerRoutingCall, GameplayOwnerRoutingOutput, GameplayProposalRouter,
    GameplayReactionFrame, GameplayReadAssemblyError, GameplayReadDiagnostic,
    GameplayReadDiagnosticCode, GameplayReadValue, GameplayRuntimeLimits, GameplayViewSource,
};
use rule_project_bundle::{
    execute_load_plan, BundleArtifacts, GameplayBindingActivationError,
    GameplayBindingEntityTargets, GameplayBoundProjectBundleSession, ProjectBundleLoadResult,
};
use serde::{Deserialize, Serialize};
use svc_serialization::{LoadPlan, LoadStep};

pub const GAMEPLAY_MODULE_CONFORMANCE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameplayModuleConformanceProject {
    pub schema_version: u32,
    pub project_id: String,
    pub scene_id: u64,
    pub runtime_session_id: u64,
    pub consumer_needs: Vec<String>,
    pub gameplay_module_bindings: GameplayModuleBindingRegistry,
    pub declared_reads: Vec<GameplayModuleConformanceDeclaredRead>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameplayModuleConformanceDeclaredRead {
    pub request_id: String,
    pub module_id: String,
    pub invocation_id: String,
    pub view: GameplayContractRef,
    pub scope: GameplayModuleStateScope,
    pub fields: Vec<String>,
}

pub struct GameplayModuleConformanceCase {
    pub project_bundle_json: String,
    pub composition: fn() -> Result<GameplayStaticComposition, GameplayStaticCompositionError>,
    pub events: Vec<GameplayEventEnvelope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayModuleConformanceCheck {
    pub id: String,
    pub passed: bool,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayModuleConformanceGap {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayModuleConformanceReport {
    pub schema_version: u32,
    pub valid: bool,
    pub project_id: String,
    pub consumer_needs: Vec<String>,
    pub module_ids: Vec<String>,
    pub registry_digest: String,
    pub registry_topology: String,
    pub binding_registry_hash: String,
    pub activation_receipt_hash: String,
    pub initial_state_hash: String,
    pub final_state_hash: String,
    pub snapshot_hash: String,
    pub reaction_frames: Vec<GameplayReactionFrame>,
    pub checks: Vec<GameplayModuleConformanceCheck>,
    pub gaps: Vec<GameplayModuleConformanceGap>,
    pub trace: String,
}

impl GameplayModuleConformanceReport {
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|value| value + "\n")
    }
}

#[derive(Debug)]
pub enum GameplayModuleConformanceError {
    InvalidProject(String),
    Load(String),
    State(GameplayModuleStateError),
    Snapshot(GameplayBindingActivationError),
    Composition(GameplayStaticCompositionError),
}

impl core::fmt::Display for GameplayModuleConformanceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for GameplayModuleConformanceError {}

struct CompletedRun {
    activation_receipt_hash: String,
    initial_state_hash: String,
    final_state_hash: String,
    snapshot_text: String,
    snapshot_hash: String,
    initial_snapshot_text: String,
    frames: Vec<GameplayReactionFrame>,
    accepted_facts: Vec<GameplayModuleFact>,
    checks: Vec<GameplayModuleConformanceCheck>,
    gaps: Vec<GameplayModuleConformanceGap>,
    module_ids: Vec<String>,
    registry_digest: String,
    registry_topology: String,
}

pub fn run_gameplay_module_conformance(
    case: GameplayModuleConformanceCase,
) -> Result<GameplayModuleConformanceReport, GameplayModuleConformanceError> {
    let project: GameplayModuleConformanceProject = serde_json::from_str(&case.project_bundle_json)
        .map_err(|error| GameplayModuleConformanceError::InvalidProject(error.to_string()))?;
    if project.schema_version != GAMEPLAY_MODULE_CONFORMANCE_SCHEMA_VERSION
        || project.project_id.trim().is_empty()
        || project.scene_id == 0
        || project.runtime_session_id == 0
        || project.consumer_needs.is_empty()
        || project.declared_reads.is_empty()
        || case.events.is_empty()
    {
        return Err(GameplayModuleConformanceError::InvalidProject(
            "schema, project/session identity, consumer needs, declared reads, and events are required"
                .to_owned(),
        ));
    }

    let first = execute_case(&project, case.composition, &case.events)?;
    if first.initial_snapshot_text.is_empty() {
        let checks = first.checks.clone();
        let gaps = first.gaps.clone();
        let trace = render_trace(&project, &first, &checks, &gaps);
        return Ok(GameplayModuleConformanceReport {
            schema_version: GAMEPLAY_MODULE_CONFORMANCE_SCHEMA_VERSION,
            valid: false,
            project_id: project.project_id,
            consumer_needs: project.consumer_needs,
            module_ids: first.module_ids,
            registry_digest: first.registry_digest,
            registry_topology: first.registry_topology,
            binding_registry_hash: project.gameplay_module_bindings.registry_hash,
            activation_receipt_hash: first.activation_receipt_hash,
            initial_state_hash: first.initial_state_hash,
            final_state_hash: first.final_state_hash,
            snapshot_hash: first.snapshot_hash,
            reaction_frames: first.frames,
            checks,
            gaps,
            trace,
        });
    }
    let second = execute_case(&project, case.composition, &case.events)?;
    let mut checks = first.checks.clone();
    let mut gaps = first.gaps.clone();

    let deterministic = first.frames.len() == second.frames.len()
        && first
            .frames
            .iter()
            .zip(&second.frames)
            .all(|(expected, actual)| verify_reaction_frame(expected, actual).is_empty())
        && first.final_state_hash == second.final_state_hash
        && first.snapshot_hash == second.snapshot_hash;
    checks.push(check(
        "verificationReplay",
        deterministic,
        first
            .frames
            .iter()
            .map(|frame| frame.frame_hash.clone())
            .collect(),
    ));
    if !deterministic {
        gaps.push(gap(
            "verificationReplayDiverged",
            "reactionFrames",
            "the second real invocation run did not reproduce frames, state, and snapshot hashes",
        ));
    }

    let playback_state_hash = playback_facts(
        &project,
        case.composition,
        &first.initial_snapshot_text,
        &first.accepted_facts,
    )?;
    let playback_matches = playback_state_hash == first.final_state_hash;
    checks.push(check(
        "recordedFactPlayback",
        playback_matches,
        vec![playback_state_hash],
    ));
    if !playback_matches {
        gaps.push(gap(
            "playbackDiverged",
            "moduleState",
            "recorded accepted facts did not reconstruct final module state",
        ));
    }

    let restored = restore_final_snapshot(&project, case.composition, &first.snapshot_text)?;
    let restore_matches = restored == first.final_state_hash;
    checks.push(check("saveReload", restore_matches, vec![restored]));
    if !restore_matches {
        gaps.push(gap(
            "saveReloadDiverged",
            "session/gameplay-modules.snapshot.json",
            "saved ProjectBundle gameplay state did not restore to the same state hash",
        ));
    }

    let valid = gaps.is_empty() && checks.iter().all(|item| item.passed);
    let trace = render_trace(&project, &first, &checks, &gaps);
    Ok(GameplayModuleConformanceReport {
        schema_version: GAMEPLAY_MODULE_CONFORMANCE_SCHEMA_VERSION,
        valid,
        project_id: project.project_id,
        consumer_needs: project.consumer_needs,
        module_ids: first.module_ids,
        registry_digest: first.registry_digest,
        registry_topology: first.registry_topology,
        binding_registry_hash: project.gameplay_module_bindings.registry_hash,
        activation_receipt_hash: first.activation_receipt_hash,
        initial_state_hash: first.initial_state_hash,
        final_state_hash: first.final_state_hash,
        snapshot_hash: first.snapshot_hash,
        reaction_frames: first.frames,
        checks,
        gaps,
        trace,
    })
}

fn execute_case(
    project: &GameplayModuleConformanceProject,
    composition_factory: fn() -> Result<GameplayStaticComposition, GameplayStaticCompositionError>,
    events: &[GameplayEventEnvelope],
) -> Result<CompletedRun, GameplayModuleConformanceError> {
    let bundle = load_bundle(project)?;
    let composition = match composition_factory() {
        Ok(composition) => composition,
        Err(error) => return Ok(composition_failure(project, error)),
    };
    let mut session = match GameplayBoundProjectBundleSession::activate(
        bundle,
        composition,
        project.gameplay_module_bindings.clone(),
        &GameplayBindingEntityTargets::new(),
    ) {
        Ok(session) => session,
        Err(GameplayBindingActivationError::Invalid { diagnostics }) => {
            return Ok(activation_failure(project, diagnostics));
        }
        Err(GameplayBindingActivationError::State(error)) => {
            return Ok(activation_state_failure(project, error));
        }
        Err(error) => return Err(GameplayModuleConformanceError::Snapshot(error)),
    };
    let registry = session.registry().readout().clone();
    let initial_state_hash = session.module_state.state_hash();
    let initial_snapshot_text = session
        .compose_gameplay_session_snapshot()
        .map_err(GameplayModuleConformanceError::Snapshot)?
        .text;
    let mut frames = Vec::new();
    let mut accepted_facts = Vec::new();
    let mut checks = vec![check(
        "compositionAndBootstrap",
        true,
        vec![
            registry.registry_digest.clone(),
            session.activation.receipt_hash.clone(),
        ],
    )];
    let mut gaps = Vec::new();
    for event in events {
        let state_before = session.module_state.state_hash();
        let observe = GameplayFabricCoordinator::new(
            session.registry(),
            limits_from_registry(session.registry()),
        )
        .observe(
            event.clone(),
            &ConformanceViews {
                registry: session.registry(),
                state: &session.module_state,
                reads: &project.declared_reads,
            },
            session.invocation_host(),
            &mut RejectSharedProposals,
        );
        let mut applied = Vec::new();
        for fact in &observe.module_facts {
            match session.module_state.apply_fact(fact.clone()) {
                Ok(_) => {
                    accepted_facts.push(fact.clone());
                    applied.push(fact.clone());
                }
                Err(error) => gaps.push(gap(
                    state_error_code(&error),
                    "moduleFacts",
                    &format!("module fact rejected: {error:?}"),
                )),
            }
        }
        for diagnostic in &observe.diagnostics {
            gaps.push(gap(
                runtime_diagnostic_code(diagnostic.code),
                &diagnostic.path,
                &diagnostic.message,
            ));
        }
        let state_after = session.module_state.state_hash();
        let final_session_hash = session
            .module_state
            .final_session_hash(&session.activation.receipt_hash);
        frames.push(GameplayReactionFrame::from_observe(
            session.registry(),
            &observe,
            Vec::new(),
            &applied,
            state_before,
            state_after,
            final_session_hash,
        ));
    }
    checks.push(check(
        "actualInvocation",
        frames.iter().all(|frame| !frame.invocations.is_empty()),
        frames
            .iter()
            .flat_map(|frame| frame.invocation_output_hashes.clone())
            .collect(),
    ));
    checks.push(check(
        "typedFrozenViews",
        frames
            .iter()
            .all(|frame| !frame.frozen_view_hashes.is_empty()),
        frames
            .iter()
            .flat_map(|frame| frame.frozen_view_hashes.clone())
            .collect(),
    ));
    checks.push(check(
        "declaredReadDelivery",
        !project.declared_reads.is_empty()
            && frames.iter().all(|frame| {
                frame
                    .diagnostic_codes
                    .iter()
                    .all(|code| code != "ReadAssemblyFailed")
            }),
        project
            .declared_reads
            .iter()
            .map(|read| read.request_id.clone())
            .collect(),
    ));
    checks.push(check(
        "moduleStateFacts",
        !accepted_facts.is_empty(),
        accepted_facts
            .iter()
            .map(|fact| fact.payload_hash.clone())
            .collect(),
    ));
    for item in &checks {
        if !item.passed {
            gaps.push(gap(
                "requiredEvidenceMissing",
                &item.id,
                "the real run did not produce the required evidence",
            ));
        }
    }
    let final_state_hash = session.module_state.state_hash();
    let snapshot = session
        .compose_gameplay_session_snapshot()
        .map_err(GameplayModuleConformanceError::Snapshot)?;
    let snapshot_hash = snapshot
        .entry
        .content_hash
        .map(|hash| hash.to_hex())
        .unwrap_or_default();
    Ok(CompletedRun {
        activation_receipt_hash: session.activation.receipt_hash,
        initial_state_hash,
        final_state_hash,
        snapshot_text: snapshot.text,
        snapshot_hash,
        initial_snapshot_text,
        frames,
        accepted_facts,
        checks,
        gaps,
        module_ids: registry.module_ids,
        registry_digest: registry.registry_digest,
        registry_topology: registry.topology_dump,
    })
}

fn playback_facts(
    project: &GameplayModuleConformanceProject,
    composition_factory: fn() -> Result<GameplayStaticComposition, GameplayStaticCompositionError>,
    initial_snapshot: &str,
    facts: &[GameplayModuleFact],
) -> Result<String, GameplayModuleConformanceError> {
    let mut session = GameplayBoundProjectBundleSession::restore(
        load_bundle(project)?,
        composition_factory().map_err(GameplayModuleConformanceError::Composition)?,
        project.gameplay_module_bindings.clone(),
        &GameplayBindingEntityTargets::new(),
        initial_snapshot,
    )
    .map_err(GameplayModuleConformanceError::Snapshot)?;
    for fact in facts {
        session
            .module_state
            .apply_fact(fact.clone())
            .map_err(GameplayModuleConformanceError::State)?;
    }
    Ok(session.module_state.state_hash())
}

fn restore_final_snapshot(
    project: &GameplayModuleConformanceProject,
    composition_factory: fn() -> Result<GameplayStaticComposition, GameplayStaticCompositionError>,
    snapshot: &str,
) -> Result<String, GameplayModuleConformanceError> {
    GameplayBoundProjectBundleSession::restore(
        load_bundle(project)?,
        composition_factory().map_err(GameplayModuleConformanceError::Composition)?,
        project.gameplay_module_bindings.clone(),
        &GameplayBindingEntityTargets::new(),
        snapshot,
    )
    .map(|session| session.module_state.state_hash())
    .map_err(GameplayModuleConformanceError::Snapshot)
}

fn load_bundle(
    project: &GameplayModuleConformanceProject,
) -> Result<ProjectBundleLoadResult, GameplayModuleConformanceError> {
    let scene_id = SceneId::new(project.scene_id);
    let scene = SceneTree {
        id: scene_id,
        schema_version: 1,
        metadata: SceneMetadata {
            name: Some(project.project_id.clone()),
            authoring_format_version: 1,
        },
        dependencies: Vec::new(),
        roots: vec![SceneNode::leaf(
            SceneNodeId::new(1),
            SceneNodeKind::EmptyGroup,
        )],
    };
    let plan = LoadPlan {
        steps: vec![
            LoadStep::ValidateVersions {
                bundle_schema_version: 1,
                protocol_version: 1,
            },
            LoadStep::LoadAssetLock {
                artifact: "assets/lock.json".to_owned(),
                asset_count: 0,
            },
            LoadStep::LoadSceneDocument {
                artifact: "scene/scene.json".to_owned(),
                scene: scene_id,
            },
            LoadStep::BootstrapScene {
                scene: scene_id,
                runtime_session: RuntimeSessionId::new(project.runtime_session_id),
            },
            LoadStep::ValidateFinalState,
        ],
    };
    let artifacts = BundleArtifacts::new()
        .with_artifact("assets/lock.json", "{ \"entries\": [] }\n")
        .with_artifact("scene/scene.json", encode(&scene.to_flat()));
    execute_load_plan(&plan, &artifacts)
        .map_err(|error| GameplayModuleConformanceError::Load(format!("{error:?}")))
}

fn activation_failure(
    project: &GameplayModuleConformanceProject,
    diagnostics: Vec<asha_gameplay_module_sdk::GameplayModuleBindingDiagnostic>,
) -> CompletedRun {
    let gaps = diagnostics
        .into_iter()
        .map(|diagnostic| {
            gap(
                diagnostic.code.as_str(),
                &diagnostic.path,
                &diagnostic.message,
            )
        })
        .collect();
    CompletedRun {
        activation_receipt_hash: String::new(),
        initial_state_hash: String::new(),
        final_state_hash: String::new(),
        snapshot_text: String::new(),
        snapshot_hash: String::new(),
        initial_snapshot_text: String::new(),
        frames: Vec::new(),
        accepted_facts: Vec::new(),
        checks: vec![check(
            "compositionAndBootstrap",
            false,
            vec![project.gameplay_module_bindings.registry_hash.clone()],
        )],
        gaps,
        module_ids: Vec::new(),
        registry_digest: String::new(),
        registry_topology: String::new(),
    }
}

fn activation_state_failure(
    project: &GameplayModuleConformanceProject,
    error: GameplayModuleStateError,
) -> CompletedRun {
    CompletedRun {
        activation_receipt_hash: String::new(),
        initial_state_hash: String::new(),
        final_state_hash: String::new(),
        snapshot_text: String::new(),
        snapshot_hash: String::new(),
        initial_snapshot_text: String::new(),
        frames: Vec::new(),
        accepted_facts: Vec::new(),
        checks: vec![check(
            "compositionAndBootstrap",
            false,
            vec![project.gameplay_module_bindings.registry_hash.clone()],
        )],
        gaps: vec![gap(
            state_error_code(&error),
            "gameplayModuleBindings",
            &format!("atomic module-state bootstrap rejected: {error:?}"),
        )],
        module_ids: Vec::new(),
        registry_digest: String::new(),
        registry_topology: String::new(),
    }
}

fn composition_failure(
    project: &GameplayModuleConformanceProject,
    error: GameplayStaticCompositionError,
) -> CompletedRun {
    let gaps = match error {
        GameplayStaticCompositionError::Registry(error) => error
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                gap(
                    diagnostic.code.as_str(),
                    &diagnostic.path,
                    &diagnostic.message,
                )
            })
            .collect(),
        GameplayStaticCompositionError::DuplicateBehavior(module_id) => vec![gap(
            "duplicateBehavior",
            "composition",
            &format!("duplicate behavior for module {module_id}"),
        )],
        GameplayStaticCompositionError::InvalidConfigurationSchema(schema) => vec![gap(
            "invalidConfigurationSchema",
            "composition",
            &format!("invalid configuration schema {schema}"),
        )],
        GameplayStaticCompositionError::StateAdapter(error) => vec![gap(
            state_error_code(&error),
            "composition.stateAdapter",
            &format!("state adapter rejected: {error:?}"),
        )],
    };
    CompletedRun {
        activation_receipt_hash: String::new(),
        initial_state_hash: String::new(),
        final_state_hash: String::new(),
        snapshot_text: String::new(),
        snapshot_hash: String::new(),
        initial_snapshot_text: String::new(),
        frames: Vec::new(),
        accepted_facts: Vec::new(),
        checks: vec![check(
            "compositionAndBootstrap",
            false,
            vec![project.gameplay_module_bindings.registry_hash.clone()],
        )],
        gaps,
        module_ids: Vec::new(),
        registry_digest: String::new(),
        registry_topology: String::new(),
    }
}

fn check(id: &str, passed: bool, evidence: Vec<String>) -> GameplayModuleConformanceCheck {
    GameplayModuleConformanceCheck {
        id: id.to_owned(),
        passed,
        evidence,
    }
}

fn gap(code: &str, path: &str, message: &str) -> GameplayModuleConformanceGap {
    GameplayModuleConformanceGap {
        code: code.to_owned(),
        path: path.to_owned(),
        message: message.to_owned(),
    }
}

fn state_error_code(error: &GameplayModuleStateError) -> &'static str {
    match error {
        GameplayModuleStateError::DuplicateAdapter => "duplicateAdapter",
        GameplayModuleStateError::MissingAdapter => "missingAdapter",
        GameplayModuleStateError::MissingOwner => "missingOwner",
        GameplayModuleStateError::OwnerMismatch => "ownerMismatch",
        GameplayModuleStateError::UndeclaredState => "undeclaredState",
        GameplayModuleStateError::UndeclaredFact => "undeclaredFact",
        GameplayModuleStateError::UndeclaredView => "undeclaredView",
        GameplayModuleStateError::ForeignModule => "foreignModule",
        GameplayModuleStateError::DuplicateInitialization => "duplicateInitialization",
        GameplayModuleStateError::InvalidMigration => "invalidMigration",
        GameplayModuleStateError::UnknownState => "unknownState",
        GameplayModuleStateError::StaleRevision => "staleRevision",
        GameplayModuleStateError::PayloadHashMismatch => "payloadHashMismatch",
        GameplayModuleStateError::AdapterRejected(_) => "adapterRejected",
        GameplayModuleStateError::DuplicateFact => "duplicateFact",
        GameplayModuleStateError::InvalidSnapshot(_) => "invalidSnapshot",
    }
}

fn runtime_diagnostic_code(
    code: rule_gameplay_fabric::GameplayRuntimeDiagnosticCode,
) -> &'static str {
    use rule_gameplay_fabric::GameplayRuntimeDiagnosticCode as Code;
    match code {
        Code::UnknownEvent => "unknownEvent",
        Code::UndeclaredInvocation => "undeclaredInvocation",
        Code::UndeclaredEvent => "undeclaredEvent",
        Code::UndeclaredProposal => "undeclaredProposal",
        Code::UndeclaredModuleFact => "undeclaredModuleFact",
        Code::MissingProposalOwner => "missingProposalOwner",
        Code::ReadAssemblyFailed => "readAssemblyFailed",
        Code::HostFailure => "hostFailure",
        Code::WaveBudgetExceeded => "waveBudgetExceeded",
        Code::EventBudgetExceeded => "eventBudgetExceeded",
        Code::ProposalBudgetExceeded => "proposalBudgetExceeded",
        Code::InvocationBudgetExceeded => "invocationBudgetExceeded",
        Code::PayloadBudgetExceeded => "payloadBudgetExceeded",
        Code::InvocationOutputBudgetExceeded => "invocationOutputBudgetExceeded",
        Code::SubscriptionDeliveryBudgetExceeded => "subscriptionDeliveryBudgetExceeded",
        Code::UnexpectedDecisionOutput => "unexpectedDecisionOutput",
        Code::MissingDecisionOutput => "missingDecisionOutput",
        Code::GuardRejected => "guardRejected",
        Code::WorkspaceContractMismatch => "workspaceContractMismatch",
        Code::WorkspaceHashMismatch => "workspaceHashMismatch",
        Code::ContinuationRequired => "continuationRequired",
        Code::ContinuationMismatch => "continuationMismatch",
        Code::ContinuationUnavailable => "continuationUnavailable",
        Code::StaleDecision => "staleDecision",
        Code::ReactionCancelled => "reactionCancelled",
        Code::ReactionSuspended => "reactionSuspended",
        Code::OwnerRejected => "ownerRejected",
    }
}

fn render_trace(
    project: &GameplayModuleConformanceProject,
    run: &CompletedRun,
    checks: &[GameplayModuleConformanceCheck],
    gaps: &[GameplayModuleConformanceGap],
) -> String {
    let mut lines = vec![
        format!("project {}", project.project_id),
        format!("registry {}", run.registry_digest),
        format!(
            "bindings {}",
            project.gameplay_module_bindings.registry_hash
        ),
    ];
    for item in checks {
        lines.push(format!(
            "check {} {} {}",
            item.id,
            if item.passed { "PASS" } else { "FAIL" },
            item.evidence.join(",")
        ));
    }
    for item in gaps {
        lines.push(format!("gap {} {} {}", item.code, item.path, item.message));
    }
    lines.push(format!(
        "result {}",
        if gaps.is_empty() { "PASS" } else { "FAIL" }
    ));
    lines.join("\n") + "\n"
}

pub fn binding_diagnostic_code(code: GameplayModuleBindingDiagnosticCode) -> &'static str {
    code.as_str()
}

struct ConformanceViews<'a> {
    registry: &'a svc_gameplay_fabric::GameplayFabricRegistry,
    state: &'a GameplayModuleStateStore,
    reads: &'a [GameplayModuleConformanceDeclaredRead],
}

impl GameplayViewSource for ConformanceViews<'_> {
    fn freeze(&self, root_id: &str, wave: u32) -> FrozenGameplayViews {
        let value = format!("{}|{root_id}|{wave}", self.registry.registry_digest());
        FrozenGameplayViews {
            epoch: u64::from(wave),
            view_hash: gameplay_module_payload_hash(value.as_bytes()),
        }
    }

    fn freeze_declared_reads(
        &self,
        module_id: &str,
        invocation_id: &str,
        event: &GameplayEventEnvelope,
    ) -> Result<Option<GameplayFrozenReadSet>, GameplayReadAssemblyError> {
        let requested = self
            .reads
            .iter()
            .filter(|read| read.module_id == module_id && read.invocation_id == invocation_id)
            .collect::<Vec<_>>();
        if requested.is_empty() {
            return Ok(None);
        }
        let manifest = self.registry.module(module_id).ok_or_else(|| {
            read_error(
                GameplayReadDiagnosticCode::UnknownModule,
                "",
                "declared conformance read names an unknown module",
            )
        })?;
        let mut reads = Vec::new();
        for request in requested {
            let requirement = manifest
                .read_views
                .iter()
                .find(|requirement| requirement.view == request.view)
                .ok_or_else(|| {
                    read_error(
                        GameplayReadDiagnosticCode::UndeclaredRead,
                        &request.request_id,
                        "conformance read is absent from the module manifest",
                    )
                })?;
            let provider = self
                .registry
                .read_view_provider(&request.view)
                .ok_or_else(|| {
                    read_error(
                        GameplayReadDiagnosticCode::MissingProvider,
                        &request.request_id,
                        "conformance read has no closed-registry provider",
                    )
                })?;
            if requirement.kind != GameplayReadViewKind::ModuleNamed
                || provider.kind != GameplayReadViewKind::ModuleNamed
                || requirement.provider_id != provider.provider_id
                || !requirement
                    .selector_capabilities
                    .contains(&GameplayReadSelectorCapability::ModuleStateScope)
                || request.fields.iter().any(|field| {
                    !requirement.fields.contains(field) || !provider.fields.contains(field)
                })
                || requirement.max_items == 0
                || provider.max_items == 0
            {
                return Err(read_error(
                    GameplayReadDiagnosticCode::ProviderMismatch,
                    &request.request_id,
                    "declared read provider, kind, selector, fields, or quota drifted",
                ));
            }
            let named = self
                .state
                .named_view_by_contract(&request.view, &request.scope)
                .map_err(|error| {
                    read_error(
                        GameplayReadDiagnosticCode::MissingModuleView,
                        &request.request_id,
                        &format!("module named view rejected: {error:?}"),
                    )
                })?;
            reads.push(GameplayFrozenRead {
                request_id: request.request_id.clone(),
                view: request.view.clone(),
                provider_id: named.provider_id,
                fields: request.fields.clone(),
                value_hash: named.view_hash.clone(),
                value: GameplayReadValue::ModuleNamed {
                    scope: named.scope,
                    revision: named.revision,
                    canonical_payload: named.canonical_payload,
                    view_hash: named.view_hash,
                },
            });
        }
        reads.sort_by(|left, right| left.request_id.cmp(&right.request_id));
        let mut frozen = GameplayFrozenReadSet {
            registry_digest: self.registry.registry_digest().to_owned(),
            module_id: module_id.to_owned(),
            invocation_id: invocation_id.to_owned(),
            event_id: event.event_id.clone(),
            wave: event.wave,
            reads,
            read_set_hash: String::new(),
        };
        frozen.read_set_hash = gameplay_module_payload_hash(
            &serde_json::to_vec(&frozen).expect("conformance read set serializes"),
        );
        Ok(Some(frozen))
    }
}

struct RejectSharedProposals;

impl GameplayProposalRouter for RejectSharedProposals {
    fn route(&mut self, _call: &GameplayOwnerRoutingCall) -> GameplayOwnerRoutingOutput {
        GameplayOwnerRoutingOutput {
            accepted: false,
            diagnostic_codes: vec!["privateOwnerRouterRequired".to_owned()],
            ..GameplayOwnerRoutingOutput::default()
        }
    }
}

fn limits_from_registry(
    registry: &svc_gameplay_fabric::GameplayFabricRegistry,
) -> GameplayRuntimeLimits {
    let mut limits = GameplayRuntimeLimits {
        max_waves: 1,
        max_events_per_root: 1,
        max_proposals_per_root: 1,
        max_invocations_per_root: 1,
        max_payload_bytes_per_root: 1,
    };
    for module_id in registry.module_order() {
        let budget = &registry
            .module(module_id)
            .expect("closed module order")
            .budget;
        limits.max_waves = limits.max_waves.max(budget.max_waves);
        limits.max_events_per_root = limits
            .max_events_per_root
            .saturating_add(budget.max_events_per_root);
        limits.max_proposals_per_root = limits
            .max_proposals_per_root
            .saturating_add(budget.max_proposals_per_root);
        limits.max_invocations_per_root = limits
            .max_invocations_per_root
            .saturating_add(budget.max_invocations_per_root);
        limits.max_payload_bytes_per_root = limits
            .max_payload_bytes_per_root
            .saturating_add(budget.max_payload_bytes_per_root);
    }
    limits
}

fn read_error(
    code: GameplayReadDiagnosticCode,
    request_id: &str,
    message: &str,
) -> GameplayReadAssemblyError {
    GameplayReadAssemblyError {
        diagnostics: vec![GameplayReadDiagnostic {
            code,
            request_id: request_id.to_owned(),
            message: message.to_owned(),
        }],
    }
}
