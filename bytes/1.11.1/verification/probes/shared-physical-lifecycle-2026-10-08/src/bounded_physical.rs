use super::*;
use creusot_std::ghost::lifetime_logic::{FullBorrow,EndBorrow};
use bounded::{Registry,RecoveryPayload};
use raw_vec::{Recovery,PhysicalRegion};
struct Bundle {recovery:Recovery,end:EndBorrow<PhysicalRegion>,shared:Ghost<frozen::FrozenRegion>}
impl RecoveryPayload for Bundle {
    type Metadata=(Lifetime,creusot_std::logic::Id,Int);
    #[logic] fn metadata(self)->Self::Metadata {(self.end.lft(),self.recovery.namespace(),self.recovery.capacity())}
    #[logic(prophetic)] fn wellformed(self)->bool {pearlite! {
        self.recovery.invariant() && self.shared.val().cur().invariant() &&
        self.shared.val().cur().namespace()==self.recovery.namespace() &&
        self.shared.val().cur().resource_id()==self.recovery.namespace() &&
        self.shared.val().cur().capacity()==self.recovery.capacity() &&
        self.shared.val().cur().lo()==0 && self.shared.val().cur().hi()==self.recovery.capacity() &&
        self.end.lft()==self.shared.val().lft() && ^self.end==self.shared.val().cur()
    }}
}

/// Exactly one native clone under a constructor-issued affine quota; either
/// handle may retire last and obtain the actual recovery bundle after Acquire.
#[ensures(result.2)]
#[ensures(input@.len()>0 ==> result.0==input@[0] && result.1==input@[0])]
#[ensures(input@.len()==0 ==> result.0==0u8 && result.1==0u8)]
pub fn clone_read_release(input:Vec<u8>,reverse:bool)->(u8,u8,bool) {
    let contents=snapshot!(input@);
    let (base,len,capacity,caps)=bound_ptr::detach_bound_vec(input);
    let (recovery,region)=caps.split();
    let lifetime=ghost! {LifetimeToken::new()};
    let (full,end)=FullBorrow::new(region,snapshot!(lifetime.lft()));
    let shared=ghost! {GhostShared::new(full).into_inner()};
    let bundle=ghost! {Bundle{recovery:recovery.into_inner(),end:end.into_inner(),shared}};
    let (registry,first,quota)=Registry::new(bundle,lifetime);
    let (old,second)=registry.clone_bounded(first.borrow(),quota);
    proof_assert!(old==1usize);
    let (first,second)=if reverse {(second,first)}else{(first,second)};
    let bytes=frozen::borrow_frozen(&base,len,shared,ghost! {&first.token});
    proof_assert!(bytes@==*contents);
    let a=if len==0 {0}else{bytes[0]};
    let first_closed=registry.retire(first);
    let bytes=frozen::borrow_frozen(&base,len,shared,ghost! {&second.token});
    proof_assert!(bytes@==*contents);
    let b=if len==0 {0}else{bytes[0]};
    let second_closed=registry.retire(second);
    registry.finish(first_closed.0,first_closed.2,second_closed.0,second_closed.2);
    let selected=if first_closed.0 {first_closed.1}else{second_closed.1};
    let capabilities=ghost! {
        let (bundle,full)=selected.into_inner().unwrap();
        let dead=full.end();let region=bundle.end.get(dead);
        (bundle.recovery,region)
    };
    unsafe {raw_vec::deallocate_bound_vec(base,capacity,capabilities);}
    (a,b,true)
}
#[cfg(all(test,not(creusot)))]
mod tests {
    #[test] fn both_release_orders() {
        for len in [0,1,8] {for spare in [0,9] {for reverse in [false,true] {
            let mut input=Vec::with_capacity(len+spare);input.resize(len,29);
            assert_eq!(super::clone_read_release(input,reverse),(if len==0{0}else{29},if len==0{0}else{29},true));
        }}}
    }
}
