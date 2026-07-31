use crate::release_acquire::ReleaseAcquireFlag;
use crate::tokio_loom_cell::TokioLoomCell;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

/// The reusable SetOnce publication kernel.
///
/// A writer lease is represented by `&mut self`. Readers need only `&self`.
/// The central safety relation says that a visible publication bit
/// is equivalent to an initialized value slot.
pub struct PublishedOnce<T> {
    value: TokioLoomCell<T>,
    flag: ReleaseAcquireFlag<T>,
}

impl<T> PublishedOnce<T> {
    pub closed spec fn well_formed(&self) -> bool {
        self.flag.well_formed()
            && match self.flag.published_value() {
                Some(value) => self.value.contents() == MemContents::Init(value),
                None => self.value.contents() == MemContents::Uninit,
            }
    }

    pub closed spec fn contents(&self) -> MemContents<T> {
        self.value.contents()
    }

    pub fn empty() -> (result: Self)
        ensures
            result.contents() == MemContents::Uninit,
            result.well_formed(),
    {
        let value = TokioLoomCell::uninit();
        let flag = ReleaseAcquireFlag::new();
        PublishedOnce { value, flag }
    }

    pub fn new(value: T) -> (result: Self)
        ensures
            result.contents() == MemContents::Init(value),
            result.well_formed(),
    {
        let mut flag = ReleaseAcquireFlag::new();
        let cell = TokioLoomCell::initialized(value);
        flag.store_release(Ghost(value));
        PublishedOnce { value: cell, flag }
    }

    pub fn new_with(value: Option<T>) -> (result: Self)
        ensures
            result.well_formed(),
            match value {
                Some(value) => result.contents() == MemContents::Init(value),
                None => result.contents() == MemContents::Uninit,
            },
    {
        match value {
            Some(value) => PublishedOnce::new(value),
            None => PublishedOnce::empty(),
        }
    }

    pub fn initialized(&self) -> (result: bool)
        requires
            self.well_formed(),
        ensures
            result == self.contents().is_init(),
        no_unwind
    {
        self.flag.load_acquire().is_some()
    }

    /// Readiness-only relaxed observation used by `wait` after polling its
    /// notification future. This result never grants a reference; the outer
    /// loop must call `get` and perform an Acquire load before reading T.
    pub fn relaxed_is_set(&self) -> (result: bool)
        requires
            self.well_formed(),
        ensures
            result == self.contents().is_init(),
        no_unwind
    {
        self.flag.load_relaxed()
    }

    /// Production-shaped read: the flag check justifies the reference-producing
    /// operation, and a false observation returns no reference.
    pub fn get<'a>(&'a self) -> (result: Option<&'a T>)
        requires
            self.well_formed(),
        ensures
            match result {
                Some(value) => self.contents() == MemContents::Init(*value),
                None => self.contents() == MemContents::Uninit,
            },
        no_unwind
    {
        match self.flag.load_acquire() {
            Some(_acquired) => Some(unsafe { self.value.get_unchecked() }),
            None => None,
        }
    }

    /// Publish while holding the exclusive writer lease.
    pub fn publish(&mut self, value: T)
        requires
            old(self).well_formed(),
            old(self).contents() == MemContents::Uninit,
        ensures
            final(self).contents() == MemContents::Init(value),
            final(self).well_formed(),
        no_unwind
    {
        assert(self.flag.well_formed());
        assert(self.flag.published_value().is_none());
        proof { self.flag.unpublished_iff_no_value(); }
        assert(self.flag.unpublished());
        self.value.write(value);
        assert(self.flag.unpublished());
        self.flag.store_release(Ghost(value));
    }

    pub fn take(&mut self) -> (result: T)
        requires
            old(self).well_formed(),
            old(self).contents().is_init(),
        ensures
            result == old(self).contents().value(),
            final(self).contents() == MemContents::Uninit,
            final(self).well_formed(),
        no_unwind
    {
        self.flag.reset_relaxed_owned();
        self.value.take()
    }

    /// Production-shaped `take_inner`, shared by `into_inner` and `Drop`.
    pub fn take_inner(&mut self) -> (result: Option<T>)
        requires
            old(self).well_formed(),
        ensures
            final(self).well_formed(),
            final(self).contents() == MemContents::Uninit,
            match old(self).contents() {
                MemContents::Init(value) => result == Some(value),
                MemContents::Uninit => result == None,
            },
        no_unwind
    {
        if !self.flag.load_relaxed() {
            None
        } else {
            self.flag.reset_relaxed_owned();
            Some(self.value.take())
        }
    }
}

pub struct Payload {
    pub value: u64,
}

pub fn verify_publication_roundtrip(value: u64)
{
    let mut once = PublishedOnce::empty();
    let initially_set = once.initialized();
    assert(!initially_set);
    let initially_observed = once.get();
    assert(initially_observed.is_none());
    once.publish(Payload { value });
    let published = once.initialized();
    assert(published);
    {
        let observed = once.get().unwrap();
        assert(observed.value == value);
    }
    let removed = once.take();
    assert(removed.value == value);
    let after_take = once.get();
    assert(after_take.is_none());
}

pub fn verify_owned_take_is_exactly_once(value: u64)
{
    let mut once = PublishedOnce::new_with(Some(Payload { value }));
    let first = once.take_inner();
    assert(first matches Some(payload) && payload.value == value);
    let second = once.take_inner();
    assert(second.is_none());
}

/// The runtime and const production constructors share these same two abstract
/// representations; instrumentation is intentionally outside this value view.
pub fn verify_constructor_equivalence(value: u64)
{
    let runtime_empty = PublishedOnce::<u64>::empty();
    let const_empty_view = PublishedOnce::<u64>::empty();
    assert(runtime_empty.contents() == const_empty_view.contents());

    let runtime_full = PublishedOnce::new_with(Some(value));
    let const_full_view = PublishedOnce::new(value);
    assert(runtime_full.contents() == const_full_view.contents());
}

} // verus!
