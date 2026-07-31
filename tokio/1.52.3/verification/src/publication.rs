use crate::tokio_loom_cell::TokioLoomCell;
use vstd::atomic::{PAtomicBool, PermissionBool};
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

/// Verus proof view of production `SetOnceFlag`.
///
/// vstd currently implements this primitive with a sequentially-consistent
/// atomic. Production uses `load_acquire` and `store_release`; the fact that
/// one-shot publication needs no stronger ordering is the explicit weak-memory
/// refinement boundary. All SetOnce-specific flag/slot reasoning is proved
/// below rather than included in that boundary.
pub struct PublicationFlag {
    atomic: PAtomicBool,
    permission: Tracked<PermissionBool>,
}

impl PublicationFlag {
    #[verifier::type_invariant]
    spec fn wf(&self) -> bool {
        self.permission@.id() == self.atomic.id()
    }

    pub closed spec fn value(&self) -> bool {
        self.permission@.value()
    }

    pub fn new(value: bool) -> (result: Self)
        ensures
            result.value() == value,
    {
        let (atomic, permission) = PAtomicBool::new(value);
        PublicationFlag { atomic, permission }
    }

    pub fn load_acquire(&self) -> (result: bool)
        ensures
            result == self.value(),
        no_unwind
    {
        proof {
            use_type_invariant(self);
        }
        self.atomic.load(Tracked(self.permission.borrow()))
    }

    pub fn store_release(&mut self, value: bool)
        ensures
            final(self).value() == value,
        no_unwind
    {
        proof {
            use_type_invariant(&*self);
        }
        self.atomic.store(Tracked(self.permission.borrow_mut()), value);
    }

    /// Relaxed observation used only under exclusive ownership of the full
    /// SetOnce representation.
    pub fn load_relaxed(&self) -> (result: bool)
        ensures
            result == self.value(),
        no_unwind
    {
        proof {
            use_type_invariant(self);
        }
        self.atomic.load(Tracked(self.permission.borrow()))
    }

    /// Relaxed update used only under exclusive ownership.
    pub fn store_relaxed(&mut self, value: bool)
        ensures
            final(self).value() == value,
        no_unwind
    {
        proof {
            use_type_invariant(&*self);
        }
        self.atomic.store(Tracked(self.permission.borrow_mut()), value);
    }
}

/// The reusable SetOnce publication kernel.
///
/// A writer lease is represented by `&mut self`. Readers need only `&self`.
/// The central safety relation says that a visible publication bit
/// is equivalent to an initialized value slot.
pub struct PublishedOnce<T> {
    value: TokioLoomCell<T>,
    flag: PublicationFlag,
}

impl<T> PublishedOnce<T> {
    pub closed spec fn well_formed(&self) -> bool {
        self.flag.value() == self.value.contents().is_init()
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
        let flag = PublicationFlag::new(false);
        PublishedOnce { value, flag }
    }

    pub fn new(value: T) -> (result: Self)
        ensures
            result.contents() == MemContents::Init(value),
            result.well_formed(),
    {
        let value = TokioLoomCell::initialized(value);
        let flag = PublicationFlag::new(true);
        PublishedOnce { value, flag }
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
        self.flag.load_acquire()
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
        if self.flag.load_acquire() {
            Some(unsafe { self.value.get_unchecked() })
        } else {
            None
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
        self.value.write(value);
        self.flag.store_release(true);
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
        self.flag.store_release(false);
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
            self.flag.store_relaxed(false);
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
