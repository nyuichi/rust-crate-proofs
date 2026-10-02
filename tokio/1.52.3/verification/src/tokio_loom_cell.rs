use crate::published_cell::PublishedCell;
use vstd::prelude::*;
use vstd::raw_ptr::MemContents;

verus! {

/// Proof view of production `SetOnceValue<T>`, the operation-specific wrapper
/// around Tokio's loom-compatible `UnsafeCell<MaybeUninit<T>>` value field.
///
/// `PublishedCell<T>` erases to vstd's `PCell` representation, which vstd
/// documents as an `UnsafeCell<MaybeUninit<T>>` at runtime.  The representation
/// correspondence with its transparent physical wrapper is the adapter
/// boundary; the same four operations are body-proved below and exercised
/// directly against production by the focused loom contract test.
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
#[path = "../../src/loom/std/unsafe_cell.rs"]
mod production_unsafe_cell;

#[cfg(test)]
mod layout_tests {
    use super::{production_unsafe_cell, TokioLoomCell};
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
        assert_eq!(
            size_of::<TokioLoomCell<T>>(),
            size_of::<production_unsafe_cell::UnsafeCell<MaybeUninit<T>>>()
        );
        assert_eq!(
            align_of::<TokioLoomCell<T>>(),
            align_of::<production_unsafe_cell::UnsafeCell<MaybeUninit<T>>>()
        );
    }

    #[test]
    fn erased_layout_matches_tokio_value_field() {
        assert_layout::<()>();
        assert_layout::<u64>();
        assert_layout::<[u8; 31]>();
        assert_layout::<Aligned>();

        let cell = production_unsafe_cell::UnsafeCell::new(MaybeUninit::new(11_u64));
        let wrapper_address = (&cell as *const _) as *const ();
        cell.with(|inner| {
            assert_eq!(wrapper_address, inner.cast::<()>());
            // SAFETY: this test constructed the field initialized above.
            assert_eq!(unsafe { (*inner).assume_init_ref() }, &11);
        });
        cell.with_mut(|inner| {
            // SAFETY: the field is initialized and exclusively accessed by
            // this synchronous test callback.
            unsafe { (*inner).as_mut_ptr().write(12) };
        });
        cell.with(|inner| {
            // SAFETY: the preceding callback wrote an initialized value.
            assert_eq!(unsafe { (*inner).assume_init_ref() }, &12);
        });
    }
}
