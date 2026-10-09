//! Generic physical effect TCB: exact consumed typed ownership, not Bytes laws.
use alloc::{alloc::{dealloc, Layout}, boxed::Box};
use core::marker::PhantomData;
use creusot_std::{prelude::*, ghost::Perm};

/// Opaque affine effect witness, with construction confined to this module's
/// generic free boundary. It gives no capability to reuse/recover the owner.
#[opaque]
pub struct TypedFreeReceipt<T> { private: PhantomData<*mut T> }
impl<T> TypedFreeReceipt<T> {
    #[logic(opaque)] pub fn pointer(self) -> *mut T { dead }
    #[logic(opaque)] pub fn size(self) -> Int { dead }
    #[logic(opaque)] pub fn align(self) -> usize { dead }
    #[logic(opaque)] pub fn allocated(self) -> bool { dead }
    #[logic(opaque)] pub fn consumed(self, owner: Box<Perm<*const T>>) -> bool { dead }
}

/// Same native typed deallocation as the prior generic B4 adapter; additionally
/// returns its erased effect. Zero-sized ownership disposal performs no free.
#[trusted]
#[requires(*owner.inner_logic().ward() == pointer as *const T)]
#[ensures(result.inner_logic().pointer() == pointer)]
#[ensures(result.inner_logic().size() == creusot_std::std::mem::size_of_logic::<T>())]
#[ensures(result.inner_logic().align() == creusot_std::std::mem::align_of_logic::<T>())]
#[ensures(result.inner_logic().allocated() == (creusot_std::std::mem::size_of_logic::<T>() > 0))]
#[ensures(result.inner_logic().consumed(owner.inner_logic()))]
pub unsafe fn deallocate_typed_box<T>(pointer: *mut T, owner: Ghost<Box<Perm<*const T>>>) -> Ghost<TypedFreeReceipt<T>> {
    if core::mem::size_of::<T>() != 0 {
        unsafe { dealloc(pointer.cast(), Layout::new::<T>()) }
    }
    Ghost::conjure()
}
