//! Generic exact ptr.add projection. Live carries a real physical-region loan;
//! Zero permits only zero displacement of an unbound, nonnull u8 pointer.
//! Returned BoundPtr is metadata, never ownership or memory-access authority.
use creusot_std::prelude::*;
use crate::raw_vec::{BoundPtr,PhysicalRegion};
pub enum AdvanceLease<'a> { Live(&'a PhysicalRegion), Zero }

#[trusted]
#[requires(bound.inner_logic().invariant())]
#[requires(!pointer.is_null_logic())]
#[requires(pointer==bound.inner_logic().raw_pointer() as *const u8)]
#[requires(match lease.inner_logic() {
    AdvanceLease::Live(region)=>bound.inner_logic()@!=None && region.invariant() &&
        bound.inner_logic()@.unwrap_logic().0==region.namespace() &&
        bound.inner_logic()@.unwrap_logic().1==region.capacity() &&
        region.lo()<=bound.inner_logic()@.unwrap_logic().2 &&
        bound.inner_logic()@.unwrap_logic().2+count@<=region.hi(),
    AdvanceLease::Zero=>count==0usize && bound.inner_logic()@==None,
})]
#[ensures(result.0==result.1.inner_logic().raw_pointer() as *const u8)]
#[ensures(result.0.addr_logic()@==result.1.inner_logic().current_address())]
#[ensures(crate::view_pointer::shifted(*bound.inner_logic(),result.1.inner_logic(),count@))]
#[ensures(count==0usize ==> result.0==pointer)]
#[ensures(!result.0.is_null_logic())]
pub unsafe fn add(pointer:*const u8,count:usize,bound:Ghost<&BoundPtr>,
    lease:Ghost<AdvanceLease<'_>>)->(*const u8,Ghost<BoundPtr>) {
    (unsafe {pointer.add(count)},Ghost::conjure())
}
