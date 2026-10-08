//! Closed, fresh, nonescaping normal-return protocol client. No presumed IDs.
use super::*;
struct Marker(u8);
impl RecoveryPayload for Marker {
    type Metadata = u8;
    #[logic] fn metadata(self) -> u8 { self.0 }
    #[logic(prophetic)] fn wellformed(self) -> bool { true }
}

/// A new child is issued after the root and a sibling retire; survivors remain
/// usable. Final success is derived from the complete cursor map, not a quota.
#[ensures(result)]
pub fn closed_sparse_lifecycle() -> bool {
    let (registry, first, mut cursor) = Registry::new(
        ghost! { Marker(42) }, ghost! { LifetimeToken::new() });
    let (_, second) = registry.register(first.borrow(), cursor.borrow_mut());
    let (_, third) = registry.register(first.borrow(), cursor.borrow_mut());
    let (third_last, third_recovery) = registry.retire(third, cursor.borrow_mut());
    proof_assert!(!third_last && third_recovery.inner_logic() == None);
    let (first_last, first_recovery) = registry.retire(first, cursor.borrow_mut());
    proof_assert!(!first_last && first_recovery.inner_logic() == None);
    let (_, fourth) = registry.register(second.borrow(), cursor.borrow_mut());
    let (second_last, second_recovery) = registry.retire(second, cursor.borrow_mut());
    proof_assert!(!second_last && second_recovery.inner_logic() == None);
    let (fourth_last, fourth_recovery) = registry.retire(fourth, cursor.borrow_mut());
    proof_assert!(fourth_last && fourth_recovery.inner_logic() != None);
    ghost! {
        let (payload, full) = fourth_recovery.into_inner().unwrap();
        proof_assert!(payload.metadata() == 42u8);
        proof_assert!(full.frac() == PositiveReal::from_int(1));
        let _dead = full.end();
    };
    fourth_last
}

#[check(ghost)]
fn cannot_extract_perm(cursor: crate::event::ScopeCursor<crate::lifecycle::State<Marker>>) -> Perm<ModelAtomic> {
    *cursor.observation()
}
