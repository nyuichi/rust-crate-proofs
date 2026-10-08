//! Body-checked finite accounting for dynamically numbered lifetime tickets.
//!
//! A map key is a monotonically allocated nonnegative ticket id. `next` is the
//! exclusive upper bound on ids allocated so far. The recurrences inspect the
//! canonical range `[0, next)`; they do not trust an aggregate map sum/count.

use creusot_std::{
    logic::{FMap, Int, real::{PositiveReal, Real},
        ra::{RA, UnitRA, auth::CancelLocalUpdateUnit, excl::Excl, update::LocalUpdate}},
    prelude::*,
};

pub type LiveFractions = FMap<Int, Excl<PositiveReal>>;

/// The fraction represented by one id, or zero when that id is absent.
#[logic(open, inline)]
pub fn ticket_fraction(live: LiveFractions, id: Int) -> Real {
    match live.get(id) {
        None => Real::from_int(0),
        Some(Excl(fraction)) => fraction.to_real(),
    }
}

/// One when `id` is live and zero otherwise.
#[logic(open, inline)]
pub fn ticket_cardinality(live: LiveFractions, id: Int) -> Int {
    if live.get(id) == None { 0 } else { 1 }
}

/// Sum of the fractions in ids `[0, next)`.
#[logic(open)]
#[requires(next >= 0)]
#[variant(next)]
pub fn sum_prefix(live: LiveFractions, next: Int) -> Real {
    if next == 0 {
        Real::from_int(0)
    } else {
        ticket_fraction(live, next - 1) + sum_prefix(live, next - 1)
    }
}

/// Number of live tickets in ids `[0, next)`.
#[logic(open)]
#[requires(next >= 0)]
#[variant(next)]
pub fn cardinality_prefix(live: LiveFractions, next: Int) -> Int {
    if next == 0 {
        0
    } else {
        ticket_cardinality(live, next - 1) + cardinality_prefix(live, next - 1)
    }
}

/// Every live id lies in the allocated prefix `[0, next)`.
#[logic(open)]
pub fn ids_bounded(live: LiveFractions, next: Int) -> bool {
    pearlite! {
        next >= 0 &&
        forall<id: Int> live.get(id) != None ==> 0 <= id && id < next
    }
}

/// Insertion at or above the prefix does not change its fraction sum.
#[logic]
#[requires(next >= 0 && next <= id)]
#[ensures(sum_prefix(live.insert(id, Excl(fraction)), next) == sum_prefix(live, next))]
#[variant(next)]
pub fn insertion_outside_sum(live: LiveFractions, id: Int, fraction: PositiveReal, next: Int) {
    if next > 0 {
        let last = next - 1;
        proof_assert!(live.insert(id, Excl(fraction)).get(last) == live.get(last));
        insertion_outside_sum(live, id, fraction, last);
    }
}

/// Insertion at or above the prefix does not change its cardinality.
#[logic]
#[requires(next >= 0 && next <= id)]
#[ensures(cardinality_prefix(live.insert(id, Excl(fraction)), next) == cardinality_prefix(live, next))]
#[variant(next)]
pub fn insertion_outside_cardinality(
    live: LiveFractions,
    id: Int,
    fraction: PositiveReal,
    next: Int,
) {
    if next > 0 {
        let last = next - 1;
        proof_assert!(live.insert(id, Excl(fraction)).get(last) == live.get(last));
        insertion_outside_cardinality(live, id, fraction, last);
    }
}

/// A fresh id at `next` adds exactly its fraction and one live ticket.
#[logic]
#[requires(next >= 0 && live.get(next) == None)]
#[ensures(sum_prefix(live.insert(next, Excl(fraction)), next + 1) ==
    fraction.to_real() + sum_prefix(live, next))]
#[ensures(cardinality_prefix(live.insert(next, Excl(fraction)), next + 1) ==
    1 + cardinality_prefix(live, next))]
pub fn insert_fresh(live: LiveFractions, next: Int, fraction: PositiveReal) {
    insertion_outside_sum(live, next, fraction, next);
    insertion_outside_cardinality(live, next, fraction, next);
}

/// Inserting the next id preserves the bounded-id invariant.
#[logic]
#[requires(ids_bounded(live, next) && live.get(next) == None)]
#[ensures(ids_bounded(live.insert(next, Excl(fraction)), next + 1))]
pub fn insert_fresh_preserves_bound(live: LiveFractions, next: Int, fraction: PositiveReal) {
    proof_assert!(forall<id: Int>
        live.insert(next, Excl(fraction)).get(id) != None ==>
            0 <= id && id < next + 1);
}

/// Removing a ticket outside `[0, next)` leaves its fraction sum unchanged.
#[logic]
#[requires(next >= 0 && next <= id)]
#[ensures(sum_prefix(live.remove(id), next) == sum_prefix(live, next))]
#[variant(next)]
pub fn removal_outside_sum(live: LiveFractions, id: Int, next: Int) {
    if next > 0 {
        let last = next - 1;
        proof_assert!(live.remove(id).get(last) == live.get(last));
        removal_outside_sum(live, id, last);
    }
}

/// Removing a ticket outside `[0, next)` leaves its cardinality unchanged.
#[logic]
#[requires(next >= 0 && next <= id)]
#[ensures(cardinality_prefix(live.remove(id), next) == cardinality_prefix(live, next))]
#[variant(next)]
pub fn removal_outside_cardinality(live: LiveFractions, id: Int, next: Int) {
    if next > 0 {
        let last = next - 1;
        proof_assert!(live.remove(id).get(last) == live.get(last));
        removal_outside_cardinality(live, id, last);
    }
}

/// Removing one known ticket subtracts exactly its fraction and one id.
#[logic]
#[requires(0 <= id && id < next)]
#[requires(live.get(id) == Some(Excl(fraction)))]
#[ensures(sum_prefix(live, next) == sum_prefix(live.remove(id), next) + fraction.to_real())]
#[ensures(cardinality_prefix(live, next) == cardinality_prefix(live.remove(id), next) + 1)]
#[variant(next)]
pub fn remove_known(live: LiveFractions, id: Int, fraction: PositiveReal, next: Int) {
    if next > id + 1 {
        let last = next - 1;
        proof_assert!(live.remove(id).get(last) == live.get(last));
        remove_known(live, id, fraction, last);
    } else {
        proof_assert!(next == id + 1);
        removal_outside_sum(live, id, id);
        removal_outside_cardinality(live, id, id);
    }
}

/// The prefix sums and counts of the empty finite map are zero.
#[logic]
#[requires(next >= 0)]
#[ensures(sum_prefix(LiveFractions::empty(), next) == Real::from_int(0))]
#[ensures(cardinality_prefix(LiveFractions::empty(), next) == 0)]
#[variant(next)]
pub fn empty_prefix(next: Int) {
    if next > 0 {
        let last = next - 1;
        empty_prefix(last);
    }
}

/// Removing the last id from a bounded map leaves all remaining ids below it.
#[logic]
#[requires(next > 0 && ids_bounded(live, next))]
#[ensures(ids_bounded(live.remove(next - 1), next - 1))]
pub fn remove_last_preserves_prefix_bound(live: LiveFractions, next: Int) {
    let last = next - 1;
    proof_assert!(forall<key: Int>
        live.remove(last).get(key) != None ==>
            live.get(key) != None && key != last);
    proof_assert!(forall<key: Int>
        live.remove(last).get(key) != None ==>
            0 <= key && key < last);
}

/// On maps whose keys lie in the counted prefix, the recursive count agrees
/// with the stock finite-map length.
#[logic]
#[requires(ids_bounded(live, next))]
#[ensures(cardinality_prefix(live, next) == live.len())]
#[variant(next)]
pub fn cardinality_prefix_is_len(live: LiveFractions, next: Int) {
    if next == 0 {
        proof_assert!(forall<key: Int> live.get(key) == None);
        proof_assert!(live.ext_eq(LiveFractions::empty()));
        proof_assert!(live.len() == 0);
    } else {
        let last = next - 1;
        let remaining = live.remove(last);
        remove_last_preserves_prefix_bound(live, next);
        cardinality_prefix_is_len(remaining, last);
        removal_outside_cardinality(live, last, last);
        proof_assert!(cardinality_prefix(live, next) ==
            ticket_cardinality(live, last) + cardinality_prefix(live, last));
        proof_assert!(cardinality_prefix(live, last) == cardinality_prefix(remaining, last));
        if live.get(last) == None {
            proof_assert!(ticket_cardinality(live, last) == 0);
            proof_assert!(remaining.len() == live.len());
            proof_assert!(cardinality_prefix(live, next) == live.len());
        } else {
            proof_assert!(ticket_cardinality(live, last) == 1);
            proof_assert!(remaining.len() == live.len() - 1);
            proof_assert!(cardinality_prefix(live, next) == live.len());
        }
    }
}

/// Apply the stock whole-map cancellation update to a live-ticket authority
/// and a fragment. The fragment is consumed into the map's identity remainder.
#[logic(open)]
#[requires(CancelLocalUpdateUnit.premise(authority, fragment))]
#[ensures(result.0 == authority.factor(fragment))]
#[ensures(result.1 == LiveFractions::unit())]
pub fn cancel_fragment_update(
    authority: LiveFractions,
    fragment: LiveFractions,
) -> (LiveFractions, LiveFractions) {
    <CancelLocalUpdateUnit as LocalUpdate<LiveFractions>>::update(
        CancelLocalUpdateUnit,
        authority,
        fragment,
    )
}

/// The factor left after cancelling a singleton ticket is the old live map
/// with precisely that id removed.
#[logic]
#[requires(authority.get(id) == Some(Excl(fraction)))]
#[ensures(authority.factor(LiveFractions::singleton(id, Excl(fraction))) == authority.remove(id))]
pub fn singleton_ticket_factor(
    authority: LiveFractions,
    id: Int,
    fraction: PositiveReal,
) {
    let ticket = LiveFractions::singleton(id, Excl(fraction));
    let remaining = authority.remove(id);
    proof_assert!(ticket.incl(authority));
    // Explain the finite-map composition pointwise. At the removed key the
    // ticket supplies the exclusive value; everywhere else the ticket is
    // empty and `remove` preserves the authority's value.
    proof_assert!(forall<key: Int>
        ticket.get(key).op(remaining.get(key)) == Some(authority.get(key)));
    proof_assert!(forall<key: Int>
        ticket.get(key).op(remaining.get(key)) != None);
    let combined = ticket.total_op(remaining);
    proof_assert!(forall<key: Int>
        Some(combined.get(key)) == ticket.get(key).op(remaining.get(key)));
    proof_assert!(combined.ext_eq(authority));
    proof_assert!(ticket.op(remaining) == Some(combined));
    proof_assert!(ticket.op(remaining) == Some(authority));
    proof_assert!(ticket.cancelable());
    proof_assert!(ticket.op(authority.factor(ticket)) == Some(authority));
    proof_assert!(authority.factor(ticket) == remaining);
}

/// Whole-map cancellation of one ticket has the same exact sum/count effect as
/// removing that ticket from the map.
#[logic]
#[requires(authority.get(id) == Some(Excl(fraction)))]
#[requires(CancelLocalUpdateUnit.premise(authority, LiveFractions::singleton(id, Excl(fraction))))]
#[ensures(sum_prefix(cancel_fragment_update(authority,
    LiveFractions::singleton(id, Excl(fraction))).0, next) + fraction.to_real() ==
    sum_prefix(authority, next))]
#[ensures(cardinality_prefix(authority, next) ==
    cardinality_prefix(cancel_fragment_update(authority,
        LiveFractions::singleton(id, Excl(fraction))).0, next) + 1)]
#[requires(0 <= id && id < next)]
pub fn cancel_singleton_effect(
    authority: LiveFractions,
    id: Int,
    fraction: PositiveReal,
    next: Int,
) {
    singleton_ticket_factor(authority, id, fraction);
    remove_known(authority, id, fraction, next);
}
