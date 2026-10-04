//! Source-correspondence probe for bytes::free_boxed_slice.
//! Caller ownership, vtable dispatch, and automatic Drop remain unconnected.
#![allow(unexpected_cfgs)]
use std::alloc::{dealloc, Layout};
use creusot_std::{prelude::*, ghost::perm::Perm, std::ptr::{PtrLive, SlicePointerExt}};

#[logic(opaque)]
fn layout_size(layout: Layout) -> Int { dead }
#[logic(opaque)]
fn layout_align(layout: Layout) -> Int { dead }

// STD-LAYOUT-01: restricted allocator layout primitive; u8 alignment only.
extern_spec! {
    impl Layout {
        #[requires(align@ == 1)]
        #[requires(0 < size@ && size@ <= isize::MAX@)]
        #[ensures(match result {
            Ok(layout) => layout_size(layout) == size@ && layout_align(layout) == 1,
            Err(_) => false
        })]
        fn from_size_align(size: usize, align: usize) -> Result<Layout, std::alloc::LayoutError>;
    }
}

// STD-PTRDIFF-01: native pointer subtraction with an actual live-allocation witness.
#[trusted]
#[erasure(<*const u8>::offset_from)]
#[requires(live.contains_range(offset, 0))]
#[requires(live.contains_range(base, 0))]
#[ensures(result@ == offset.sub_logic(base))]
unsafe fn offset_from_live(offset: *const u8, base: *const u8, live: Ghost<PtrLive<'_, u8>>) -> isize {
    let _ = live;
    offset.offset_from(base)
}

// STD-DEALLOC-01: consumes the unique allocation permission returned by Perm::from_box.
// No bytes-specific refcount or lifetime theorem is postulated.
#[trusted]
#[erasure(dealloc)]
#[requires(ptr as *const u8 == (*ownership.ward()).thin())]
#[requires(0 < ownership.val()@.len())]
#[requires(layout_size(layout) == ownership.val()@.len())]
#[requires(layout_align(layout) == 1)]
unsafe fn dealloc_bytes(ptr: *mut u8, layout: Layout, ownership: Ghost<Box<Perm<*const [u8]>>>) {
    let _ = ownership;
    dealloc(ptr, layout)
}

/// Exact runtime arithmetic/deallocation path, with permission-aware primitive calls.
#[requires(buf as *const u8 == (*ownership.ward()).thin())]
#[requires(0 < ownership.val()@.len())]
#[requires(len@ <= ownership.val()@.len())]
#[requires(offset == (buf as *const u8).offset_logic(ownership.val()@.len() - len@))]
pub unsafe fn free_boxed_slice(buf: *mut u8, offset: *const u8, len: usize,
    ownership: Ghost<Box<Perm<*const [u8]>>>) {
    let live = ghost! { ownership.live() };
    let cap = offset_from_live(offset, buf, live) as usize + len;
    dealloc_bytes(buf, Layout::from_size_align(cap, 1).unwrap(), ownership)
}

/// A real Box constructor establishes all preconditions of the extracted body.
/// This is not the Bytes constructor or its vtable path.
#[requires(0 < input@.len())]
#[requires(start@ <= input@.len())]
pub fn release_suffix(input: Box<[u8]>, start: usize) {
    use creusot_std::std::ptr::PtrAddExt;
    let full = input.len();
    let (base, ownership) = Perm::from_box(input);
    let live = ghost! { ownership.live() };
    let offset = unsafe { (base as *const u8).add_live(start, live) };
    unsafe { free_boxed_slice(base as *mut u8, offset, full - start, ownership) }
}

#[cfg(feature = "wrong_layout")]
#[requires(0 < input@.len() && input@.len() < isize::MAX@)]
pub fn wrong_layout(input: Box<[u8]>) {
    let full = input.len();
    let (base, ownership) = Perm::from_box(input);
    let layout = Layout::from_size_align(full + 1, 1).unwrap();
    unsafe { dealloc_bytes(base as *mut u8, layout, ownership) }
}

#[cfg(feature = "wrong_capacity")]
#[requires(buf as *const u8 == (*ownership.ward()).thin())]
#[requires(0 < ownership.val()@.len() && ownership.val()@.len() < isize::MAX@)]
#[requires(len@ <= ownership.val()@.len())]
#[requires(offset == (buf as *const u8).offset_logic(ownership.val()@.len() - len@))]
pub unsafe fn wrong_capacity(buf: *mut u8, offset: *const u8, len: usize,
    ownership: Ghost<Box<Perm<*const [u8]>>>) {
    let live = ghost! { ownership.live() };
    let cap = offset_from_live(offset, buf, live) as usize + len + 1;
    dealloc_bytes(buf, Layout::from_size_align(cap, 1).unwrap(), ownership)
}
