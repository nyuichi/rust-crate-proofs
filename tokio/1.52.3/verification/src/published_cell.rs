use vstd::cell::pcell_maybe_uninit as un;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

/// A value slot whose physical contents and Verus permission stay together.
///
/// Publication ordering is deliberately a separate concern: this type proves
/// that initialization, shared borrowing, and removal preserve the exact value.
pub struct PublishedCell<T> {
    cell: un::PCell<T>,
    permission: Tracked<un::PointsTo<T>>,
}

impl<T> PublishedCell<T> {
    #[verifier::type_invariant]
    spec fn wf(&self) -> bool {
        self.permission@.id() == self.cell.id()
    }

    pub closed spec fn contents(&self) -> MemContents<T> {
        self.permission@.mem_contents()
    }

    pub fn empty() -> (result: Self)
        ensures
            result.contents() == MemContents::Uninit,
    {
        let (cell, permission) = un::PCell::empty();
        PublishedCell { cell, permission }
    }

    pub fn new(value: T) -> (result: Self)
        ensures
            result.contents() == MemContents::Init(value),
    {
        let (cell, permission) = un::PCell::new(value);
        PublishedCell { cell, permission }
    }

    pub fn publish(&mut self, value: T)
        requires
            old(self).contents() == MemContents::Uninit,
        ensures
            final(self).contents() == MemContents::Init(value),
        no_unwind
    {
        proof {
            use_type_invariant(&*self);
        }
        self.cell.put(Tracked(self.permission.borrow_mut()), value);
    }

    pub fn get<'a>(&'a self) -> (result: &'a T)
        requires
            self.contents().is_init(),
        ensures
            *result == self.contents().value(),
        no_unwind
    {
        proof {
            use_type_invariant(self);
        }
        self.cell.borrow(Tracked(self.permission.borrow()))
    }

    pub fn take(&mut self) -> (result: T)
        requires
            old(self).contents().is_init(),
        ensures
            result == old(self).contents().value(),
            final(self).contents() == MemContents::Uninit,
        no_unwind
    {
        proof {
            use_type_invariant(&*self);
        }
        self.cell.take(Tracked(self.permission.borrow_mut()))
    }
}

pub struct Payload {
    pub value: u64,
}

/// Representative non-`Copy` caller: the reference returned by `get` is tied
/// to the `PublishedCell`, while `take` is available again after that borrow ends.
pub fn verify_reference_roundtrip(value: u64)
{
    let mut cell = PublishedCell::empty();
    cell.publish(Payload { value });
    {
        let observed = cell.get();
        assert(observed.value == value);
    }
    let removed = cell.take();
    assert(removed.value == value);
}

} // verus!
