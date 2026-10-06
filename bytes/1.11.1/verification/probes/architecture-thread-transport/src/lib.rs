//! Architecture prerequisite: real scoped-thread transport of physical capabilities.
#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit="512"]
extern crate alloc;
use creusot_std::{prelude::*, ghost::{Perm, invariant::Tokens},
    std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::{Acquire,Relaxed,Release,None as NoStore}},
        committer::Committer, view::{AtView,HasTimestamp,SyncView}}};
#[path="../../weak-native-publication/src/primitive.rs"] mod primitive;
use primitive::NativeAtomic;
#[path="../../weak-physical-retirement/src/concurrent.rs"] mod concurrent;
use creusot_std::std::thread::{self, JoinHandleExt};
use concurrent::{SharedRetirement,Ticket};
#[cfg(not(creusot))]
#[path="../../../../src/allocation_ops.rs"] mod allocation_ops;
#[path="../../../../src/provenance_specs.rs"] mod provenance_specs;
#[path="../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[path="../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[path="../../../../src/ownership_proof/bound_ptr.rs"] mod bound_ptr;
use raw_vec::{BoundPtr,PhysicalRegion,Recovery};

struct RetiredPart { region:PhysicalRegion, recovery:Option<Recovery> }

#[logic(prophetic)]
fn full_partition(base:BoundPtr,capacity:usize,parts:(RetiredPart,RetiredPart))->bool {
    pearlite! {
        base.invariant() && base@ != None &&
        parts.0.recovery != None && parts.1.recovery == None &&
        parts.0.recovery.unwrap_logic().invariant() &&
        base@ == Some((parts.0.recovery.unwrap_logic().namespace(),capacity@,0int)) &&
        parts.0.recovery.unwrap_logic().capacity() == capacity@ &&
        parts.0.region.invariant() && parts.1.region.invariant() &&
        parts.0.region.namespace() == parts.0.recovery.unwrap_logic().namespace() &&
        parts.1.region.namespace() == parts.0.recovery.unwrap_logic().namespace() &&
        parts.0.region.resource_id() == parts.0.recovery.unwrap_logic().namespace() &&
        parts.1.region.resource_id() == parts.0.recovery.unwrap_logic().namespace() &&
        parts.0.region.capacity() == capacity@ && parts.1.region.capacity() == capacity@ &&
        parts.0.region.lo() == 0 && parts.0.region.hi() == parts.1.region.lo() && parts.1.region.hi() == capacity@
    }
}

/// Parent-side cleanup after the final observer returned the complete capabilities.
#[requires(full_partition(base,capacity,parts.inner_logic()))]
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
    let expected=snapshot!((left.inner_logic(),right.inner_logic()));
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
