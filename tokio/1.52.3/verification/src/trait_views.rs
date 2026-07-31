use crate::publication::PublishedOnce;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

/// Generic lifting of a value clone through SetOnce. The caller supplies the
/// result of `T::clone`; its value-preservation contract is the standard trait
/// boundary, while all SetOnce state selection remains body-proved here.
pub fn clone_from_value<T>(
    source: &PublishedOnce<T>,
    cloned: Option<T>,
) -> (result: PublishedOnce<T>)
    requires
        source.well_formed(),
        match source.contents() {
            MemContents::Init(value) => cloned == Some(value),
            MemContents::Uninit => cloned == None,
        },
    ensures
        result.well_formed(),
        result.contents() == source.contents(),
{
    PublishedOnce::new_with(cloned)
}

/// Generic lifting of a correctly-specified `T::eq` result through SetOnce's
/// optional-state comparison. This separates the generic standard trait call
/// from the body-proved SetOnce state logic.
pub fn eq_from_value<T>(
    left: &PublishedOnce<T>,
    right: &PublishedOnce<T>,
    initialized_values_equal: bool,
) -> (result: bool)
    requires
        left.well_formed(),
        right.well_formed(),
        match (left.contents(), right.contents()) {
            (MemContents::Init(left), MemContents::Init(right)) => {
                initialized_values_equal == (left == right)
            },
            _ => true,
        },
    ensures
        result == (left.contents() == right.contents()),
    no_unwind
{
    match (left.get(), right.get()) {
        (Some(_), Some(_)) => initialized_values_equal,
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => false,
    }
}

pub fn verify_trait_views(value: u64)
{
    let empty = PublishedOnce::<u64>::empty();
    let empty_clone = clone_from_value(&empty, None);
    let empty_equal = eq_from_value(&empty, &empty_clone, false);
    assert(empty_equal);

    let initialized = PublishedOnce::new(value);
    let initialized_clone = clone_from_value(&initialized, Some(value));
    let initialized_equal = eq_from_value(&initialized, &initialized_clone, true);
    assert(initialized_equal);
    let observed = initialized_clone.get().unwrap();
    assert(*observed == value);
}

} // verus!
