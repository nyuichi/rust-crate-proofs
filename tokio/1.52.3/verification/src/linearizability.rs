use crate::publication::PublishedOnce;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

/// Sequential specification used to identify each SetOnce linearization point.
pub open spec fn set_step<T>(
    before: MemContents<T>,
    input: T,
    result: Result<(), T>,
    after: MemContents<T>,
) -> bool {
    match before {
        MemContents::Uninit => result == Ok(()) && after == MemContents::Init(input),
        MemContents::Init(previous) => {
            result == Err(input) && after == MemContents::Init(previous)
        },
    }
}

pub open spec fn get_step<T>(
    state: MemContents<T>,
    result: Option<&T>,
) -> bool {
    match result {
        Some(value) => state == MemContents::Init(*value),
        None => state == MemContents::Uninit,
    }
}

/// Production-shaped public `set`: an optimistic Acquire observation followed
/// by the locked second check. A successful operation linearizes at the
/// Release publication; either rejection linearizes at the check that observed
/// the already-published state.
pub fn set_production_shape<T>(
    target: &mut PublishedOnce<T>,
    value: T,
) -> (result: Result<(), T>)
    requires
        old(target).well_formed(),
    ensures
        final(target).well_formed(),
        set_step(old(target).contents(), value, result, final(target).contents()),
    no_unwind
{
    if target.initialized() {
        Err(value)
    } else {
        // Acquiring `&mut target` is the proof-view writer lease. Keep the
        // production second check explicit even though no interference can
        // occur after that exclusive capability has been obtained here.
        if target.initialized() {
            Err(value)
        } else {
            target.publish(value);
            Ok(())
        }
    }
}

/// `get` linearizes at its Acquire observation. If it observes publication,
/// the same observation justifies the returned reference.
pub fn get_production_shape<'a, T>(
    target: &'a PublishedOnce<T>,
) -> (result: Option<&'a T>)
    requires
        target.well_formed(),
    ensures
        get_step(target.contents(), result),
    no_unwind
{
    target.get()
}

/// Notification is deliberately after publication. Therefore an unwind in
/// the notification phase cannot roll the abstract state back to empty.
pub fn published_before_notification<T>(
    target: &mut PublishedOnce<T>,
    value: T,
) -> (notification_may_run: bool)
    requires
        old(target).well_formed(),
        old(target).contents() == MemContents::Uninit,
    ensures
        notification_may_run,
        final(target).well_formed(),
        final(target).contents() == MemContents::Init(value),
    no_unwind
{
    target.publish(value);
    true
}

pub fn verify_linearization_sequence(first: u64, rejected: u64)
{
    let mut target = PublishedOnce::empty();

    let absent = get_production_shape(&target);
    assert(absent.is_none());

    let first_result = set_production_shape(&mut target, first);
    assert(first_result == Ok(()));

    let observed = get_production_shape(&target).unwrap();
    assert(*observed == first);

    let rejected_result = set_production_shape(&mut target, rejected);
    assert(rejected_result == Err(rejected));

    let still_first = get_production_shape(&target).unwrap();
    assert(*still_first == first);
}

} // verus!
