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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SetOnceDebugShape {
    Empty,
    Published,
}

/// Production `SetOnce::fmt` calls `get` once and passes the resulting
/// `Option<&T>` to the standard debug formatter. This body proves the
/// SetOnce-owned state selection; rendering `T` is the generic fmt boundary.
pub fn debug_shape<T>(source: &PublishedOnce<T>) -> (result: SetOnceDebugShape)
    requires source.well_formed(),
    ensures
        result == match source.contents() {
            MemContents::Init(_) => SetOnceDebugShape::Published,
            MemContents::Uninit => SetOnceDebugShape::Empty,
        },
    no_unwind
{
    match source.get() {
        Some(_) => SetOnceDebugShape::Published,
        None => SetOnceDebugShape::Empty,
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SetOnceErrorDisplay {
    SetOnceError,
}

#[derive(PartialEq, Eq)]
pub struct SetOnceErrorModel<T> {
    payload: T,
}

impl<T> SetOnceErrorModel<T> {
    pub closed spec fn payload(&self) -> T { self.payload }

    pub fn new(payload: T) -> (result: Self)
        ensures result.payload() == payload,
        no_unwind
    {
        SetOnceErrorModel { payload }
    }

    /// `Display` emits a fixed label and never observes or changes `T`.
    pub fn display(&self) -> (result: SetOnceErrorDisplay)
        ensures result == SetOnceErrorDisplay::SetOnceError,
        no_unwind
    {
        SetOnceErrorDisplay::SetOnceError
    }

    /// Derived `Debug` borrows the payload. The actual `T: Debug` formatting
    /// is the standard trait boundary; SetOnceError retains its payload.
    pub fn debug_borrows_payload(&self) -> (result: &T)
        ensures *result == self.payload(),
        no_unwind
    {
        &self.payload
    }

    /// Production's marker `Error` implementation inherits the default
    /// source, which is always absent.
    pub fn error_source_is_none(&self) -> (result: bool)
        ensures result,
        no_unwind
    {
        true
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

    let empty_debug = debug_shape(&empty);
    let initialized_debug = debug_shape(&initialized);
    assert(empty_debug == SetOnceDebugShape::Empty);
    assert(initialized_debug == SetOnceDebugShape::Published);

    let error = SetOnceErrorModel::new(value);
    let display = error.display();
    let debug_payload = error.debug_borrows_payload();
    let no_source = error.error_source_is_none();
    assert(display == SetOnceErrorDisplay::SetOnceError);
    assert(*debug_payload == value);
    assert(no_source);
    assert(error.payload() == value);
}

} // verus!
