use crate::published_cell::PublishedCell;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

/// Proof view of Tokio's loom-compatible
/// `UnsafeCell<MaybeUninit<T>>` value field.
///
/// `PublishedCell<T>` erases to vstd's `PCell` representation, which vstd
/// documents as an `UnsafeCell<MaybeUninit<T>>` at runtime.  The representation
/// correspondence with Tokio's private loom wrapper is the Phase 1 adapter
/// boundary; initialization and reference-producing operations below are
/// body-proved and are not part of that boundary.
#[repr(transparent)]
pub struct TokioLoomCell<T> {
    slot: PublishedCell<T>,
}

impl<T> TokioLoomCell<T> {
    pub closed spec fn contents(&self) -> MemContents<T> {
        self.slot.contents()
    }

    pub fn uninit() -> (result: Self)
        ensures
            result.contents() == MemContents::Uninit,
    {
        TokioLoomCell { slot: PublishedCell::empty() }
    }

    pub fn initialized(value: T) -> (result: Self)
        ensures
            result.contents() == MemContents::Init(value),
    {
        TokioLoomCell { slot: PublishedCell::new(value) }
    }

    /// Proof counterpart of the write performed through Tokio's
    /// `UnsafeCell::with_mut` callback in `SetOnce::set`.
    pub fn write(&mut self, value: T)
        requires
            old(self).contents() == MemContents::Uninit,
        ensures
            final(self).contents() == MemContents::Init(value),
        no_unwind
    {
        self.slot.publish(value);
    }

    /// Production-shaped counterpart of `SetOnce::get_unchecked`.
    ///
    /// The method stays unsafe at the Rust API level and requires the exact
    /// initialization fact that the caller must obtain from publication.
    pub unsafe fn get_unchecked<'a>(&'a self) -> (result: &'a T)
        requires
            self.contents().is_init(),
        ensures
            *result == self.contents().value(),
        no_unwind
    {
        self.slot.get()
    }

    pub fn take(&mut self) -> (result: T)
        requires
            old(self).contents().is_init(),
        ensures
            result == old(self).contents().value(),
            final(self).contents() == MemContents::Uninit,
        no_unwind
    {
        self.slot.take()
    }
}

pub struct NonCopyPayload {
    pub first: u64,
    pub second: u64,
}

pub fn verify_production_get_unchecked_shape(first: u64, second: u64)
{
    let mut cell = TokioLoomCell::uninit();
    cell.write(NonCopyPayload { first, second });
    {
        let observed = unsafe { cell.get_unchecked() };
        assert(observed.first == first);
        assert(observed.second == second);
    }
    let value = cell.take();
    assert(value.first == first);
    assert(value.second == second);
}

} // verus!

#[cfg(test)]
mod layout_tests {
    use super::TokioLoomCell;
    use core::cell::UnsafeCell;
    use core::mem::{align_of, size_of, MaybeUninit};

    #[repr(align(64))]
    struct Aligned([u8; 3]);

    fn assert_layout<T>() {
        assert_eq!(
            size_of::<TokioLoomCell<T>>(),
            size_of::<UnsafeCell<MaybeUninit<T>>>()
        );
        assert_eq!(
            align_of::<TokioLoomCell<T>>(),
            align_of::<UnsafeCell<MaybeUninit<T>>>()
        );
    }

    #[test]
    fn erased_layout_matches_tokio_value_field() {
        assert_layout::<()>();
        assert_layout::<u64>();
        assert_layout::<[u8; 31]>();
        assert_layout::<Aligned>();
    }
}
