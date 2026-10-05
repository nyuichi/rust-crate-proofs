//! Merge adjacent affine packets, retaining the left registration identity.
use super::*;

#[check(ghost)]
#[requires(packet_matches(left.inner_logic(), coordinator.inner_logic().public()))]
#[requires(packet_matches(right.inner_logic(), coordinator.inner_logic().public()))]
#[requires(left.inner_logic().0.logical_id() != right.inner_logic().0.logical_id())]
#[requires(left.inner_logic().1.hi() == right.inner_logic().1.lo())]
#[ensures(packet_matches(result.inner_logic(), (^coordinator.inner_logic()).public()))]
#[ensures(result.inner_logic().0.logical_id() == left.inner_logic().0.logical_id())]
#[ensures(result.inner_logic().1.lo() == left.inner_logic().1.lo())]
#[ensures(result.inner_logic().1.hi() == right.inner_logic().1.hi())]
#[ensures((^coordinator.inner_logic()).public().capacity == coordinator.inner_logic().public().capacity)]
#[ensures((^coordinator.inner_logic()).public().allocation == coordinator.inner_logic().public().allocation)]
#[ensures((^coordinator.inner_logic()).public().registration == coordinator.inner_logic().public().registration)]
#[ensures((^coordinator.inner_logic()).public().next_id == coordinator.inner_logic().public().next_id)]
#[ensures(*(^coordinator.inner_logic()).public().pending ==
    (*coordinator.inner_logic().public().pending).remove(right.inner_logic().0.logical_id())
        .insert(left.inner_logic().0.logical_id(), (left.inner_logic().1.lo(), right.inner_logic().1.hi())))]
#[ensures((* (^coordinator.inner_logic()).public().pending).len() + 1 ==
    (*coordinator.inner_logic().public().pending).len())]
#[ensures(forall<index: Int> result.inner_logic().1.slot(index) ==
    if left.inner_logic().1.lo() <= index && index < left.inner_logic().1.hi() {
        left.inner_logic().1.slot(index)
    } else if right.inner_logic().1.lo() <= index && index < right.inner_logic().1.hi() {
        right.inner_logic().1.slot(index)
    } else { None })]
#[ensures(forall<other: Packet> packet_matches(other, coordinator.inner_logic().public()) &&
    other.0.logical_id() != left.inner_logic().0.logical_id() &&
    other.0.logical_id() != right.inner_logic().0.logical_id() ==>
        packet_matches(other, (^coordinator.inner_logic()).public()))]
pub(crate) fn merge(
    coordinator: Ghost<&mut Coordinator>, left: Ghost<Packet>, right: Ghost<Packet>,
) -> Ghost<Packet> {
    ghost! {
        NonAtomicInvariant::open_mut(coordinator, move |state: Ghost<&mut State>| {
            ghost! {
                let state = state.into_inner();
                let (left_ticket, left_region) = left.into_inner();
                let (right_ticket, right_region) = right.into_inner();
                let left_id = snapshot!(left_ticket.logical_id());
                let right_id = snapshot!(right_ticket.logical_id());
                let bounds = snapshot!((left_region.lo(), right_region.hi()));
                let old_pending = state.status.pending;
                active_partition_at(old_pending, right_id);
                let old_active = snapshot!(active_registrations(*old_pending));
                let remaining = snapshot!(active_registrations((*old_pending).remove(*right_id)));
                let right_value = snapshot!(right_ticket.fragment@);
                let old_returned = snapshot!(state.returned.unwrap_logic()@);
                let authority = snapshot!(state.authority.unwrap_logic()@);
                let returned = state.returned.take().unwrap().join(right_ticket.fragment);
                ledger_transfer(old_returned, right_value, remaining, old_active,
                    snapshot!(returned@), authority);
                let new_pending = snapshot!((*old_pending).remove(*right_id).insert(*left_id, *bounds));
                proof_assert!(active_registrations(*new_pending).ext_eq(*remaining));
                proof_assert!(forall<index: Int> pending_outside(*new_pending, index) == pending_outside(*old_pending, index));
                proof_assert!(forall<id: Int> id != *left_id && id != *right_id ==>
                    match (*old_pending).get(id) {
                        Some((lo, hi)) => lo == hi || bounds.0 == bounds.1 ||
                            hi <= bounds.0 || bounds.1 <= lo,
                        None => true,
                    });
                state.returned = Some(returned);
                state.status.pending = new_pending;
                proof_assert!(state_ledger_valid(*state));
                proof_assert!(state_interval_bounds_valid(*state));
                proof_assert!(state_pending_pool_disjoint(*state));
                proof_assert!(state_pending_regions_disjoint(*state));
                proof_assert!(state_pool_complement_coverage(*state));
                let region = left_region.join(right_region);
                (left_ticket, region)
            }
        }).into_inner()
    }
}
