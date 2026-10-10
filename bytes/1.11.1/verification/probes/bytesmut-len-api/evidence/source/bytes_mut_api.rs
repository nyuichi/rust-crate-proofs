use alloc::vec::Vec;
use core::{ptr::NonNull, sync::atomic::AtomicUsize};
use creusot_std::prelude::*;
use crate::ownership_proof::raw_vec::{BoundPtr, RawAllocation, Recovery, PhysicalRegion, slot_known};

struct Shared {
    vec: Vec<u8>,
    original_capacity_repr: usize,
    ref_count: AtomicUsize,
}
pub struct BytesMut {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
    data: *mut Shared,
    // No authority is established for unselected constructors/representations.
    #[cfg(all(creusot, bytes_original_unique_gate))]
    unique_proof: Option<OriginalUniqueProof>,
}
#[cfg(all(creusot, bytes_original_unique_gate))]
struct OriginalUniqueProof {
    base: BoundPtr,
    raw: RawAllocation,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
}
#[cfg(all(creusot, bytes_original_unique_gate))]
impl BytesMut {
    #[logic(open(self))]
    pub(crate) fn unique_length(self) -> Int { pearlite! { self.len@ } }
    #[logic(open(self))]
    pub(crate) fn unique_capacity(self) -> Int { pearlite! { self.cap@ } }
    #[logic(open(self))]
    pub(crate) fn unique_pointer(self) -> *mut u8 { pearlite! { self.ptr@ } }
    #[logic(open(self))]
    pub(crate) fn unique_tag_pointer(self) -> *mut () { pearlite! { self.data as *mut () } }
    #[logic(open(self), prophetic)]
    pub(crate) fn unique_valid(self) -> bool {
        pearlite! {
            match self.unique_proof {
                None => false,
                Some(proof) => {
                    let caps = proof.capabilities.inner_logic();
                    proof.base.invariant() && proof.raw.invariant() &&
                    proof.raw.namespace() == caps.0.namespace() && proof.raw.capacity() == self.cap@ &&
                    proof.raw.raw_pointer() == proof.base.raw_pointer() && caps.0.invariant() && caps.1.invariant() &&
                    proof.base@ == Some((caps.0.namespace(), self.cap@, 0int)) &&
                    self.ptr@ == proof.base.raw_pointer() &&
                    caps.0.capacity() == self.cap@ && caps.1.capacity() == self.cap@ &&
                    caps.1.namespace() == caps.0.namespace() &&
                    caps.1.resource_id() == caps.0.namespace() &&
                    caps.1.lo() == 0 && caps.1.hi() == self.cap@ &&
                    self.len <= self.cap &&
                    self.data.addr_logic()@ < 32 && self.data.addr_logic()@ % 4 == 1 &&
                    (forall<i:Int> 0 <= i && i < self.len@ ==> slot_known(caps.1.slot(i)))
                }
            }
        }
    }
    #[logic(open(self))]
    pub(crate) fn unique_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! { self.unique_proof.unwrap_logic().capabilities.inner_logic().1.slot(index) }
    }
    #[logic(open(self))]
    pub(crate) fn unique_bytes(self) -> Seq<u8> {
        pearlite! { Seq::create(self.len@, |i:Int| self.unique_slot(i).unwrap_logic().unwrap_logic()) }
    }
}
#[cfg(all(creusot, bytes_original_unique_gate))]
#[check(ghost)]
#[bitwise_proof]
#[requires(repr <= 7usize)]
#[ensures(((repr << 2usize) | 1usize)@ < 32)]
#[ensures(((repr << 2usize) | 1usize)@ % 4 == 1)]
fn original_unique_metadata_bits(repr: usize) {}
#[cfg(all(creusot, bytes_original_unique_gate))]
#[check(ghost)]
#[bitwise_proof]
#[requires(addr@ < 32 && addr@ % 4 == 1)]
#[ensures(addr & 1usize == 1usize)]
#[ensures(addr >> 5usize == 0usize)]
fn original_unique_decode_bits(addr: usize) {}

impl BytesMut {
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == self.unique_length()))]
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_length() ==
        if len@ <= self.unique_length() { len@ } else { self.unique_length() }))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_pointer() == self.unique_pointer() &&
        (^self).unique_capacity() == self.unique_capacity() &&
        (^self).unique_tag_pointer() == self.unique_tag_pointer()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).unique_proof.unwrap_logic().raw.namespace() ==
            self.unique_proof.unwrap_logic().raw.namespace() &&
        (^self).unique_proof.unwrap_logic().capabilities.inner_logic().0.namespace() ==
            self.unique_proof.unwrap_logic().capabilities.inner_logic().0.namespace() &&
        (^self).unique_proof.unwrap_logic().capabilities.inner_logic().1.resource_id() ==
            self.unique_proof.unwrap_logic().capabilities.inner_logic().1.resource_id()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < self.unique_capacity() ==> (^self).unique_slot(i) == self.unique_slot(i)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_bytes() ==
        self.unique_bytes().subsequence(0,
            if len@ <= self.unique_length() { len@ } else { self.unique_length() })))]
    pub fn truncate(&mut self, len: usize) {
        if len <= self.len() {
            // SAFETY: Shrinking the buffer cannot expose uninitialized bytes.
            unsafe { self.set_len(len) };
        }
    }

    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(len@ <= self.unique_capacity()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(forall<i:Int>
        self.unique_length() <= i && i < len@ ==> slot_known(self.unique_slot(i))))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_length() == len@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_pointer() == self.unique_pointer() &&
        (^self).unique_capacity() == self.unique_capacity() &&
        (^self).unique_tag_pointer() == self.unique_tag_pointer()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).unique_proof.unwrap_logic().raw.namespace() ==
            self.unique_proof.unwrap_logic().raw.namespace() &&
        (^self).unique_proof.unwrap_logic().capabilities.inner_logic().0.namespace() ==
            self.unique_proof.unwrap_logic().capabilities.inner_logic().0.namespace() &&
        (^self).unique_proof.unwrap_logic().capabilities.inner_logic().1.resource_id() ==
            self.unique_proof.unwrap_logic().capabilities.inner_logic().1.resource_id()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < self.unique_capacity() ==> (^self).unique_slot(i) == self.unique_slot(i)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_bytes() ==
        Seq::create(len@, |i:Int| self.unique_slot(i).unwrap_logic().unwrap_logic())))]
    #[inline]
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.cap, "set_len out of bounds");
        self.len = len;
    }

}
