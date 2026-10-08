//! UNPROVED, default-off client experiment for quota-free sparse registration
//! and arbitrary retirement order. The attempted multi-clone driver did not
//! pass its final-last proof obligation; it is not positive coverage. See
//! `evidence/attempt-combined-driver-v12-diagnostic-only.tar.gz` for the
//! immutable failed run and its exact null tasks.

use super::*;
use creusot_std::ghost::lifetime_logic::LifetimeToken;

struct Marker(u8);

impl RecoveryPayload for Marker {
    type Metadata = u8;

    #[logic]
    fn metadata(self) -> Self::Metadata {
        self.0
    }

    #[logic(prophetic)]
    fn wellformed(self) -> bool {
        true
    }
}

/// Exclusive sparse-map fragments make live logical IDs distinct without
/// relying on a bounded clone quota or an exact counter observation.
#[check(ghost)]
#[requires(first.valid(*public) && second.valid(*public))]
#[ensures((^first).valid(*public) && second.valid(*public))]
#[ensures((^first).id() != second.id())]
fn tickets_are_distinct(
    first: &mut Ticket<Marker>,
    second: &Ticket<Marker>,
    public: Snapshot<<State<Marker> as Protocol>::Public>,
) {
    first.fragment.valid_op_lemma(&second.fragment);
    proof_assert!(first.fragment@.op(second.fragment@) != None);
    proof_assert!(first.id != second.id);
}

/// UNPROVED experiment: attempt three clones from the same source, remove a
/// middle id, allocate a new ID beyond the hole, retire the payload-bearing
/// source early, and obtain typed payload plus a full LifetimeToken at the
/// final Acquire. The required final-last guarantee remains unproved because
/// the client has no body-proved completeness receipt for all live tickets.
#[ensures(result.0 == 1usize && result.1 == 2usize && result.2 == 3usize && result.3 == 3usize)]
#[ensures(!result.4 && !result.5 && !result.6 && !result.7 && result.8)]
pub fn three_clones_sparse_hole_mixed_retirement() ->
    (usize, usize, usize, usize, bool, bool, bool, bool, bool)
{
    let (registry, first) = Registry::new(
        ghost! { Marker(42) },
        ghost! { LifetimeToken::new() },
    );

    let (old_one, second) = registry.register(first.borrow());
    let (old_two, third) = registry.register(first.borrow());
    let (old_three, fourth) = registry.register(first.borrow());
    proof_assert!(second.inner_logic().id() == 1);
    proof_assert!(third.inner_logic().id() == 2);
    proof_assert!(fourth.inner_logic().id() == 3);

    // Retiring an interior ticket leaves a sparse hole while source id 0 is
    // still live and can be used for a fresh registration.
    let (middle_last, middle_recovery) = registry.retire(third);
    proof_assert!(!middle_last && middle_recovery.inner_logic() == None);
    let (old_four, fifth) = registry.register(first.borrow());
    proof_assert!(fifth.inner_logic().id() == 4);

    // The payload-bearing source retires before the remaining peers. Its
    // AtView remains inside State until the last ticket reaches Acquire.
    let (source_last, source_recovery) = registry.retire(first);
    proof_assert!(!source_last && source_recovery.inner_logic() == None);
    let (fourth_last, fourth_recovery) = registry.retire(fifth);
    proof_assert!(!fourth_last && fourth_recovery.inner_logic() == None);
    let (third_last, third_recovery) = registry.retire(fourth);
    proof_assert!(!third_last && third_recovery.inner_logic() == None);
    let (second_last, second_recovery) = registry.retire(second);
    proof_assert!(second_last && second_recovery.inner_logic() != None);

    ghost! {
        let (payload, full) = second_recovery.into_inner().unwrap();
        proof_assert!(payload.0 == 42u8);
        proof_assert!(payload.metadata() == 42u8);
        proof_assert!(full.frac() == PositiveReal::from_int(1));
        let _dead = full.end();
    };

    (
        old_one,
        old_two,
        old_three,
        old_four,
        middle_last,
        source_last,
        fourth_last,
        third_last,
        second_last,
    )
}
