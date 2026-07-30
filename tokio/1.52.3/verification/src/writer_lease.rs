use crate::publication::PublishedOnce;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

/// Exclusive capability corresponding to the part of Tokio's `NotifyGuard`
/// contract needed by `SetOnce::set`.
///
/// Constructing this value from `&mut PublishedOnce` is fully checked here.
/// The production adapter still has to establish that holding the waiter-list
/// guard grants the same exclusivity.
pub struct WriterLease<'a, T> {
    target: &'a mut PublishedOnce<T>,
}

impl<'a, T> WriterLease<'a, T> {
    pub closed spec fn contents(&self) -> MemContents<T> {
        self.target.contents()
    }

    pub closed spec fn well_formed(&self) -> bool {
        self.target.well_formed()
    }

    pub fn new(target: &'a mut PublishedOnce<T>) -> (result: Self)
        requires
            old(target).well_formed(),
        ensures
            result.well_formed(),
            result.contents() == old(target).contents(),
    {
        WriterLease { target }
    }

    /// The double-check-and-publish core of production `SetOnce::set` after
    /// the waiter-list lock has been acquired.
    pub fn set(&mut self, value: T) -> (result: Result<(), T>)
        requires
            self.well_formed(),
        ensures
            match old(self).contents() {
                MemContents::Uninit => {
                    result == Ok(())
                        && final(self).well_formed()
                        && final(self).contents() == MemContents::Init(value)
                },
                MemContents::Init(previous) => {
                    result == Err(value)
                        && final(self).well_formed()
                        && final(self).contents() == MemContents::Init(previous)
                },
            },
        no_unwind
    {
        if self.target.initialized() {
            Err(value)
        } else {
            self.target.publish(value);
            Ok(())
        }
    }

    pub fn get<'b>(&'b self) -> (result: Option<&'b T>)
        requires
            self.well_formed(),
        ensures
            match result {
                Some(value) => self.contents() == MemContents::Init(*value),
                None => self.contents() == MemContents::Uninit,
            },
        no_unwind
    {
        self.target.get()
    }
}

pub fn verify_first_writer_wins(first: u64, second: u64)
{
    let mut once = PublishedOnce::empty();
    let mut lease = WriterLease::new(&mut once);
    let first_result = lease.set(first);
    assert(first_result == Ok(()));
    let second_result = lease.set(second);
    assert(second_result == Err(second));
    let observed = lease.get().unwrap();
    assert(*observed == first);
}

} // verus!
