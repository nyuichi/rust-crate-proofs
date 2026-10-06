//! Explicit scoped readers for the verified modified bytes variant.
//! Allocation ownership remains in Owner; every reader closes explicitly.
#![allow(missing_debug_implementations, unexpected_cfgs, dead_code)]

use crate::ownership_proof::{bound_ptr, frozen_region, raw_vec};
use alloc::vec::Vec;
use creusot_std::{prelude::*, ghost::{Perm, invariant::Tokens, lifetime_logic::{Lifetime,LifetimeToken,FullBorrow}, GhostShared},
    logic::real::PositiveReal,
    std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::{Acquire,Relaxed,Release,None as NoStore}},
        committer::Committer, view::{AtView,HasTimestamp,SyncView}}};
mod atomic;
use atomic as primitive;
use atomic::NativeAtomic;
mod retirement;
mod tree;
pub mod exclusive;
pub use tree::scoped_tree;
use retirement::{Payload,SharedRetirement,Ticket,Receipt};

impl Payload for LifetimeToken {
    type Metadata=(Lifetime,PositiveReal);
    #[logic]
    fn metadata(self)->Self::Metadata { (self.lft(),self.frac()) }
    #[logic(prophetic)]
    fn wellformed(self)->bool { self.frac() <= PositiveReal::from_int(1) }
}

/// Owns detached physical storage. No ordinary Vec remains after construction.
pub(crate) struct Owner {
    frozen:frozen_region::FrozenOwner,
    anchor:Ghost<LifetimeToken>,
    len:usize,
    model:Snapshot<Seq<u8>>,
}

/// Scoped immutable bytes plus an affine close obligation, including when empty.
pub(crate) struct ReadHandle<'a> {
    bytes:&'a [u8],
    permit:Ghost<LifetimeToken>,
    ticket:Ghost<Ticket>,
}

/// Two-reader native weak-refcount protocol. It is finalized after both joins.
pub(crate) struct CloseContext { machine:SharedRetirement<LifetimeToken> }

/// Result of one explicit close; capabilities are available only to the last close.
pub(crate) struct Closed {
    last:bool,
    recovered:Ghost<Option<(LifetimeToken,LifetimeToken)>>,
    receipt:Ghost<Receipt>,
}

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
#[cfg_attr(creusot, check(ghost))]
fn borrow_anchored<'a>(
    bound: &'a raw_vec::BoundPtr,
    len: usize,
    shared: Ghost<frozen_region::FrozenRegion>,
    ticket: Ghost<&'a LifetimeToken>,
) -> &'a [u8] {
    let region: Ghost<&'a raw_vec::PhysicalRegion> = ghost! {
        let full: &'a FullBorrow<raw_vec::PhysicalRegion> = (*shared).to_ref();
        full.borrow(ticket.into_inner())
    };
    unsafe { raw_vec::borrow_bound(bound, len, region) }
}


impl Owner {
    #[logic(open(self), prophetic)]
    pub(crate) fn valid(self)->bool {
        pearlite! {
            self.frozen.valid() && self.anchor.lft() == self.frozen.shared.val().lft() &&
            self.len@ == self.model.len() && self.len <= self.frozen.capacity &&
            forall<i:Int> 0 <= i && i < self.len@ ==>
                self.frozen.shared.val().cur().slot(i) == Some(Some(self.model[i]))
        }
    }
    #[logic]
    pub fn contents(self)->Seq<u8> { *self.model }

    #[ensures(result.valid())]
    #[ensures(result.contents() == input@)]
    #[ensures(result.anchor.frac() == PositiveReal::from_int(1))]
    pub fn new(input:Vec<u8>)->Self {
        let model=snapshot!(input@);
        let (base,len,capacity,caps)=bound_ptr::detach_bound_vec(input);
        let (recovery,region)=caps.split();
        let anchor=ghost! { LifetimeToken::new() };
        let (full,end)=FullBorrow::new(region,snapshot!(anchor.lft()));
        let shared=ghost! { GhostShared::new(full).into_inner() };
        let frozen=frozen_region::FrozenOwner{base,capacity,recovery,end,shared};
        Self{frozen,anchor,len,model}
    }

    #[requires(self.valid())]
    #[requires(self.anchor.frac() == PositiveReal::from_int(1))]
    #[ensures((^self).valid() && (^self).contents() == self.contents())]
    #[ensures((^self).anchor.frac() == PositiveReal::from_int(1)/PositiveReal::from_int(2))]
    #[ensures(result.0.contents() == self.contents() && result.1.contents() == self.contents())]
    #[ensures(result.2.valid())]
    #[ensures(result.0.accepted_by(result.2) && result.1.accepted_by(result.2))]
    #[ensures(result.0.is_left() && !result.1.is_left())]
    #[ensures(result.2.matches_owner(^self))]
    pub fn share_pair(&mut self)->(ReadHandle<'_>,ReadHandle<'_>,CloseContext) {
        let pair=ghost! { self.anchor.split_off().split() };
        let (left,right)=pair.split();
        let expected=snapshot!((left.metadata(),right.metadata()));
        let (machine,left_ticket,right_ticket)=SharedRetirement::new(expected);
        let bytes=borrow_anchored(&self.frozen.base,self.len,self.frozen.shared,ghost!{ &*self.anchor });
        (ReadHandle{bytes,permit:left,ticket:left_ticket},
         ReadHandle{bytes,permit:right,ticket:right_ticket},CloseContext{machine})
    }

    /// Both close receipts are checked before recovering the full synthetic lifetime.
    #[requires(self.valid())]
    #[requires(context.valid() && context.matches_owner(self))]
    #[requires(first.accepted_by(context) && second.accepted_by(context))]
    #[requires(first.is_left() != second.is_left())]
    #[ensures(result.0 != result.1)]
    pub fn close(self,context:CloseContext,first:Closed,second:Closed)->(bool,bool) {
        let first_last=first.last;
        let second_last=second.last;
        context.machine.finish(first_last,first.receipt,second_last,second.receipt);
        let recovered=if first_last {first.recovered} else {second.recovered};
        let caps=ghost! {
            let (left,right)=recovered.into_inner().unwrap();
            let full=self.anchor.into_inner().join(left).join(right);
            proof_assert!(full.frac().ext_eq(PositiveReal::from_int(1)));
            let region=self.frozen.end.into_inner().get(full.end());
            (self.frozen.recovery.into_inner(),region)
        };
        unsafe { raw_vec::deallocate_bound_vec(self.frozen.base,self.frozen.capacity,caps); }
        (first_last,second_last)
    }
}

impl CloseContext {
    #[logic]
    pub fn valid(self)->bool { self.machine.valid() }
    #[logic(open(self), prophetic)]
    pub(crate) fn matches_owner(self,owner:Owner)->bool {
        pearlite! {
            owner.anchor.frac() == PositiveReal::from_int(1)/PositiveReal::from_int(2) &&
            self.machine.expected().0 == (owner.anchor.lft(),PositiveReal::from_int(1)/PositiveReal::from_int(4)) &&
            self.machine.expected().1 == (owner.anchor.lft(),PositiveReal::from_int(1)/PositiveReal::from_int(4))
        }
    }
}

impl<'a> ReadHandle<'a> {
    #[logic]
    pub fn contents(self)->Seq<u8> { pearlite!{ self.bytes@ } }
    #[logic]
    pub fn is_left(self)->bool { self.ticket.is_left() }
    #[logic(open(self), prophetic)]
    pub fn accepted_by(self,context:CloseContext)->bool {
        context.machine.accepts_payload(self.ticket.inner_logic(),self.permit.inner_logic())
    }
    #[ensures(result@ == self.contents().len())]
    pub fn len(&self)->usize { self.bytes.len() }
    #[requires(index@ < self.contents().len())]
    #[ensures(result == self.contents()[index@])]
    pub fn read(&self,index:usize)->u8 { self.bytes[index] }

    #[requires(context.valid() && self.accepted_by(*context))]
    #[requires(tokens.contains(retirement::PUBLICATION()))]
    #[ensures(result.accepted_by(*context))]
    #[ensures(result.is_left() == self.is_left())]
    pub fn close(self,context:&CloseContext,tokens:Ghost<Tokens>)->Closed {
        let (last,recovered,receipt)=context.machine.retire(self.ticket,self.permit,tokens);
        Closed{last,recovered,receipt}
    }
}

impl Closed {
    #[logic]
    pub fn is_left(self)->bool { self.receipt.is_left() }
    #[logic(open(self), prophetic)]
    pub fn accepted_by(self,context:CloseContext)->bool {
        pearlite! {
            context.machine.accepts_receipt(self.receipt.inner_logic(),self.last) &&
            self.last == (self.recovered.inner_logic() != None) &&
            (self.last ==> (self.recovered.inner_logic().unwrap_logic().0.metadata(),
                self.recovered.inner_logic().unwrap_logic().1.metadata()) == context.machine.expected())
        }
    }
}

/// Normal-return architecture witness: both children read real bytes, close,
/// join, and cause one unconditional consuming Owner cleanup.
#[ensures(result.0 == if index@ < input@.len() {Some(input@[index@])} else {None})]
#[ensures(result.1 == result.0)]
#[ensures(result.2 != result.3)]
pub fn scoped_roundtrip(input:Vec<u8>,index:usize,reverse:bool)->(Option<u8>,Option<u8>,bool,bool) {
    use creusot_std::std::thread::{self,JoinHandleExt};
    let mut owner=Owner::new(input);
    let (left,right,context)=owner.share_pair();
    let (first,second)=if reverse {(right,left)} else {(left,right)};
    let context_ref=&context;
    let (first,second)=thread::scope(move |scope| {
        let first=scope.spawn(move |tokens| {
            let byte=if index < first.len() {Some(first.read(index))} else {None};
            (byte,first.close(context_ref,tokens))
        });
        let second=scope.spawn(move |tokens| {
            let byte=if index < second.len() {Some(second.read(index))} else {None};
            (byte,second.close(context_ref,tokens))
        });
        (first.join_unwrap(),second.join_unwrap())
    });
    let flags=owner.close(context,first.1,second.1);
    (first.0,second.0,flags.0,flags.1)
}

/// Ordered witness: the first worker closes and joins while the second reader
/// remains live; a second real worker then reads those bytes and closes.
#[ensures(result.0 == if index@ < input@.len() {Some(input@[index@])} else {None})]
#[ensures(result.1 == result.0)]
#[ensures(result.2 != result.3)]
pub fn scoped_after_peer_close(input:Vec<u8>,index:usize,reverse:bool)->(Option<u8>,Option<u8>,bool,bool) {
    use creusot_std::std::thread::{self,JoinHandleExt};
    let mut owner=Owner::new(input);
    let (left,right,context)=owner.share_pair();
    let (first,second)=if reverse {(right,left)} else {(left,right)};
    let context_ref=&context;
    let (first,second)=thread::scope(move |scope| {
        let first=scope.spawn(move |tokens| {
            let byte=if index < first.len() {Some(first.read(index))} else {None};
            (byte,first.close(context_ref,tokens))
        });
        let first=first.join_unwrap();
        let second=scope.spawn(move |tokens| {
            let byte=if index < second.len() {Some(second.read(index))} else {None};
            (byte,second.close(context_ref,tokens))
        });
        (first,second.join_unwrap())
    });
    let flags=owner.close(context,first.1,second.1);
    (first.0,second.0,flags.0,flags.1)
}

/// Checked exclusive mutation followed by arbitrary finite scoped sharing.
/// The changed Vec model is transferred through B1 into the same physical read
/// and explicit cleanup protocol used by scoped_tree.
#[ensures(result.0 == (write_index@ < input@.len()))]
#[ensures((result.1 == None) == (leaves < 2usize))]
#[ensures(result.1 != None ==> result.1.unwrap_logic().0 ==
    if read_index@ < input@.len() {
        Some(if read_index == write_index { value } else { input@[read_index@] })
    } else { None })]
#[ensures(result.1 != None ==> result.1.unwrap_logic().1 == leaves)]
#[ensures(result.1 != None ==> result.1.unwrap_logic().2 != result.1.unwrap_logic().3)]
pub fn scoped_set_and_read(input:Vec<u8>,write_index:usize,value:u8,read_index:usize,leaves:usize)
    ->(bool,Option<(Option<u8>,usize,bool,bool)>)
{
    let mut exclusive=exclusive::ExclusiveBytes::from_vec(input);
    let changed=exclusive.set(write_index,value);
    let result=scoped_tree(exclusive.into_vec(),read_index,leaves);
    (changed,result)
}

#[cfg(all(test,not(creusot)))]
mod tests {
    use super::*;
    #[test]
    fn actual_scoped_readers_and_single_cleanup() {
        for len in [0,1,5] { for spare in [0,8] { for index in [0,1,4,8] { for reverse in [false,true] {
            let mut input=Vec::with_capacity(len+spare);
            input.extend((0..len).map(|i| (31+i) as u8));
            let expected=input.get(index).copied();
            let (a,b,x,y)=scoped_roundtrip(input.clone(),index,reverse);
            assert_eq!(a,expected);assert_eq!(b,expected);assert_ne!(x,y);
            let (a,b,x,y)=scoped_after_peer_close(input,index,reverse);
            assert_eq!(a,expected);assert_eq!(b,expected);assert_ne!(x,y);
        }}}}
    }
    #[test]
    fn exclusive_mutation_reaches_shared_physical_reads() {
        for leaves in [0,1,2,3,7] { for len in [0,1,5] {
            for write in [0,4,8] { for read in [0,4,8] {
                let mut input=Vec::with_capacity(len+8);
                input.extend((0..len).map(|i| (91+i) as u8));
                let mut expected=input.clone();
                let changed=write < expected.len();
                if changed { expected[write]=211; }
                let expected=expected.get(read).copied();
                let (actual_changed,result)=scoped_set_and_read(input,write,211,read,leaves);
                assert_eq!(actual_changed,changed);
                if leaves < 2 {assert!(result.is_none());}
                else {let (byte,count,a,b)=result.unwrap();assert_eq!(byte,expected);assert_eq!(count,leaves);assert_ne!(a,b);}
            }}
        }}
    }

}
