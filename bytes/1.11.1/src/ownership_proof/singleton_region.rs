//! Consume the last registration without deallocating its storage.
//! Included as a child of scalable_tickets so all authority remains sealed.
use super::*;

#[check(ghost)]
#[requires(packet_matches(packet.inner_logic(), coordinator.inner_logic().public()))]
#[requires((*coordinator.inner_logic().public().pending).len() == 1)]
#[ensures(result.inner_logic().0.invariant() && result.inner_logic().1.invariant())]
#[ensures(result.inner_logic().0.capacity() == coordinator.inner_logic().public().capacity)]
#[ensures(result.inner_logic().1.capacity() == coordinator.inner_logic().public().capacity)]
#[ensures(result.inner_logic().0.namespace() == coordinator.inner_logic().public().allocation)]
#[ensures(result.inner_logic().1.namespace() == coordinator.inner_logic().public().allocation)]
#[ensures(result.inner_logic().1.resource_id() == coordinator.inner_logic().public().allocation)]
#[ensures(result.inner_logic().1.lo() == 0 && result.inner_logic().1.hi() == coordinator.inner_logic().public().capacity)]
#[ensures(forall<index: Int> packet.inner_logic().1.lo() <= index && index < packet.inner_logic().1.hi() ==>
    result.inner_logic().1.slot(index) == packet.inner_logic().1.slot(index))]
pub(crate) fn recover(
    coordinator: Ghost<Coordinator>,
    packet: Ghost<Packet>,
) -> Ghost<(Recovery, PhysicalRegion)> {
    ghost! {
        let state = coordinator.into_inner().into_inner();
        let (ticket, region) = packet.into_inner();
        let pending = state.status.pending;
        let ticket_id = snapshot!(ticket.logical_id());
        proof_assert!((*pending).contains(*ticket_id));
        proof_assert!((*pending).remove(*ticket_id).len() == 0);
        pending_cardinality(snapshot!((*pending).remove(*ticket_id)));
        proof_assert!(forall<id: Int> (*pending).contains(id) ==> id == *ticket_id);
        proof_assert!(forall<index: Int> pending_outside(*pending, index) ==
            !(region.lo() <= index && index < region.hi()));
        let recovery = state.recovery.unwrap();
        let pool = state.pool.unwrap();
        proof_assert!(forall<index: Int> pool.contains(index) ==
            (0 <= index && index < state.status.capacity &&
                !(region.lo() <= index && index < region.hi())));
        let full = pool.retire(region).finish(0int);
        let _ = (ticket, state.authority, state.returned);
        (recovery, full)
    }
}
