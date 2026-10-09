//! Generic same-allocation pointer distance; no Bytes or capacity-recovery law.
//! Sealed descriptors and a live physical-region borrow establish provenance.
//! The exact native offset_from remains an explicitly assumed pointer boundary.
use creusot_std::prelude::*;
use crate::raw_vec::{BoundPtr,PhysicalRegion};
#[trusted]
#[requires(view.inner_logic().invariant() && origin.inner_logic().invariant())]
#[requires(view.inner_logic()@!=None && origin.inner_logic()@!=None)]
#[requires(region.inner_logic().invariant() && region.inner_logic().capacity()>0)]
#[requires(region.inner_logic().lo()==0 && region.inner_logic().hi()==region.inner_logic().capacity())]
#[requires(origin.inner_logic()@==Some((region.inner_logic().namespace(),region.inner_logic().capacity(),0int)))]
#[requires(view.inner_logic()@.unwrap_logic().0==region.inner_logic().namespace())]
#[requires(view.inner_logic()@.unwrap_logic().1==region.inner_logic().capacity())]
#[requires(pointer==view.inner_logic().raw_pointer() as *const u8 && base==origin.inner_logic().raw_pointer())]
#[requires(!pointer.is_null_logic() && !base.is_null_logic())]
#[requires(view.inner_logic().current_address()==pointer.addr_logic()@)]
#[requires(origin.inner_logic().current_address()==base.addr_logic()@)]
#[requires(view.inner_logic().current_address()==origin.inner_logic().current_address()+view.inner_logic()@.unwrap_logic().2)]
#[ensures(result@==view.inner_logic()@.unwrap_logic().2)]
#[ensures(0<=result@ && result@<=region.inner_logic().capacity())]
pub unsafe fn distance(pointer:*const u8,base:*mut u8,view:Ghost<&BoundPtr>,
    origin:Ghost<&BoundPtr>,region:Ghost<&PhysicalRegion>)->isize {
    unsafe {pointer.offset_from(base)}
}
