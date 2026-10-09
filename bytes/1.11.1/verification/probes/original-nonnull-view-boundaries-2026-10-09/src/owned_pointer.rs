//! Generic actual-core AtomicPtr / owned-Perm event correspondence.
//!
//! The trusted operations are counterparts of private Std 0.13's
//! `std/sync/atomic.rs` load, strong compare_exchange and into_inner contracts. They use
//! the SAME pointer_model as pointer_event::new_pointer, and preserve native
//! Acquire / (AcqRel, Acquire) orderings. No SC permission extraction, pointer
//! dereference, allocation transfer, successful-CAS or Bytes law is assumed.
//! Callbacks capture actual affine permissions and use Std Committer operations.
//! Exact core-field/model identity and erased callback invocation remain TCB;
//! replacement is a supported core AtomicPtr adapter with these contracts.
//!
//! load_known / exchange_known / first_strong_cas below are body proved. Their
//! all-history-equals premise is temporary: exchange_known exports the actual
//! old/new history and does NOT recreate an immutable pointer binding.
use core::sync::atomic::{AtomicPtr as CoreAtomicPtr, Ordering};
use creusot_std::{
    ghost::{FnGhost, Perm},
    logic::FMap,
    prelude::*,
    std::sync::{
        atomic::{AtomicPtr as ModelAtomicPtr, ordering::{Acquire, Release, None as NoStore}},
        committer::Committer,
        view::{HasTimestamp, SyncView},
    },
};
use crate::pointer_event::pointer_model;

/// One actual Acquire load and its operation-bound ghost callback.
#[trusted]
#[requires(forall<c: &Committer<ModelAtomicPtr<()>, *mut (), Acquire, NoStore>>
    !c.shot_store() && c.ward() == pointer_model(field) ==>
    f.precondition((c,)))]
#[ensures(exists<c: &Committer<ModelAtomicPtr<()>, *mut (), Acquire, NoStore>>
    !c.shot_store() && c.ward() == pointer_model(field) &&
    result == c.val_load() && f.postcondition_once((c,), ()))]
pub(crate) fn load_acquire<F>(field: &CoreAtomicPtr<()>, f: Ghost<F>) -> *mut ()
where F: FnGhost + FnOnce(&Committer<ModelAtomicPtr<()>, *mut (), Acquire, NoStore>) {
    field.load(Ordering::Acquire)
}

/// Native STRONG CAS. Unlike weak CAS, failure entails unequal deep models.
/// The success callback must shoot exactly one store; failure cannot store.
#[trusted]
#[requires(forall<c: &mut Committer<ModelAtomicPtr<()>, *mut (), Acquire, Release>>
    !c.shot_store() && c.ward() == pointer_model(field) &&
    c.val_load().deep_model() == expected.deep_model() && c.val_store() == new ==>
    f.precondition((Ok(c),)) &&
    (f.postcondition_once((Ok(c),), ()) ==> (^c).shot_store()))]
#[requires(forall<c: &Committer<ModelAtomicPtr<()>, *mut (), Acquire, NoStore>>
    !c.shot_store() && c.ward() == pointer_model(field) &&
    c.val_load().deep_model() != expected.deep_model() ==>
    f.precondition((Err(c),)))]
#[ensures(match result {
    Ok(old) => exists<c: &mut Committer<ModelAtomicPtr<()>, *mut (), Acquire, Release>>
        !c.shot_store() && c.ward() == pointer_model(field) &&
        c.val_load().deep_model() == expected.deep_model() && c.val_store() == new &&
        old == c.val_load() && f.postcondition_once((Ok(c),), ()),
    Err(old) => exists<c: &Committer<ModelAtomicPtr<()>, *mut (), Acquire, NoStore>>
        !c.shot_store() && c.ward() == pointer_model(field) &&
        c.val_load().deep_model() != expected.deep_model() &&
        old == c.val_load() && f.postcondition_once((Err(c),), ())
})]
pub(crate) fn compare_exchange<F>(
    field: &CoreAtomicPtr<()>, expected: *mut (), new: *mut (), f: Ghost<F>,
) -> Result<*mut (), *mut ()>
where F: FnGhost + FnOnce(Result<
    &mut Committer<ModelAtomicPtr<()>, *mut (), Acquire, Release>,
    &Committer<ModelAtomicPtr<()>, *mut (), Acquire, NoStore>,
>) {
    field.compare_exchange(expected, new, Ordering::AcqRel, Ordering::Acquire)
}

/// Body-proved load from a still-unmodified owned history. The owner is borrowed
/// and remains available for the later CAS; native allocation may occur between.
#[requires(*own.ward() == pointer_model(field))]
#[requires(forall<t: Int> own.val().get(t) != None ==>
    own.val().get(t).unwrap_logic().0 == expected)]
#[ensures(result == expected)]
#[ensures(**current <= ^current)]
pub(crate) fn load_known(
    field: &CoreAtomicPtr<()>, expected: *mut (), own: Ghost<&Perm<ModelAtomicPtr<()>>>,
    mut current: Ghost<&mut SyncView>,
) -> *mut () {
    load_acquire(field, ghost! {
        |c: &Committer<ModelAtomicPtr<()>, *mut (), Acquire, NoStore>| {
            c.shoot_load(&**own, &mut **current);
        }
    })
}

/// Body-proved first strong CAS: the full permission excludes an untracked
/// competing write. Success follows from the captured history and the strong
/// failure inequality, not from a trusted successful-promotion assertion.
#[requires(*own.ward() == pointer_model(field))]
#[requires(forall<t: Int> own.val().get(t) != None ==>
    own.val().get(t).unwrap_logic().0 == expected)]
#[ensures(result == Ok(expected))]
#[ensures((^own).ward() == own.ward())]
#[ensures(**current <= ^current)]
#[ensures(exists<t: Int>
    own.val().get(t) != None && own.val().get(t + 1) == None &&
    pointer_model(field).get_timestamp(^current) > t &&
    (^own).val() == own.val().insert(t + 1, (new, ^current)))]
pub(crate) fn exchange_known(
    field: &CoreAtomicPtr<()>, expected: *mut (), new: *mut (),
    mut own: Ghost<&mut Perm<ModelAtomicPtr<()>>>, mut current: Ghost<&mut SyncView>,
) -> Result<*mut (), *mut ()> {
    compare_exchange(field, expected, new, ghost! {
        |event: Result<
            &mut Committer<ModelAtomicPtr<()>, *mut (), Acquire, Release>,
            &Committer<ModelAtomicPtr<()>, *mut (), Acquire, NoStore>,
        >| {
            match event {
                Ok(c) => {
                    c.shoot_load(&**own, &mut **current);
                    c.shoot_store(&mut **own, &mut **current);
                }
                Err(c) => {
                    c.shoot_load(&**own, &mut **current);
                }
            }
        }
    })
}

/// Pure predicates on an existing history; neither creates nor returns an owner.
#[logic(open)]
pub(crate) fn latest_is(history: FMap<Int, (*mut (), SyncView)>, expected: *mut ()) -> bool {
    pearlite! { exists<t: Int> history.get(t) != None &&
        history.get(t).unwrap_logic().0 == expected &&
        forall<u: Int> history.get(u) != None ==> u <= t }
}

#[logic(open)]
pub(crate) fn visible_is(history: FMap<Int, (*mut (), SyncView)>, lower: Int, expected: *mut ()) -> bool {
    pearlite! { forall<t: Int> lower <= t && history.get(t) != None ==>
        history.get(t).unwrap_logic().0 == expected }
}

/// The selected first store starts from an actual singleton, so the appended
/// entry is both maximal and the only value visible after this thread's CAS.
/// Unlike exchange_known, this stronger interface does not cover histories
/// with arbitrary gaps or later entries.
#[requires(*own.ward() == pointer_model(field))]
#[requires(exists<t: Int, published: SyncView>
    own.val() == FMap::singleton(t, (expected, published)))]
#[ensures(result == Ok(expected))]
#[ensures((^own).ward() == own.ward())]
#[ensures(**current <= ^current)]
#[ensures(latest_is((^own).val(), new))]
#[ensures(visible_is((^own).val(), pointer_model(field).get_timestamp(^current), new))]
#[ensures(exists<t: Int>
    own.val().get(t) != None && own.val().get(t + 1) == None &&
    pointer_model(field).get_timestamp(^current) > t &&
    (^own).val() == own.val().insert(t + 1, (new, ^current)))]
pub(crate) fn exchange_singleton(
    field: &CoreAtomicPtr<()>, expected: *mut (), new: *mut (),
    own: Ghost<&mut Perm<ModelAtomicPtr<()>>>, current: Ghost<&mut SyncView>,
) -> Result<*mut (), *mut ()> {
    exchange_known(field, expected, new, own, current)
}

/// Body-proved same-thread Acquire read after publication. Historical raw
/// values remain in the history: only the current-view lower bound excludes
/// reading them. This helper must correspond to a real native Acquire site.
#[requires(*own.ward() == pointer_model(field))]
#[requires(visible_is(own.val(), pointer_model(field).get_timestamp(**current), expected))]
#[ensures(result == expected)]
#[ensures(**current <= ^current)]
#[ensures(visible_is(own.val(), pointer_model(field).get_timestamp(^current), expected))]
pub(crate) fn load_visible_known(
    field: &CoreAtomicPtr<()>, expected: *mut (), own: Ghost<&Perm<ModelAtomicPtr<()>>>,
    mut current: Ghost<&mut SyncView>,
) -> *mut () {
    load_acquire(field, ghost! {
        |c: &Committer<ModelAtomicPtr<()>, *mut (), Acquire, NoStore>| {
            c.shoot_load(&**own, &mut **current);
        }
    })
}

/// Generic terminal exclusive read. Std AtomicPtr::into_inner consumes the
/// atomic and its complete Perm and returns the maximal history value/view.
/// Here the native caller instead has &mut to the exact core field, performs
/// *get_mut(), and consumes its complete Perm. No new permission/binding is
/// returned and the native field is unchanged. Interpretation of exclusive
/// access as the latest history entry is the same explicit generic TCB.
/// We retain ONLY its value/maximal-timestamp consequences. A pure timestamp
/// snapshot carries no SyncView or synchronization capability, and this does
/// not invent a native Acquire operation.
#[trusted]
#[requires(*own.ward() == pointer_model(field))]
#[ensures(match own.val().get(*result.1) {
    Some((value, _)) => result.0 == value, None => false
})]
#[ensures(forall<t: Int> match own.val().get(t) {
    Some(_) => t <= *result.1,
    None => true
})]
#[ensures(^field == *field)]
pub(crate) fn get_mut_finish(
    field: &mut CoreAtomicPtr<()>, own: Ghost<Perm<ModelAtomicPtr<()>>>,
) -> (*mut (), Snapshot<Int>) {
    #[cfg(creusot)]
    { unreachable!("generic terminal owned-history interpretation") }
    #[cfg(not(creusot))]
    // The timestamp is entirely erased; no runtime value is used as evidence.
    { (*field.get_mut(), snapshot!(0)) }
}

/// Body-proved latest-value consequence, retaining no phantom readonly owner.
#[requires(*own.ward() == pointer_model(field))]
#[requires(latest_is(own.val(), expected))]
#[ensures(result == expected)]
#[ensures(^field == *field)]
pub(crate) fn finish_latest(
    field: &mut CoreAtomicPtr<()>, expected: *mut (), own: Ghost<Perm<ModelAtomicPtr<()>>>,
) -> *mut () {
    let (value, _terminal_timestamp) = get_mut_finish(field, own);
    value
}

/// Closed generic witness: fresh singleton history, real Acquire, strong CAS.
/// Arbitrary pointer values are only copied, compared and stored, never used
/// as allocation authority or dereferenced.
#[ensures(result)]
pub fn first_strong_cas(initial: *mut (), new: *mut ()) -> bool {
    let mut current = ghost! { SyncView::new().into_inner() };
    let (field, mut own) = crate::pointer_event::new_pointer(initial, current.borrow_mut());
    let observed = load_known(&field, initial, own.borrow(), current.borrow_mut());
    match exchange_known(&field, observed, new, own.borrow_mut(), current.borrow_mut()) {
        Ok(old) => old == initial,
        Err(_) => false,
    }
}

/// Distinguishing post-store witness: both entries coexist, but a same-thread
/// Acquire and a terminal exclusive read each return the actual new value.
#[ensures(result)]
pub fn first_cas_read_finish(initial: *mut (), new: *mut ()) -> bool {
    let mut current = ghost! { SyncView::new().into_inner() };
    let (mut field, mut own) = crate::pointer_event::new_pointer(initial, current.borrow_mut());
    let observed = load_known(&field, initial, own.borrow(), current.borrow_mut());
    let exchanged = exchange_singleton(&field, observed, new, own.borrow_mut(), current.borrow_mut());
    let visible = load_visible_known(&field, new, own.borrow(), current.borrow_mut());
    let latest = finish_latest(&mut field, new, own);
    exchanged == Ok(initial) && visible == new && latest == new
}
