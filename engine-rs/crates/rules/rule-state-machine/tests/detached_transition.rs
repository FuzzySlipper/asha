use core_ids::{EntityId, ModeId, ProcessId};
use rule_state_machine::{
    apply_transition_to_instance, MachineInstance, StateMachineSpec, TransitionRequest,
};

#[test]
fn detached_transition_preserves_store_authority_checks() {
    let entity = EntityId::new(7);
    let machine = ProcessId::new(8);
    let idle = ModeId::new(1);
    let run = ModeId::new(2);
    let spec = StateMachineSpec::new(machine, [idle, run]).allow(idle, run);
    let instance = MachineInstance {
        entity,
        machine,
        current: idle,
        revision: 4,
    };

    let applied = apply_transition_to_instance(
        &spec,
        instance,
        TransitionRequest::new(entity, machine, idle, run).expecting_revision(4),
    )
    .expect("detached transition");

    assert_eq!(applied.instance.current, run);
    assert_eq!(applied.instance.revision, 5);
    assert_eq!(applied.previous, idle);
}
