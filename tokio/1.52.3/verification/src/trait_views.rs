use crate::publication::PublishedOnce;
use vstd::prelude::*;

verus! {

/// Value-preserving proof counterpart of production `Clone` for a concrete
/// clonable/copyable payload. Generic Clone execution remains a standard
/// library trait boundary; the SetOnce state selection is body-proved here.
pub fn clone_u64(source: &PublishedOnce<u64>) -> (result: PublishedOnce<u64>)
    requires
        source.well_formed(),
    ensures
        result.well_formed(),
        result.contents() == source.contents(),
{
    let copied = match source.get() {
        Some(value) => Some(*value),
        None => None,
    };
    PublishedOnce::new_with(copied)
}

/// Production `PartialEq` compares the optional published references. This
/// proof body establishes the same relation for the exact abstract contents.
pub fn eq_u64(left: &PublishedOnce<u64>, right: &PublishedOnce<u64>) -> (result: bool)
    requires
        left.well_formed(),
        right.well_formed(),
    ensures
        result == (left.contents() == right.contents()),
    no_unwind
{
    match (left.get(), right.get()) {
        (Some(left), Some(right)) => *left == *right,
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => false,
    }
}

pub fn verify_trait_views(value: u64)
{
    let empty = PublishedOnce::<u64>::empty();
    let empty_clone = clone_u64(&empty);
    let empty_equal = eq_u64(&empty, &empty_clone);
    assert(empty_equal);

    let initialized = PublishedOnce::new(value);
    let initialized_clone = clone_u64(&initialized);
    let initialized_equal = eq_u64(&initialized, &initialized_clone);
    assert(initialized_equal);
    let observed = initialized_clone.get().unwrap();
    assert(*observed == value);
}

} // verus!
