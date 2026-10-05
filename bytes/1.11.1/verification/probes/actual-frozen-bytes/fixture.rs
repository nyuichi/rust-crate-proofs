use creusot_std::ghost::{
    lifetime_logic::{FullBorrow, LifetimeToken}, GhostShared,
};
use crate::frozen_region::FrozenReader;

#[requires(at@ <= input@.len())]
#[ensures(input@.len() > 0 ==> result.0 == input@[0])]
#[ensures(at@ < input@.len() ==> result.1 == input@[at@])]
pub(crate) fn actual_receiver_views(input: Vec<u8>, at: usize, table: &'static Vtable) -> (u8, u8) {
    let (base, len, capacity, capabilities) = crate::bound_ptr::detach_bound_vec(input);
    let (recovery, region) = capabilities.split();
    let lifetime = LifetimeToken::new();
    let (full, end) = FullBorrow::new(region, snapshot!(lifetime.lft()));
    let shared = ghost! { GhostShared::new(full).into_inner() };
    let (first, second) = lifetime.split();
    let suffix = base.advance_within(at);
    let first = unsafe { Bytes::with_vtable(base, len, AtomicPtr::new(core::ptr::null_mut()), table,
        FrozenReader { shared, ticket: first }) };
    let second = unsafe { Bytes::with_vtable(suffix, len - at, AtomicPtr::new(core::ptr::null_mut()), table,
        FrozenReader { shared, ticket: second }) };
    let all = first.as_slice();
    let tail = second.as_slice();
    proof_assert!(forall<i: Int> 0 <= i && i < (len-at)@ ==> tail@[i] == all@[at@+i]);
    let a = if len == 0 { 0 } else { all[0] };
    let b = if len == at { 0 } else { tail[0] };
    let _ = (all, tail);
    let first = first.proof_return_frozen_ticket();
    let second = second.proof_return_frozen_ticket();
    let dead = second.join(first).end();
    let region = ghost! { end.into_inner().get(dead) };
    unsafe { crate::raw_vec::deallocate_bound_vec(base, capacity,
        ghost! { (recovery.into_inner(), region.into_inner()) }); }
    (a,b)
}

#[cfg(feature = "negative_shared_clone")]
impl Bytes {
    // Exact receiver and signature of Clone::clone. The attempted affine
    // witness update is rejected because cloning only grants shared access.
    fn attempt_clone_from_shared_receiver(&self) -> LifetimeToken {
        self.frozen.as_ref().unwrap().ticket.split_off()
    }
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::*;
    unsafe fn forbidden_clone(_: &AtomicPtr<()>, _: *const u8, _: usize, _: &'static Vtable) -> Bytes { panic!("dispatch excluded") }
    unsafe fn forbidden_vec(_: &AtomicPtr<()>, _: *const u8, _: usize) -> Vec<u8> { panic!("dispatch excluded") }
    unsafe fn forbidden_mut(_: &AtomicPtr<()>, _: *const u8, _: usize) -> crate::BytesMut { panic!("dispatch excluded") }
    unsafe fn forbidden_unique(_: &AtomicPtr<()>) -> bool { panic!("dispatch excluded") }
    unsafe fn forbidden_drop(_: &mut AtomicPtr<()>, _: *const u8, _: usize) { panic!("dispatch excluded") }
    static TABLE: Vtable = Vtable { clone: forbidden_clone, into_vec: forbidden_vec, into_mut: forbidden_mut, is_unique: forbidden_unique, drop: forbidden_drop };
    #[test]
    fn actual_receiver_reads_and_explicit_close() {
        for values in [alloc::vec![], alloc::vec![7], alloc::vec![2,4,6,8]] {
            for at in 0..=values.len() {
                let expected = (values.first().copied().unwrap_or(0), values.get(at).copied().unwrap_or(0));
                assert_eq!(actual_receiver_views(values.clone(), at, &TABLE), expected);
                assert_eq!(actual_freeze_then_read_and_reclaim(values.clone(), &TABLE),expected.0);
            }
        }
    }
}

#[ensures(input@.len() > 0 ==> result == input@[0])]
pub(crate) fn actual_freeze_then_read_and_reclaim(input:Vec<u8>,table:&'static Vtable)->u8 {
    let mutable=BytesMut::from_vec(input);
    let mut coordinator=None;
    let mut bytes=mutable.freeze(&mut coordinator,table);
    let other=bytes.proof_share_frozen(0,bytes.len);
    let contents=bytes.as_slice();
    let first=if contents.len() == 0 {0} else {contents[0]};
    let _=contents;
    let ticket=bytes.proof_return_frozen_ticket();
    let other_ticket=other.proof_return_frozen_ticket();
    let ticket=ticket.join(other_ticket);
    proof_assert!(ticket.frac().ext_eq(creusot_std::logic::real::PositiveReal::from_int(1)));
    coordinator.unwrap().reclaim(ticket);
    first
}
