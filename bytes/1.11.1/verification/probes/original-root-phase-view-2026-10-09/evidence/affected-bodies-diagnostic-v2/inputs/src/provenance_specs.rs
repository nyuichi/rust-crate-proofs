//! Source-gated pointer wrappers and the existing STD-PTRWRAP-01 address-only contract.
//! RawVec offset helpers remain included and proved, even though the selected client does not call them.
#[cfg(creusot)] use creusot_std::std::mem::size_of_logic;
// STD-PTRWRAP-01: Creusot 0.13 has no contract for raw-pointer wrapping_add.
// This extern spec is usable only for one-byte pointees (the u8 operation used
// by pointer tagging) and states only the numeric address calculation. It says
// nothing about pointer equality, provenance, live ranges, `Perm`, `PtrLive`,
// or dereferenceability.
#[cfg(creusot)]
extern_spec! {
    impl<T> *mut T {
        #[requires(size_of_logic::<T>() == 1)]
        #[ensures(
            result.addr_logic()@ ==
                (self.addr_logic()@ + offset@) % (usize::MAX@ + 1)
        )]
        fn wrapping_add(self, offset: usize) -> *mut T;
    }
}

#[cfg(creusot)]
use creusot_std::prelude::*;

#[cfg_attr(creusot, ensures(result == ptr.addr_logic()))]
#[cfg_attr(creusot, check(ghost))]
pub(crate) fn pointer_addr<T>(ptr: *const T) -> usize {
    ptr.addr()
}

#[cfg_attr(creusot, ensures(
    result.addr_logic()@ == (ptr.addr_logic()@ + offset@) % (usize::MAX@ + 1)
))]
pub(crate) fn wrapping_offset(ptr: *mut u8, offset: usize) -> *mut u8 {
    ptr.wrapping_add(offset)
}
