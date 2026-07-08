use core_ids::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Absent,
    Active,
    Despawning,
    Despawned,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleTransitionKind {
    Spawned,
    DespawnStarted,
    DespawnCompleted,
    TerminalMarked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleTransition {
    pub entity: EntityId,
    pub kind: LifecycleTransitionKind,
    pub from: LifecycleState,
    pub to: LifecycleState,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleRejection {
    AlreadySpawned { state: LifecycleState },
    NotActive { state: LifecycleState },
    NotDespawning { state: LifecycleState },
    DespawnedIsFinal,
    TerminalIsFinal,
    EmptyTerminalReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleCell {
    entity: EntityId,
    state: LifecycleState,
    revision: u64,
    terminal_reason: Option<String>,
}

impl LifecycleCell {
    pub fn absent(entity: EntityId) -> Self {
        Self {
            entity,
            state: LifecycleState::Absent,
            revision: 0,
            terminal_reason: None,
        }
    }

    pub fn entity(&self) -> EntityId {
        self.entity
    }

    pub fn state(&self) -> LifecycleState {
        self.state
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn terminal_reason(&self) -> Option<&str> {
        self.terminal_reason.as_deref()
    }

    pub fn spawn(&mut self) -> Result<LifecycleTransition, LifecycleRejection> {
        match self.state {
            LifecycleState::Absent => {
                Ok(self.accept(LifecycleTransitionKind::Spawned, LifecycleState::Active))
            }
            LifecycleState::Despawned => Err(LifecycleRejection::DespawnedIsFinal),
            LifecycleState::Terminal => Err(LifecycleRejection::TerminalIsFinal),
            state => Err(LifecycleRejection::AlreadySpawned { state }),
        }
    }

    pub fn begin_despawn(&mut self) -> Result<LifecycleTransition, LifecycleRejection> {
        match self.state {
            LifecycleState::Active => Ok(self.accept(
                LifecycleTransitionKind::DespawnStarted,
                LifecycleState::Despawning,
            )),
            LifecycleState::Despawned => Err(LifecycleRejection::DespawnedIsFinal),
            LifecycleState::Terminal => Err(LifecycleRejection::TerminalIsFinal),
            state => Err(LifecycleRejection::NotActive { state }),
        }
    }

    pub fn complete_despawn(&mut self) -> Result<LifecycleTransition, LifecycleRejection> {
        match self.state {
            LifecycleState::Despawning => Ok(self.accept(
                LifecycleTransitionKind::DespawnCompleted,
                LifecycleState::Despawned,
            )),
            LifecycleState::Despawned => Err(LifecycleRejection::DespawnedIsFinal),
            LifecycleState::Terminal => Err(LifecycleRejection::TerminalIsFinal),
            state => Err(LifecycleRejection::NotDespawning { state }),
        }
    }

    pub fn mark_terminal(
        &mut self,
        reason: impl Into<String>,
    ) -> Result<LifecycleTransition, LifecycleRejection> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(LifecycleRejection::EmptyTerminalReason);
        }
        match self.state {
            LifecycleState::Active | LifecycleState::Despawning => {
                self.terminal_reason = Some(reason);
                Ok(self.accept(
                    LifecycleTransitionKind::TerminalMarked,
                    LifecycleState::Terminal,
                ))
            }
            LifecycleState::Despawned => Err(LifecycleRejection::DespawnedIsFinal),
            LifecycleState::Terminal => Err(LifecycleRejection::TerminalIsFinal),
            state => Err(LifecycleRejection::NotActive { state }),
        }
    }

    fn accept(&mut self, kind: LifecycleTransitionKind, to: LifecycleState) -> LifecycleTransition {
        let from = self.state;
        self.state = to;
        self.revision = self.revision.saturating_add(1);
        LifecycleTransition {
            entity: self.entity,
            kind,
            from,
            to,
            revision: self.revision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity() -> EntityId {
        EntityId::new(7)
    }

    #[test]
    fn spawn_and_despawn_are_explicit_product_neutral_transitions() {
        let mut lifecycle = LifecycleCell::absent(entity());

        let spawned = lifecycle.spawn().expect("spawn");
        assert_eq!(spawned.entity, entity());
        assert_eq!(spawned.kind, LifecycleTransitionKind::Spawned);
        assert_eq!(spawned.from, LifecycleState::Absent);
        assert_eq!(spawned.to, LifecycleState::Active);
        assert_eq!(lifecycle.state(), LifecycleState::Active);

        let despawning = lifecycle.begin_despawn().expect("begin despawn");
        assert_eq!(despawning.kind, LifecycleTransitionKind::DespawnStarted);
        assert_eq!(despawning.from, LifecycleState::Active);
        assert_eq!(despawning.to, LifecycleState::Despawning);

        let despawned = lifecycle.complete_despawn().expect("complete despawn");
        assert_eq!(despawned.kind, LifecycleTransitionKind::DespawnCompleted);
        assert_eq!(despawned.from, LifecycleState::Despawning);
        assert_eq!(despawned.to, LifecycleState::Despawned);
        assert_eq!(lifecycle.revision(), 3);
    }

    #[test]
    fn spawn_and_despawn_reject_invalid_states_without_mutation() {
        let mut absent = LifecycleCell::absent(entity());
        assert_eq!(
            absent.begin_despawn(),
            Err(LifecycleRejection::NotActive {
                state: LifecycleState::Absent,
            })
        );
        assert_eq!(absent.state(), LifecycleState::Absent);
        assert_eq!(absent.revision(), 0);

        absent.spawn().expect("spawn");
        assert_eq!(
            absent.spawn(),
            Err(LifecycleRejection::AlreadySpawned {
                state: LifecycleState::Active,
            })
        );
        assert_eq!(absent.state(), LifecycleState::Active);
        assert_eq!(absent.revision(), 1);

        assert_eq!(
            absent.complete_despawn(),
            Err(LifecycleRejection::NotDespawning {
                state: LifecycleState::Active,
            })
        );
        assert_eq!(absent.state(), LifecycleState::Active);
    }

    #[test]
    fn terminal_state_is_final_and_requires_a_reason() {
        let mut lifecycle = LifecycleCell::absent(entity());
        lifecycle.spawn().expect("spawn");

        assert_eq!(
            lifecycle.mark_terminal(" "),
            Err(LifecycleRejection::EmptyTerminalReason)
        );
        assert_eq!(lifecycle.state(), LifecycleState::Active);

        let terminal = lifecycle
            .mark_terminal("scripted-complete")
            .expect("terminal");
        assert_eq!(terminal.kind, LifecycleTransitionKind::TerminalMarked);
        assert_eq!(terminal.from, LifecycleState::Active);
        assert_eq!(terminal.to, LifecycleState::Terminal);
        assert_eq!(lifecycle.terminal_reason(), Some("scripted-complete"));

        assert_eq!(
            lifecycle.begin_despawn(),
            Err(LifecycleRejection::TerminalIsFinal)
        );
        assert_eq!(lifecycle.spawn(), Err(LifecycleRejection::TerminalIsFinal));
        assert_eq!(lifecycle.state(), LifecycleState::Terminal);
        assert_eq!(lifecycle.revision(), 2);
    }
}
