//! Read-only sharing of an existing physical region through lifetime fractions.
//! This adds no physical primitive: the final read uses the existing B4-read
//! bridge, and ending the full synthetic lifetime returns the original region.

use creusot_std::{
    ghost::{lifetime_logic::{FullBorrow, LifetimeToken}, GhostShared},
    prelude::*,
};
use super::raw_vec::{self, BoundPtr, PhysicalRegion};

pub(crate) type FrozenRegion = GhostShared<FullBorrow<PhysicalRegion>>;

/// Each independent token fraction supports an immutable borrow. The reference
/// lifetime is tied to both its descriptor and its own lifetime-token borrow.
#[requires(bound.invariant() && bound@ != None)]
#[requires(shared.val().cur().invariant())]
#[requires(shared.val().lft() == ticket.lft())]
#[requires(bound@.unwrap_logic().0 == shared.val().cur().namespace())]
#[requires(bound@.unwrap_logic().1 == shared.val().cur().capacity())]
#[requires(shared.val().cur().lo() <= bound@.unwrap_logic().2)]
#[requires(bound@.unwrap_logic().2 + len@ <= shared.val().cur().hi())]
#[requires(forall<offset: Int> 0 <= offset && offset < len@ ==>
    raw_vec::slot_known(shared.val().cur().slot(bound@.unwrap_logic().2 + offset)))]
#[ensures(result@.len() == len@)]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    shared.val().cur().slot(bound@.unwrap_logic().2 + offset) == Some(Some(result@[offset])))]
pub(crate) fn borrow_frozen<'a>(
    bound: &'a BoundPtr,
    len: usize,
    shared: Ghost<FrozenRegion>,
    ticket: &'a LifetimeToken,
) -> &'a [u8] {
    let region: Ghost<&'a PhysicalRegion> = ghost! {
        let full: &'a FullBorrow<PhysicalRegion> = (*shared).to_ref();
        full.borrow(ticket)
    };
    unsafe { raw_vec::borrow_bound(bound, len, region) }
}

/// Affine read ticket carried by the extraction-only actual Bytes receiver.
/// The shared ghost view is copyable; the lifetime fraction is not.
pub(crate) struct FrozenReader {
    pub(crate) shared: Ghost<FrozenRegion>,
    pub(crate) ticket: LifetimeToken,
}

impl FrozenReader {
    #[logic(open(crate), prophetic)]
    pub(crate) fn valid(self, bound: BoundPtr, len: usize) -> bool {
        pearlite! {
            bound.invariant() && bound@ != None &&
            self.shared.val().cur().invariant() &&
            self.shared.val().lft() == self.ticket.lft() &&
            bound@.unwrap_logic().0 == self.shared.val().cur().namespace() &&
            bound@.unwrap_logic().1 == self.shared.val().cur().capacity() &&
            self.shared.val().cur().lo() <= bound@.unwrap_logic().2 &&
            bound@.unwrap_logic().2 + len@ <= self.shared.val().cur().hi() &&
            bound@.unwrap_logic().2 + len@ <= bound@.unwrap_logic().1 &&
            forall<offset: Int> 0 <= offset && offset < len@ ==>
                raw_vec::slot_known(self.shared.val().cur().slot(bound@.unwrap_logic().2 + offset))
        }
    }
}

/// Explicit sequential recovery coordinator. Native `Bytes::Drop` dispatch is
/// outside this adapter: clients must return every read ticket before reclaim.
pub(crate) struct FrozenOwner {
    pub(crate) base: BoundPtr,
    pub(crate) capacity: usize,
    pub(crate) recovery: Ghost<raw_vec::Recovery>,
    pub(crate) end: Ghost<creusot_std::ghost::lifetime_logic::EndBorrow<PhysicalRegion>>,
    pub(crate) shared: Ghost<FrozenRegion>,
}

impl FrozenOwner {
    #[logic(open(crate), prophetic)]
    pub(crate) fn valid(self) -> bool {
        pearlite! {
            self.base.invariant() && self.recovery.inner_logic().invariant() &&
            self.base@ == Some((self.recovery.namespace(),self.capacity@,0int)) &&
            self.recovery.capacity() == self.capacity@ &&
            self.shared.val().cur().invariant() &&
            self.shared.val().cur().namespace() == self.recovery.namespace() &&
            self.shared.val().cur().resource_id() == self.recovery.namespace() &&
            self.shared.val().cur().capacity() == self.capacity@ &&
            self.shared.val().cur().lo() == 0 && self.shared.val().cur().hi() == self.capacity@ &&
            self.end.lft() == self.shared.val().lft() &&
            ^self.end == self.shared.val().cur()
        }
    }

    #[requires(bound.invariant())]
    #[requires(capabilities.inner_logic().0.invariant() && capabilities.inner_logic().1.invariant())]
    #[requires(bound@ == Some((capabilities.inner_logic().0.namespace(), capacity@, 0int)))]
    #[requires(capabilities.inner_logic().0.capacity() == capacity@)]
    #[requires(capabilities.inner_logic().1.capacity() == capacity@)]
    #[requires(capabilities.inner_logic().1.namespace() == capabilities.inner_logic().0.namespace())]
    #[requires(capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace())]
    #[requires(capabilities.inner_logic().1.lo() == 0 && capabilities.inner_logic().1.hi() == capacity@)]
    #[requires(len <= capacity)]
    #[requires(forall<i:Int> 0 <= i && i < len@ ==> raw_vec::slot_known(capabilities.inner_logic().1.slot(i)))]
    #[ensures(result.0.valid() && result.1.valid(bound,len))]
    #[ensures(result.0.shared == result.1.shared)]
    #[ensures(result.1.ticket.frac() == creusot_std::logic::real::PositiveReal::from_int(1))]
    #[ensures(result.1.shared.val().cur() == capabilities.inner_logic().1)]
    pub(crate) fn new(bound:BoundPtr, capacity:usize, len:usize,
        capabilities:Ghost<(raw_vec::Recovery,PhysicalRegion)>) -> (Self,FrozenReader) {
        let (recovery,region)=capabilities.split();
        let ticket=LifetimeToken::new();
        let (full,end)=FullBorrow::new(region,snapshot!(ticket.lft()));
        let shared=ghost! { GhostShared::new(full).into_inner() };
        (Self{base:bound,capacity,recovery,end,shared},FrozenReader{shared,ticket})
    }

    #[requires(self.valid())]
    #[requires(ticket.lft() == self.shared.val().lft())]
    #[requires(ticket.frac() == creusot_std::logic::real::PositiveReal::from_int(1))]
    pub(crate) fn reclaim(self,ticket:LifetimeToken) {
        let dead=ticket.end();
        let region=ghost! { self.end.into_inner().get(dead) };
        unsafe { raw_vec::deallocate_bound_vec(self.base,self.capacity,
            ghost! { (self.recovery.into_inner(),region.into_inner()) }); }
    }
}
