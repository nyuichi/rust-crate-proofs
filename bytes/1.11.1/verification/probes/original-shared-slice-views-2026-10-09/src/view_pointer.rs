//! Generic native pointer metadata, without ownership or read authority.
//! `add_live` uses an existing live physical borrow; `wrapping_bounded` only
//! adjusts metadata. `without_provenance` produces unbound, provenance-free
//! metadata at the exact address used by the native bytes helper.
use creusot_std::prelude::*;
use crate::raw_vec::{BoundPtr,PhysicalRegion};

#[logic(open(crate))]
pub fn shifted(before:BoundPtr,after:BoundPtr,count:Int)->bool {
    pearlite! {after.invariant() && after.current_address()==before.current_address()+count &&
        match before@ {
            Some((ns,cap,offset))=>after@==Some((ns,cap,offset+count)),
            None=>count==0 && after@==None,
        }}
}

#[trusted]
#[requires(bound.inner_logic().invariant() && bound.inner_logic()@!=None)]
#[requires(pointer==bound.inner_logic().raw_pointer() as *const u8)]
#[requires(region.inner_logic().invariant())]
#[requires(bound.inner_logic()@.unwrap_logic().0==region.inner_logic().namespace())]
#[requires(bound.inner_logic()@.unwrap_logic().1==region.inner_logic().capacity())]
#[requires(region.inner_logic().lo()<=bound.inner_logic()@.unwrap_logic().2)]
#[requires(bound.inner_logic()@.unwrap_logic().2+count@<=region.inner_logic().hi())]
#[ensures(result.0==result.1.inner_logic().raw_pointer() as *const u8)]
#[ensures(result.0.addr_logic()@==result.1.inner_logic().current_address())]
#[ensures(shifted(*bound.inner_logic(),result.1.inner_logic(),count@))]
#[ensures(!result.0.is_null_logic())]
pub unsafe fn add_live(pointer:*const u8,count:usize,bound:Ghost<&BoundPtr>,
    region:Ghost<&PhysicalRegion>)->(*const u8,Ghost<BoundPtr>) {
    (unsafe {pointer.add(count)},Ghost::conjure())
}

#[trusted]
#[requires(bound.inner_logic().invariant())]
#[requires(pointer==bound.inner_logic().raw_pointer() as *const u8)]
#[requires(match bound.inner_logic()@ {
    Some((_,cap,offset))=>offset+count@<=cap,None=>count==0usize})]
#[ensures(result.0==result.1.inner_logic().raw_pointer() as *const u8)]
#[ensures(result.0.addr_logic()@==result.1.inner_logic().current_address())]
#[ensures(shifted(*bound.inner_logic(),result.1.inner_logic(),count@))]
#[ensures(!result.0.is_null_logic())]
pub fn wrapping_bounded(pointer:*const u8,count:usize,bound:Ghost<&BoundPtr>)->(*const u8,Ghost<BoundPtr>) {
    (pointer.wrapping_add(count),Ghost::conjure())
}

#[trusted]
#[requires(!pointer.is_null_logic())]
#[ensures(result.1.inner_logic().invariant() && result.1.inner_logic()@==None)]
#[ensures(result.0==result.1.inner_logic().raw_pointer() as *const u8)]
#[ensures(result.0.addr_logic()@==result.1.inner_logic().current_address())]
#[ensures(result.1.inner_logic().current_address()==pointer.addr_logic()@)]
#[ensures(result.0.addr_logic()==pointer.addr_logic() && !result.0.is_null_logic())]
pub fn without_provenance(pointer:*const u8)->(*const u8,Ghost<BoundPtr>) {
    (core::ptr::null::<u8>().wrapping_add(pointer as usize),Ghost::conjure())
}

/// Std specifies nullness by address. This exact native null word is additionally
/// reified for readonly atomic binding; it contains no allocation authority.
#[logic(opaque)] pub fn null_word()->*mut () {dead}
#[trusted]
#[ensures(result==null_word() && result.is_null_logic())]
pub fn null_pointer()->*mut () {core::ptr::null_mut()}
