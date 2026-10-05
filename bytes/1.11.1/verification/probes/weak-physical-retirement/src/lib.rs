//! Physical allocation retirement through a two-ticket weak-memory invariant.
#![allow(unexpected_cfgs, unused_variables, dead_code)]
#![recursion_limit="512"]
extern crate alloc;
use creusot_std::{prelude::*, ghost::{Perm, invariant::Tokens},
    std::sync::{atomic::{AtomicUsize as ModelAtomic, ordering::{Acquire,Relaxed,Release,None as NoStore}},
        committer::Committer, view::{AtView,HasTimestamp,SyncView}}};
#[path="../../weak-native-publication/src/primitive.rs"] mod primitive;
use primitive::NativeAtomic;
mod concurrent;
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

/// The only native cleanup branch. Both physical regions and the unique
/// recovery marker have been recovered from AtView after the Acquire fence.
#[requires(machine.valid() && machine.accepts_payload(ticket.inner_logic(),part.inner_logic()))]
#[requires(full_partition(base,capacity,machine.expected()))]
#[requires(tokens.contains(concurrent::PUBLICATION()))]
fn retire_and_reclaim(machine:&SharedRetirement<RetiredPart>,base:BoundPtr,capacity:usize,
    ticket:Ghost<Ticket>,part:Ghost<RetiredPart>,tokens:Ghost<Tokens>)->bool {
    let (last,returned)=machine.retire(ticket,part,tokens);
    if last {
        let caps=ghost! {
            let (left,right)=returned.into_inner().unwrap();
            (left.recovery.unwrap(),left.region.join(right.region))
        };
        unsafe { raw_vec::deallocate_bound_vec(base,capacity,caps); }
    }
    last
}

/// Exercise both retirement orders using actual B1 physical capabilities.
/// This sequential harness invokes a shared atomic-invariant operation; native
/// Bytes Send/Sync and heap Shared-cell destruction remain separate gates.
#[requires(tokens.contains(concurrent::PUBLICATION()))]
pub fn physical_roundtrip(input:Vec<u8>,cut:usize,reverse:bool,mut tokens:Ghost<Tokens>)->(bool,bool) {
    let (base,len,capacity,caps)=bound_ptr::detach_bound_vec(input);
    let at=core::cmp::min(cut,capacity);
    let (left,right)=ghost! {
        let (recovery,region)=caps.into_inner();
        let (left,right)=region.split_at(*Int::new(at as i128));
        (RetiredPart{region:left,recovery:Some(recovery)},RetiredPart{region:right,recovery:None})
    }.split();
    let expected=snapshot!((left.inner_logic(),right.inner_logic()));
    let (machine,left_ticket,right_ticket)=SharedRetirement::new(expected);
    if reverse {
        let first=retire_and_reclaim(&machine,base,capacity,right_ticket,right,ghost! {tokens.reborrow()});
        let second=retire_and_reclaim(&machine,base,capacity,left_ticket,left,tokens);
        (first,second)
    } else {
        let first=retire_and_reclaim(&machine,base,capacity,left_ticket,left,ghost! {tokens.reborrow()});
        let second=retire_and_reclaim(&machine,base,capacity,right_ticket,right,tokens);
        (first,second)
    }
}

#[cfg(all(test,not(creusot)))]
mod tests {
    use super::*;
    #[test]
    fn actual_regions_empty_spare_endpoints_and_both_orders() {
        for len in [0,1,5] {
            for capacity in [len,len+8] {
                for cut in 0..=capacity {
                    for reverse in [false,true] {
                        let mut input=Vec::with_capacity(capacity);
                        input.resize(len,31);
                        assert_eq!(physical_roundtrip(input,cut,reverse,ghost! {Tokens::new().into_inner()}),(false,true));
                    }
                }
            }
        }
    }
}
