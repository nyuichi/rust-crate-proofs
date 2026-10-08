//! Generic pointer projection of existing B3/B4 authority, never bytes ownership.
use creusot_std::prelude::*;
use crate::raw_vec::{self, BoundPtr, PhysicalRegion, Recovery};

#[trusted]
#[requires(bound.inner_logic().invariant() && bound.inner_logic()@ != None)]
#[requires(pointer == bound.inner_logic().raw_pointer() as *const u8)]
#[requires(region.inner_logic().invariant())]
#[requires(bound.inner_logic()@.unwrap_logic().0 == region.inner_logic().namespace())]
#[requires(bound.inner_logic()@.unwrap_logic().1 == region.inner_logic().capacity())]
#[requires(region.inner_logic().lo() <= bound.inner_logic()@.unwrap_logic().2)]
#[requires(bound.inner_logic()@.unwrap_logic().2 + len@ <= region.inner_logic().hi())]
#[requires(forall<i:Int> 0 <= i && i < len@ ==>
    raw_vec::slot_known(region.inner_logic().slot(bound.inner_logic()@.unwrap_logic().2+i)))]
#[ensures(result@.len() == len@)]
#[ensures(forall<i:Int> 0 <= i && i < len@ ==>
    region.inner_logic().slot(bound.inner_logic()@.unwrap_logic().2+i) == Some(Some(result@[i])))]
pub unsafe fn borrow<'a>(pointer:*const u8,len:usize,
    bound:Ghost<&'a BoundPtr>,region:Ghost<&'a PhysicalRegion>)->&'a [u8] {
    unsafe {core::slice::from_raw_parts(pointer,len)}
}

#[trusted]
#[requires(bound.inner_logic().invariant())]
#[requires(pointer == bound.inner_logic().raw_pointer() as *const u8)]
#[ensures(result@.len() == 0)]
pub unsafe fn borrow_empty<'a>(pointer:*const u8,bound:Ghost<&'a BoundPtr>)->&'a [u8] {
    unsafe {core::slice::from_raw_parts(pointer,0)}
}

#[trusted]
#[requires(bound.inner_logic().invariant())]
#[requires(pointer == bound.inner_logic().raw_pointer())]
#[requires(capabilities.inner_logic().0.invariant() && capabilities.inner_logic().1.invariant())]
#[requires(bound.inner_logic()@ == Some((capabilities.inner_logic().0.namespace(),capacity@,0int)))]
#[requires(capabilities.inner_logic().0.capacity() == capacity@)]
#[requires(capabilities.inner_logic().1.capacity() == capacity@)]
#[requires(capabilities.inner_logic().1.namespace() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0 && capabilities.inner_logic().1.hi() == capacity@)]
#[requires(capacity > 0usize)]
pub unsafe fn deallocate(pointer:*mut u8,capacity:usize,bound:Ghost<BoundPtr>,
    capabilities:Ghost<(Recovery,PhysicalRegion)>) {
    unsafe {alloc::alloc::dealloc(pointer,alloc::alloc::Layout::from_size_align(capacity,1).unwrap())}
}

/// Negative controls: neither raw-address substitution nor an out-of-region
/// read may borrow the initialized physical allocation through this boundary.
#[cfg(feature="negative_physical_projection")]
#[requires(bound.inner_logic().invariant())]
#[requires(pointer != bound.inner_logic().raw_pointer() as *const u8)]
pub unsafe fn wrong_pointer<'a>(pointer:*const u8,bound:Ghost<&'a BoundPtr>)->&'a [u8] {
    unsafe {borrow_empty(pointer,bound)}
}

#[cfg(feature="negative_physical_projection")]
#[requires(bound.inner_logic().invariant() && bound.inner_logic()@ != None)]
#[requires(pointer == bound.inner_logic().raw_pointer() as *const u8)]
#[requires(region.inner_logic().invariant())]
#[requires(bound.inner_logic()@.unwrap_logic().0 == region.inner_logic().namespace())]
#[requires(bound.inner_logic()@.unwrap_logic().1 == region.inner_logic().capacity())]
#[requires(region.inner_logic().lo() <= bound.inner_logic()@.unwrap_logic().2)]
#[requires(bound.inner_logic()@.unwrap_logic().2 + len@ > region.inner_logic().hi())]
pub unsafe fn wrong_bounds<'a>(pointer:*const u8,len:usize,
    bound:Ghost<&'a BoundPtr>,region:Ghost<&'a PhysicalRegion>)->&'a [u8] {
    unsafe {borrow(pointer,len,bound,region)}
}
