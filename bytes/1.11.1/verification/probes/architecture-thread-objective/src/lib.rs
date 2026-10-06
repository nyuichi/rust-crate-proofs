//! Architecture prerequisite: real scoped-thread transport of physical capabilities.
#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit="512"]
extern crate alloc;
use creusot_std::{prelude::*, ghost::{Perm, invariant::Tokens},
    std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::{Acquire,Relaxed,Release,None as NoStore}},
        committer::Committer, view::{AtView,HasTimestamp,SyncView}}};
#[path="../../weak-native-publication/src/primitive.rs"] mod primitive;
use primitive::NativeAtomic;
mod concurrent;
use creusot_std::std::thread::{self, JoinHandleExt};
use concurrent::{SharedRetirement,Payload};
use creusot_std::logic::Id;
#[cfg(not(creusot))]
#[path="../../../../src/allocation_ops.rs"] mod allocation_ops;
#[path="../../../../src/provenance_specs.rs"] mod provenance_specs;
#[path="../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[path="../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[path="../../../../src/ownership_proof/bound_ptr.rs"] mod bound_ptr;
use raw_vec::{BoundPtr,PhysicalRegion,Recovery};

struct RetiredPart { region:PhysicalRegion, recovery:Option<Recovery> }

/// This objective descriptor owns no physical resource and contains no capability.
#[derive(core::clone::Clone,Copy)]
struct PartMetadata { namespace:Id, capacity:Int, lo:Int, hi:Int, recovery:bool }

impl Payload for RetiredPart {
    type Metadata=PartMetadata;
    #[logic]
    fn metadata(self)->PartMetadata {
        PartMetadata { namespace:self.region.namespace(), capacity:self.region.capacity(),
            lo:self.region.lo(), hi:self.region.hi(), recovery:self.recovery != None }
    }
    #[logic(prophetic)]
    fn wellformed(self)->bool {
        pearlite! {
            self.region.invariant() && self.region.resource_id() == self.region.namespace() &&
            match self.recovery { None=>true, Some(recovery)=>
                recovery.invariant() && recovery.namespace() == self.region.namespace() &&
                recovery.capacity() == self.region.capacity() }
        }
    }
}

#[logic]
fn partition_metadata(base:BoundPtr,capacity:usize,parts:(PartMetadata,PartMetadata))->bool {
    pearlite! {
        base.invariant() && base@ == Some((parts.0.namespace,capacity@,0int)) &&
        parts.0.recovery && !parts.1.recovery &&
        parts.0.namespace == parts.1.namespace &&
        parts.0.capacity == capacity@ && parts.1.capacity == capacity@ &&
        parts.0.lo == 0 && parts.0.hi == parts.1.lo && parts.1.hi == capacity@
    }
}

/// Parent cleanup consumes real recovered capabilities, never metadata alone.
#[requires(parts.inner_logic().0.wellformed() && parts.inner_logic().1.wellformed())]
#[requires(partition_metadata(base,capacity,(parts.inner_logic().0.metadata(),parts.inner_logic().1.metadata())))]
fn reclaim_joined(base:BoundPtr,capacity:usize,parts:Ghost<(RetiredPart,RetiredPart)>) {
    let caps=ghost! {
        let (left,right)=parts.into_inner();
        (left.recovery.unwrap(),left.region.join(right.region))
    };
    unsafe { raw_vec::deallocate_bound_vec(base,capacity,caps); }
}

/// Pointer metadata remains in the parent; only affine physical capabilities cross.
pub fn thread_roundtrip(input:Vec<u8>,cut:usize,reverse:bool)->(bool,bool) {
    let (base,_len,capacity,caps)=bound_ptr::detach_bound_vec(input);
    let at=core::cmp::min(cut,capacity);
    let (left,right)=ghost! {
        let (recovery,region)=caps.into_inner();
        let (left,right)=region.split_at(*Int::new(at as i128));
        (RetiredPart{region:left,recovery:Some(recovery)},RetiredPart{region:right,recovery:None})
    }.split();
    let expected=snapshot!((left.inner_logic().metadata(),right.inner_logic().metadata()));
    let (machine,left_ticket,right_ticket)=SharedRetirement::new(expected);
    let machine=&machine;
    let ((first_ticket,first),(second_ticket,second))=if reverse {
        ((right_ticket,right),(left_ticket,left))
    } else {
        ((left_ticket,left),(right_ticket,right))
    };
    let (first,second)=thread::scope(move |scope| {
        let first=scope.spawn(move |tokens| machine.retire(first_ticket,first,tokens));
        let second=scope.spawn(move |tokens| machine.retire(second_ticket,second,tokens));
        (first.join_unwrap(),second.join_unwrap())
    });
    let first_last=first.0;
    let second_last=second.0;
    if first_last { reclaim_joined(base,capacity,ghost! {first.1.into_inner().unwrap()}); }
    if second_last { reclaim_joined(base,capacity,ghost! {second.1.into_inner().unwrap()}); }
    (first_last,second_last)
}

#[cfg(all(test,not(creusot)))]
mod tests {
    use super::*;
    #[test]
    fn physical_resources_cross_actual_threads() {
        let mut cases=0;
        for len in [0,1,5] {
            for capacity in [len,len+8] {
                for cut in 0..=capacity {
                    for reverse in [false,true] {
                        let mut input=Vec::with_capacity(capacity);
                        input.resize(len,31);
                        let (left,right)=thread_roundtrip(input,cut,reverse);
                        assert_ne!(left,right);
                        cases+=1;
                    }
                }
            }
        }
        eprintln!("threaded native cases: {cases}");
    }
}
