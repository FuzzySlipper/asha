use crate::{GameplayModuleBehavior, GameplayModuleContext};
use protocol_game_extension::{GameplayContractRef, GameplayModuleManifest};
use rule_gameplay_fabric::{
    gameplay_module_payload_hash, register_standard_owner_events, FrozenGameplayViews,
    GameplayFabricCoordinator, GameplayHostError, GameplayInvocationCall, GameplayInvocationHost,
    GameplayModuleStateError, GameplayModuleStateRegistration, GameplayObserveReceipt,
    GameplayOwnerRoutingCall, GameplayOwnerRoutingOutput, GameplayProposalRouter,
    GameplayRuntimeLimits, GameplayViewSource,
};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use svc_gameplay_fabric::{
    GameplayEventCodecRegistration, GameplayFabricRegistry, GameplayFabricRegistryBuilder,
    GameplayLinkedProvider, GameplayProposalOwnerRegistration,
    GameplayReadViewProviderRegistration, GameplayRegistryBuildError,
    GameplayStateOwnerRegistration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayConfigurationFieldMetadata {
    pub name: String,
    pub value_type: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayConfigurationSchemaMetadata {
    pub module_id: String,
    pub configuration: GameplayContractRef,
    pub codec_id: String,
    pub fields: Vec<GameplayConfigurationFieldMetadata>,
}

pub struct GameplayStaticModuleProvider {
    pub manifest: GameplayModuleManifest,
    pub linked_provider: GameplayLinkedProvider,
    pub configuration_schemas: Vec<GameplayConfigurationSchemaMetadata>,
    event_codecs: Vec<GameplayEventCodecRegistration>,
    proposal_owners: Vec<GameplayProposalOwnerRegistration>,
    read_view_providers: Vec<GameplayReadViewProviderRegistration>,
    state_owners: Vec<GameplayStateOwnerRegistration>,
    state_adapters: Vec<GameplayModuleStateRegistration>,
    behavior: Box<dyn GameplayModuleBehavior>,
}

impl GameplayStaticModuleProvider {
    pub fn new(
        manifest: GameplayModuleManifest,
        linked_provider: GameplayLinkedProvider,
        behavior: impl GameplayModuleBehavior + 'static,
    ) -> Self {
        Self {
            manifest,
            linked_provider,
            configuration_schemas: Vec::new(),
            event_codecs: Vec::new(),
            proposal_owners: Vec::new(),
            read_view_providers: Vec::new(),
            state_owners: Vec::new(),
            state_adapters: Vec::new(),
            behavior: Box::new(behavior),
        }
    }

    pub fn linked_from_manifest(
        manifest: GameplayModuleManifest,
        behavior: impl GameplayModuleBehavior + 'static,
    ) -> Self {
        let module = &manifest.module_ref;
        let linked = GameplayLinkedProvider {
            provider_id: module.provider_id.clone(),
            module_id: module.module_id.clone(),
            version: module.version.clone(),
            contract_hash: module.contract_hash.clone(),
            artifact_hash: module.artifact_hash.clone(),
            sdk_hash: module.sdk_hash.clone(),
            source_hash: manifest.source_hash.clone(),
        };
        Self::new(manifest, linked, behavior)
    }

    pub fn event_codec(mut self, registration: GameplayEventCodecRegistration) -> Self {
        self.event_codecs.push(registration);
        self
    }

    pub fn proposal_owner(mut self, registration: GameplayProposalOwnerRegistration) -> Self {
        self.proposal_owners.push(registration);
        self
    }

    pub fn read_view_provider(
        mut self,
        registration: GameplayReadViewProviderRegistration,
    ) -> Self {
        self.read_view_providers.push(registration);
        self
    }

    pub fn state_owner(mut self, registration: GameplayStateOwnerRegistration) -> Self {
        self.state_owners.push(registration);
        self
    }

    pub fn state_adapter(mut self, registration: GameplayModuleStateRegistration) -> Self {
        self.state_adapters.push(registration);
        self
    }

    pub fn configuration_schema(mut self, schema: GameplayConfigurationSchemaMetadata) -> Self {
        self.configuration_schemas.push(schema);
        self
    }
}

#[derive(Debug)]
pub enum GameplayStaticCompositionError {
    DuplicateBehavior(String),
    InvalidConfigurationSchema(String),
    Registry(GameplayRegistryBuildError),
    StateAdapter(GameplayModuleStateError),
}

impl core::fmt::Display for GameplayStaticCompositionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for GameplayStaticCompositionError {}

#[derive(Default)]
pub struct GameplayStaticCompositionBuilder {
    providers: Vec<GameplayStaticModuleProvider>,
    include_standard_owner_events: bool,
}

impl GameplayStaticCompositionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_provider(&mut self, provider: GameplayStaticModuleProvider) -> &mut Self {
        self.providers.push(provider);
        self
    }

    /// Include the engine-owned asha event publisher/codecs so downstream
    /// modules can subscribe to semantic owner facts without private imports.
    /// This is explicit because pure module-unit compositions may not need them.
    pub fn include_standard_owner_events(&mut self) -> &mut Self {
        self.include_standard_owner_events = true;
        self
    }

    pub fn build(self) -> Result<GameplayStaticComposition, GameplayStaticCompositionError> {
        let mut registry_builder = GameplayFabricRegistryBuilder::new();
        if self.include_standard_owner_events {
            register_standard_owner_events(&mut registry_builder);
        }
        let mut behaviors = BTreeMap::new();
        let mut state_adapters = Vec::new();
        let mut configuration_schemas = Vec::new();
        for provider in self.providers {
            let module_id = provider.manifest.module_ref.module_id.clone();
            if behaviors
                .insert(module_id.clone(), provider.behavior)
                .is_some()
            {
                return Err(GameplayStaticCompositionError::DuplicateBehavior(module_id));
            }
            validate_configuration_schemas(&provider.manifest, &provider.configuration_schemas)?;
            configuration_schemas.extend(provider.configuration_schemas);
            state_adapters.extend(provider.state_adapters);
            registry_builder
                .register_module(provider.manifest)
                .register_linked_provider(provider.linked_provider);
            for codec in provider.event_codecs {
                registry_builder.register_event_codec_registration(codec);
            }
            for owner in provider.proposal_owners {
                registry_builder.register_proposal_owner(owner);
            }
            for view in provider.read_view_providers {
                registry_builder.register_read_view_provider(view);
            }
            for owner in provider.state_owners {
                registry_builder.register_state_owner(owner);
            }
        }
        let registry = Rc::new(
            registry_builder
                .build()
                .map_err(GameplayStaticCompositionError::Registry)?,
        );
        for adapter in &state_adapters {
            adapter
                .validate_against_registry(registry.as_ref())
                .map_err(GameplayStaticCompositionError::StateAdapter)?;
        }
        configuration_schemas.sort_by(|left, right| {
            (left.module_id.as_str(), left.configuration.key())
                .cmp(&(right.module_id.as_str(), right.configuration.key()))
        });
        Ok(GameplayStaticComposition {
            registry,
            host: GameplayStaticInvocationHost { behaviors },
            state_adapters,
            configuration_schemas,
        })
    }
}

pub struct GameplayStaticComposition {
    registry: Rc<GameplayFabricRegistry>,
    host: GameplayStaticInvocationHost,
    state_adapters: Vec<GameplayModuleStateRegistration>,
    configuration_schemas: Vec<GameplayConfigurationSchemaMetadata>,
}

impl GameplayStaticComposition {
    pub fn registry(&self) -> &GameplayFabricRegistry {
        self.registry.as_ref()
    }

    pub fn invocation_host(&self) -> &GameplayStaticInvocationHost {
        &self.host
    }

    pub fn configuration_schemas(&self) -> &[GameplayConfigurationSchemaMetadata] {
        &self.configuration_schemas
    }

    /// Executes one static Session Observe root for modules that emit only
    /// events and module-local facts. Shared proposals fail closed here; the
    /// owning RuntimeSession supplies its private owner router for them.
    pub fn observe_session_event(
        &self,
        event: protocol_game_extension::GameplayEventEnvelope,
    ) -> GameplayObserveReceipt {
        GameplayFabricCoordinator::new(&self.registry, limits_from_registry(&self.registry))
            .observe(
                event,
                &StaticSessionViews {
                    registry_digest: self.registry.registry_digest(),
                },
                &self.host,
                &mut RejectSharedProposals,
            )
    }

    pub fn into_parts(self) -> GameplayStaticCompositionParts {
        GameplayStaticCompositionParts {
            registry: self.registry,
            host: self.host,
            state_adapters: self.state_adapters,
            configuration_schemas: self.configuration_schemas,
        }
    }
}

struct StaticSessionViews<'a> {
    registry_digest: &'a str,
}

impl GameplayViewSource for StaticSessionViews<'_> {
    fn freeze(&self, root_id: &str, wave: u32) -> FrozenGameplayViews {
        let key = format!("{}|{root_id}|{wave}", self.registry_digest);
        FrozenGameplayViews {
            epoch: u64::from(wave),
            view_hash: gameplay_module_payload_hash(key.as_bytes()),
        }
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

fn limits_from_registry(registry: &GameplayFabricRegistry) -> GameplayRuntimeLimits {
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

pub struct GameplayStaticCompositionParts {
    pub registry: Rc<GameplayFabricRegistry>,
    pub host: GameplayStaticInvocationHost,
    pub state_adapters: Vec<GameplayModuleStateRegistration>,
    pub configuration_schemas: Vec<GameplayConfigurationSchemaMetadata>,
}

pub struct GameplayStaticInvocationHost {
    behaviors: BTreeMap<String, Box<dyn GameplayModuleBehavior>>,
}

impl GameplayInvocationHost for GameplayStaticInvocationHost {
    fn invoke(
        &self,
        call: &GameplayInvocationCall,
    ) -> Result<rule_gameplay_fabric::GameplayInvocationOutput, GameplayHostError> {
        let behavior = self
            .behaviors
            .get(&call.module_id)
            .ok_or_else(|| GameplayHostError {
                code: "missingStaticBehavior".to_owned(),
                message: format!("no behavior instance for module `{}`", call.module_id),
            })?;
        behavior
            .invoke(&GameplayModuleContext::new(call))
            .map(|actions| actions.finish())
            .map_err(Into::into)
    }
}

fn validate_configuration_schemas(
    manifest: &GameplayModuleManifest,
    schemas: &[GameplayConfigurationSchemaMetadata],
) -> Result<(), GameplayStaticCompositionError> {
    let mut seen = BTreeSet::new();
    for schema in schemas {
        if schema.module_id != manifest.module_ref.module_id
            || schema.configuration.namespace != manifest.module_ref.namespace
            || schema.configuration.version == 0
            || schema.configuration.schema_hash.trim().is_empty()
            || schema.codec_id.trim().is_empty()
            || !seen.insert(schema.configuration.key())
        {
            return Err(GameplayStaticCompositionError::InvalidConfigurationSchema(
                schema.configuration.key(),
            ));
        }
        let mut field_names = BTreeSet::new();
        if schema.fields.iter().any(|field| {
            field.name.trim().is_empty()
                || field.value_type.trim().is_empty()
                || !field_names.insert(field.name.as_str())
        }) {
            return Err(GameplayStaticCompositionError::InvalidConfigurationSchema(
                schema.configuration.key(),
            ));
        }
    }
    Ok(())
}
