#![allow(unexpected_cfgs,unused_variables,dead_code)]
#![recursion_limit="512"]
extern crate alloc;
use creusot_std::{prelude::*,ghost::{Perm,GhostShared,lifetime_logic::{LifetimeToken,Lifetime}},
    std::sync::{atomic::{AtomicUsize as ModelAtomic,ordering::{Relaxed,Release,Acquire,None as NoStore}},
        committer::Committer,view::{AtView,SyncView,HasTimestamp}}};
#[path="../../../../src/provenance_specs.rs"] mod provenance_specs;
#[path="../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[path="../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[path="../../../../src/ownership_proof/bound_ptr.rs"] mod bound_ptr;
mod frozen;
mod event;
mod release;
mod retirement;
use event::{RawAtomic,EventAtomic,EventProtocol};
use retirement::{SharedRetirement,Payload};
use creusot_std::logic::real::PositiveReal;
impl Payload for LifetimeToken {
    type Metadata=(Lifetime,PositiveReal);
    #[logic] fn metadata(self)->Self::Metadata {(self.lft(),self.frac())}
    #[logic(prophetic)] fn wellformed(self)->bool {true}
}

/// Interim physical component: cleanup, when performed, consumes the actual B1
/// region after both affine reader tokens have crossed the Release/Acquire path.
/// The returned bool exposes whether the bounded caller completed cleanup; this
/// first gate does not infer eventual cleanup from a native XOR assertion.
#[ensures(input@.len()>0 ==> result.0==input@[0] && result.1==input@[0])]
#[ensures(input@.len()==0 ==> result.0==0u8 && result.1==0u8)]
pub fn physical_roundtrip(input:Vec<u8>)->(u8,u8,bool) {
    let (base,len,capacity,caps)=bound_ptr::detach_bound_vec(input);
    let (owner,reader)=frozen::FrozenOwner::new(base,capacity,len,caps);
    let (left,right)=ghost! {reader.ticket.into_inner().split()}.split();
    let expected=snapshot!((left.metadata(),right.metadata()));
    let (machine,a,b)=SharedRetirement::new(expected);
    require_sync(&machine);
    let first=frozen::borrow_frozen(&owner.base,len,reader.shared,left.borrow());
    let a_byte=if len==0 {0} else {first[0]};
    let first_closed=machine.retire(a,left);
    // Explicit read after the peer retired, while this reader remains live.
    let second=frozen::borrow_frozen(&owner.base,len,reader.shared,right.borrow());
    let b_byte=if len==0 {0} else {second[0]};
    let second_closed=machine.retire(b,right);
    let (done,tokens)=if first_closed.0 {first_closed} else {second_closed};
    if done {
        let full=ghost! {
            let (a,b)=tokens.into_inner().unwrap();
            let full=a.join(b);
            proof_assert!(full.frac().ext_eq(PositiveReal::from_int(1)));
            full
        };
        let dead=ghost! {full.into_inner().end()};
        let region=ghost! {owner.end.into_inner().get(dead.into_inner())};
        unsafe {raw_vec::deallocate_bound_vec(owner.base,owner.capacity,
            ghost! {(owner.recovery.into_inner(),region.into_inner())});}
    }
    (a_byte,b_byte,done)
}
fn require_sync<T:Sync>(_:&T) {}

#[cfg(feature="negative_missing_ticket")]
pub fn premature_end(input:Vec<u8>) {
    let (base,len,cap,caps)=bound_ptr::detach_bound_vec(input);
    let (owner,reader)=frozen::FrozenOwner::new(base,cap,len,caps);
    let (left,_right)=ghost! {reader.ticket.into_inner().split()}.split();
    owner.reclaim(left);
}
#[cfg(feature="negative_duplicate")]
fn duplicate_token(token:LifetimeToken)->(LifetimeToken,LifetimeToken) {(token,token)}

#[cfg(all(test,not(creusot)))]
mod tests {
    #[test] fn physical_bytes_and_empty_allocations() {
        for len in [0,1,8] { for spare in [0,9] {
            let mut input=Vec::with_capacity(len+spare); input.resize(len,37);
            let result=super::physical_roundtrip(input);
            assert_eq!(result,(if len==0 {0}else{37},if len==0 {0}else{37},true));
        }}
    }
}
