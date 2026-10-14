#[cfg(all(creusot, bytes_original_unique_gate))]
use creusot_std::prelude::*;
#[cfg(all(creusot, bytes_original_unique_gate))]
use crate::ownership_proof::raw_vec::{self, BoundPtr, RawAllocation, Recovery, PhysicalRegion, slot_known};
#[cfg(all(creusot, bytes_original_unique_gate))]
use crate::ownership_proof::shared_context::BorrowedSharedContext;
use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};
use core::{cmp, fmt, hash, slice};

use alloc::{
    borrow::{Borrow, BorrowMut},
    boxed::Box,
    string::String,
    vec,
    vec::Vec,
};

use crate::buf::{IntoIter, UninitSlice};
use crate::bytes::Vtable;
#[cfg(all(creusot, bytes_original_unique_gate))]
use crate::ownership_proof::{
    boxed_alignment,
    ref_count_ledger::{AllocationBinding, OwnerLedger},
    terminal_phase::PhaseAuthority,
};
#[allow(unused)]
use crate::loom::sync::atomic::AtomicMut;
use crate::loom::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use crate::ref_count_ops::RefCountCell;
use crate::vec_storage::{VecRawParts, VecStorage};
use crate::{Buf, BufMut, Bytes, TryGetError};

#[cfg(all(creusot, bytes_original_unique_gate))]
#[path = "bytes_mut/shared_capsule.rs"]
mod shared_capsule;

// Creusot-only exclusions below cover unrelated Shared/refcount/recovery and adapter bodies.
// Native APIs remain present, and excluded implementations are not treated as proven contracts.

/// Drop-free field bundle used only while installing the two views produced by
/// a Vec-to-shared transfer. It carries pointer metadata and, in the proof
/// build, the matching affine owner packet; it never owns or drops the data.
#[allow(missing_debug_implementations)]
struct SharedSplitView {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
    data: *mut Shared,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    packet: shared_capsule::SharedOwnerPacket,
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl SharedSplitView {
    /// Packet facts needed to install this exact native view. The predicate is
    /// intentionally stated in terms of the descriptor's concrete fields,
    /// rather than assuming a `BytesMut` layout predicate at the call site.
    #[logic(open(self), prophetic)]
    fn packet_frame_valid(self) -> bool {
        pearlite! {
            self.packet.invariant() &&
            self.packet.capsule_public().0 == self.data as *mut () &&
            (self.data.addr_logic() & KIND_MASK) == KIND_ARC &&
            self.packet.base@ == Some((self.packet.region.inner_logic().namespace(),
                self.packet.region.inner_logic().capacity(), 0int)) &&
            self.packet.cursor@ != None &&
            self.packet.cursor.raw_pointer() == self.ptr@ &&
            self.packet.cursor@.unwrap_logic().0 ==
                self.packet.region.inner_logic().namespace() &&
            self.packet.cursor@.unwrap_logic().1 ==
                self.packet.region.inner_logic().capacity() &&
            self.packet.cursor.current_address() == self.packet.base.current_address() +
                self.packet.cursor@.unwrap_logic().2 &&
            self.len@ <= self.cap@ &&
            self.packet.cursor@.unwrap_logic().2 + self.cap@ ==
                self.packet.region.inner_logic().hi()
        }
    }

    #[logic(open(self), prophetic)]
    fn visible_prefix_known(self) -> bool {
        pearlite! {
            self.packet_frame_valid() &&
            forall<i:Int> 0 <= i && i < self.len@ ==>
                slot_known(self.packet.allocation_slot(
                    self.packet.cursor@.unwrap_logic().2 + i))
        }
    }
}

/// A unique reference to a contiguous slice of memory.
///
/// `BytesMut` represents a unique view into a potentially shared memory region.
/// Given the uniqueness guarantee, owners of `BytesMut` handles are able to
/// mutate the memory.
///
/// `BytesMut` can be thought of as containing a `buf: Arc<Vec<u8>>`, an offset
/// into `buf`, a slice length, and a guarantee that no other `BytesMut` for the
/// same `buf` overlaps with its slice. That guarantee means that a write lock
/// is not required.
///
/// # Growth
///
/// `BytesMut`'s `BufMut` implementation will implicitly grow its buffer as
/// necessary. However, explicitly reserving the required space up-front before
/// a series of inserts will be more efficient.
///
/// # Examples
///
/// ```
/// use bytes::{BytesMut, BufMut};
///
/// let mut buf = BytesMut::with_capacity(64);
///
/// buf.put_u8(b'h');
/// buf.put_u8(b'e');
/// buf.put(&b"llo"[..]);
///
/// assert_eq!(&buf[..], b"hello");
///
/// // Freeze the buffer so that it can be shared
/// let a = buf.freeze();
///
/// // This does not allocate, instead `b` points to the same memory.
/// let b = a.clone();
///
/// assert_eq!(&a[..], b"hello");
/// assert_eq!(&b[..], b"hello");
/// ```
pub struct BytesMut {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
    data: *mut Shared,
    // No authority is established for unselected constructors/representations.
    #[cfg(all(creusot, bytes_original_unique_gate))]
    unique_proof: Option<OriginalUniqueProof>,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    shared_proof: Option<shared_capsule::SharedOwnerPacket>,
}

#[cfg(all(creusot, bytes_original_unique_gate))]
struct OriginalUniqueProof {
    // Allocation origin stays fixed; cursor tracks the current Vec view.
    base: BoundPtr,
    cursor: BoundPtr,
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
    pub(crate) fn unique_offset(self) -> Int {
        pearlite! { self.unique_proof.unwrap_logic().cursor@.unwrap_logic().2 }
    }
    #[logic(open(self))]
    pub(crate) fn unique_allocation_capacity(self) -> Int {
        pearlite! { self.unique_proof.unwrap_logic().raw.capacity() }
    }
    #[logic(open(self))]
    pub(crate) fn unique_pointer(self) -> *mut u8 { pearlite! { self.ptr@ } }
    #[logic(open(self))]
    pub(crate) fn unique_tag_pointer(self) -> *mut () { pearlite! { self.data as *mut () } }
    /// The allocation, cursor, native pointer, packed tag, and affine
    /// capabilities agree. This deliberately omits the current length bound
    /// and initialized-prefix fact so the two private advance phases can be
    /// specified while `len` still describes the old view.
    #[logic(open(self), prophetic)]
    pub(crate) fn unique_layout_valid(self) -> bool {
        pearlite! {
            self.shared_proof == None && self.unique_proof != None && {
                let proof = self.unique_proof.unwrap_logic();
                let caps = proof.capabilities.inner_logic();
                proof.base.invariant() && proof.cursor.invariant() && proof.raw.invariant() &&
                proof.raw.namespace() == caps.0.namespace() &&
                proof.raw.raw_pointer() == proof.base.raw_pointer() &&
                caps.0.invariant() && caps.1.invariant() &&
                proof.base@ == Some((caps.0.namespace(), proof.raw.capacity(), 0int)) &&
                proof.cursor@ == Some((caps.0.namespace(), proof.raw.capacity(), self.unique_offset())) &&
                self.ptr@ == proof.cursor.raw_pointer() &&
                proof.cursor.current_address() == proof.base.current_address() + self.unique_offset() &&
                0 <= self.unique_offset() && self.unique_offset() <= proof.raw.capacity() &&
                self.unique_offset() + self.cap@ == proof.raw.capacity() &&
                caps.0.capacity() == proof.raw.capacity() &&
                caps.1.capacity() == proof.raw.capacity() &&
                caps.1.namespace() == caps.0.namespace() &&
                caps.1.resource_id() == caps.0.namespace() &&
                caps.1.lo() == 0 && caps.1.hi() == proof.raw.capacity() &&
                (self.data.addr_logic() & 3usize) == 1usize &&
                (self.data.addr_logic() >> VEC_POS_OFFSET)@ == self.unique_offset() &&
                (self.data.addr_logic() & KIND_MASK) == KIND_VEC
            }
        }
    }
    #[logic(open(self), prophetic)]
    pub(crate) fn unique_valid(self) -> bool {
        pearlite! {
            self.unique_layout_valid() && self.len <= self.cap &&
            (forall<i:Int> 0 <= i && i < self.len@ ==>
                slot_known(self.unique_slot(i)))
        }
    }
    /// The allocation prefix hidden before this view is initialized. This is
    /// independent of `len` so it remains available during the private layout
    /// phase, when the pointer has moved but the old length is still stored.
    #[logic(open(self), prophetic)]
    pub(crate) fn unique_hidden_prefix_known(self) -> bool {
        pearlite! {
            self.unique_layout_valid() &&
            (forall<i:Int> 0 <= i && i < self.unique_offset() ==>
                slot_known(self.unique_allocation_slot(i)))
        }
    }
    /// A stable Vec-backed view whose whole reconstructed Vec prefix is
    /// initialized. `rebuild_vec` sets its Vec length to `offset + len`, so
    /// visible-view validity alone is insufficient when `offset` is nonzero.
    #[logic(open(self), prophetic)]
    pub(crate) fn unique_vec_rebuildable(self) -> bool {
        pearlite! {
            self.unique_valid() && self.unique_hidden_prefix_known()
        }
    }
    /// Slot in allocation coordinates, including hidden prefixes and suffixes.
    #[logic(open(self))]
    pub(crate) fn unique_allocation_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! { self.unique_proof.unwrap_logic().capabilities.inner_logic().1.slot(index) }
    }
    /// Slot relative to this handle's current visible pointer.
    #[logic(open(self))]
    pub(crate) fn unique_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! { self.unique_allocation_slot(self.unique_offset() + index) }
    }
    #[logic(open(self))]
    pub(crate) fn unique_bytes(self) -> Seq<u8> {
        pearlite! { Seq::create(self.len@, |i:Int| self.unique_slot(i).unwrap_logic().unwrap_logic()) }
    }

    /// The handle's proof packet names the same header and ARC-tagged data
    /// pointer. Its invariant relates the ticket's allocation binding to the
    /// packet region and the capsule's protected storage. Selected proofs do
    /// not establish all production paths that create or preserve this state.
    #[logic(open(self), prophetic)]
    fn shared_packet_header_valid(self) -> bool {
        pearlite! {
            self.unique_proof == None &&
            match self.shared_proof {
                Some(packet) => packet.invariant() &&
                    packet.capsule_public().0 == self.data as *mut () &&
                    (self.data.addr_logic() & KIND_MASK) == KIND_ARC,
                None => false,
            }
        }
    }

    /// The packet's cursor is the handle pointer, and both bound descriptors
    /// retain one allocation namespace and base-relative offset.
    #[logic(open(self), prophetic)]
    fn shared_cursor_valid(self) -> bool {
        pearlite! {
            match self.shared_proof {
                Some(packet) =>
                    packet.base@ == Some((packet.region.inner_logic().namespace(),
                        packet.region.inner_logic().capacity(), 0int)) &&
                    packet.cursor@ != None &&
                    packet.cursor.raw_pointer() == self.ptr@ &&
                    packet.cursor@.unwrap_logic().0 == packet.region.inner_logic().namespace() &&
                    packet.cursor@.unwrap_logic().1 == packet.region.inner_logic().capacity() &&
                    packet.cursor.current_address() == packet.base.current_address() +
                        packet.cursor@.unwrap_logic().2,
                None => false,
            }
        }
    }

    /// The view fits its packet's physical suffix. This is independent of
    /// whether the current visible bytes are initialized.
    #[logic(open(self), prophetic)]
    fn shared_extent_valid(self) -> bool {
        pearlite! {
            match self.shared_proof {
                Some(packet) =>
                    self.len@ <= self.cap@ &&
                    packet.cursor@.unwrap_logic().2 + self.cap@ == packet.region.inner_logic().hi(),
                None => false,
            }
        }
    }

    /// Stable handle layout for the cursor-only shared path. The packet
    /// invariant links its region, base and capacity to the capsule's current
    /// storage binding through the live ledger ticket. This predicate does
    /// not establish that every public producer or later lifecycle transition
    /// creates and preserves the relation.
    #[logic(open(self), prophetic)]
    fn shared_layout_valid(self) -> bool {
        pearlite! {
            self.shared_packet_header_valid() &&
            self.shared_cursor_valid() &&
            self.shared_extent_valid()
        }
    }

    /// Stable shared physical layout needed for a length-only operation. This
    /// retains the native ARC tag and capsule/header pointer relation but
    /// deliberately excludes the capsule protocol, native refcount, and owner
    /// ledger: changing `len` does not inspect or transfer any of them. Like
    /// `unique_layout_valid`, it omits the current length bound and visible
    /// initialized-prefix fact so private layout transitions can be specified.
    #[logic(open(self), prophetic)]
    pub(crate) fn shared_view_layout_valid(self) -> bool {
        pearlite! {
            self.unique_proof == None &&
            match self.shared_proof {
                Some(packet) => {
                    let region = packet.region.inner_logic();
                    let offset = packet.cursor@.unwrap_logic().2;
                    region.invariant() &&
                    packet.base.invariant() && packet.cursor.invariant() &&
                    packet.base@ == Some((region.namespace(), region.capacity(), 0int)) &&
                    packet.cursor@ != None &&
                    packet.capsule_public().0 == self.data as *mut () &&
                    (self.data.addr_logic() & KIND_MASK) == KIND_ARC &&
                    packet.cursor.raw_pointer() == self.ptr@ &&
                    packet.cursor@.unwrap_logic().0 == region.namespace() &&
                    packet.cursor@.unwrap_logic().1 == region.capacity() &&
                    packet.cursor.current_address() == packet.base.current_address() + offset &&
                    region.lo() <= offset && offset <= region.hi() &&
                    offset + self.cap@ == region.hi()
                },
                None => false,
            }
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn shared_view_slots_known(self) -> bool {
        pearlite! { self.shared_slots_known_upto(self.len@) }
    }

    #[logic(open(self), prophetic)]
    fn shared_slots_known_upto(self, upto: Int) -> bool {
        pearlite! {
            forall<i:Int> 0 <= i && i < upto ==>
                slot_known(self.shared_proof.unwrap_logic().region.inner_logic().slot(
                    self.shared_proof.unwrap_logic().cursor@.unwrap_logic().2 + i))
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn shared_view_valid(self) -> bool {
        pearlite! {
            self.shared_view_layout_valid() && self.len@ <= self.cap@ &&
            self.shared_view_slots_known()
        }
    }

    #[logic(open(self), prophetic)]
    fn shared_visible_slots_known(self) -> bool {
        pearlite! {
            forall<i:Int> 0 <= i && i < self.len@ ==>
                slot_known(self.shared_proof.unwrap_logic().allocation_slot(
                    self.shared_proof.unwrap_logic().cursor@.unwrap_logic().2 + i))
        }
    }

    /// Only the bytes in the current shared view must be initialized. Hidden
    /// bytes before the cursor and spare capacity may be unknown.
    #[logic(open(self), prophetic)]
    fn shared_visible_prefix_known(self) -> bool {
        pearlite! {
            self.shared_layout_valid() &&
            self.shared_visible_slots_known()
        }
    }

    #[logic(open(self))]
    pub(crate) fn shared_bytes(self) -> Seq<u8> {
        pearlite! {
            Seq::create(self.len@, |i:Int|
                self.shared_proof.unwrap_logic().allocation_slot(
                    self.shared_proof.unwrap_logic().cursor@.unwrap_logic().2 + i
                ).unwrap_logic().unwrap_logic())
        }
    }

    /// The stable allocation/layout facts for either representation of a
    /// mutable view. This is a local representation predicate, not a claim
    /// that every native producer establishes it.
    #[logic(open(self), prophetic)]
    pub(crate) fn buffer_layout_valid(self) -> bool {
        pearlite! {
            self.unique_layout_valid() || self.shared_view_layout_valid()
        }
    }

    /// A live mutable view with an initialized visible prefix, in either the
    /// Vec-backed or packet-backed Shared representation.
    #[logic(open(self), prophetic)]
    pub(crate) fn buffer_view_valid(self) -> bool {
        pearlite! {
            self.buffer_layout_valid() && self.len@ <= self.cap@ &&
            self.buffer_prefix_known(self.len@)
        }
    }

    /// Slot in the current view, independent of its native representation.
    #[logic(open(self), prophetic)]
    pub(crate) fn buffer_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            if self.unique_proof != None {
                self.unique_slot(index)
            } else {
                self.shared_proof.unwrap_logic().region.inner_logic().slot(
                    self.shared_proof.unwrap_logic().cursor@.unwrap_logic().2 + index
                )
            }
        }
    }

    /// The initialized prefix shared by both native representations.
    #[logic(open(self), prophetic)]
    pub(crate) fn buffer_prefix_known(self, upto: Int) -> bool {
        pearlite! {
            forall<i:Int> 0 <= i && i < upto ==>
                slot_known(self.buffer_slot(i))
        }
    }

    /// Full allocation capacity represented by the current affine view.
    #[logic(open(self), prophetic)]
    pub(crate) fn buffer_allocation_capacity(self) -> Int {
        pearlite! {
            if self.unique_proof != None {
                self.unique_allocation_capacity()
            } else {
                self.shared_proof.unwrap_logic().region.inner_logic().capacity()
            }
        }
    }

    /// Slot in allocation coordinates, retaining hidden prefixes and spare
    /// capacity in the frame of a view-only operation.
    #[logic(open(self), prophetic)]
    pub(crate) fn buffer_allocation_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            if self.unique_proof != None {
                self.unique_allocation_slot(index)
            } else {
                self.shared_proof.unwrap_logic().region.inner_logic().slot(index)
            }
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn buffer_bytes(self) -> Seq<u8> {
        pearlite! {
            Seq::create(self.len@, |i:Int|
                self.buffer_slot(i).unwrap_logic().unwrap_logic())
        }
    }
}
/// Combine the currently initialized visible prefix with the range newly
/// exposed by `set_len`. Slot indices are absolute allocation coordinates.
#[cfg(all(creusot, bytes_original_unique_gate))]
#[check(ghost)]
#[requires((*prestate).unique_valid())]
#[requires(old_len@ == (*prestate).unique_length())]
#[requires(new_len@ <= (*prestate).unique_capacity())]
#[requires(forall<i:Int> old_len@ <= i && i < new_len@ ==>
    slot_known((*prestate).unique_slot(i)))]
#[ensures(forall<i:Int> 0 <= i && i < new_len@ ==>
    slot_known((*prestate).unique_slot(i)))]
fn unique_initialized_prefix_extend(
    prestate: Snapshot<BytesMut>, old_len: usize, new_len: usize,
) {
    proof_assert!(forall<i:Int> 0 <= i && i < new_len@ ==>
        if i < old_len@ {
            slot_known((*prestate).unique_slot(i))
        } else {
            slot_known((*prestate).unique_slot(i))
        }
    );
}

/// Combine the current initialized view with a range newly exposed by
/// `set_len`, without depending on whether the view is Vec-backed or Shared.
#[cfg(all(creusot, bytes_original_unique_gate))]
#[check(ghost)]
#[requires((*prestate).buffer_view_valid())]
#[requires(old_len@ == (*prestate).len@)]
#[requires(new_len@ <= (*prestate).cap@)]
#[requires(forall<i:Int> old_len@ <= i && i < new_len@ ==>
    slot_known((*prestate).buffer_slot(i)))]
#[ensures(forall<i:Int> 0 <= i && i < new_len@ ==>
    slot_known((*prestate).buffer_slot(i)))]
fn buffer_initialized_prefix_extend(
    prestate: Snapshot<BytesMut>, old_len: usize, new_len: usize,
) {
    proof_assert!(forall<i:Int> 0 <= i && i < new_len@ ==>
        if i < old_len@ {
            slot_known((*prestate).buffer_slot(i))
        } else {
            slot_known((*prestate).buffer_slot(i))
        }
    );
}

/// Convert the common visible-prefix observation to and from each native
/// representation's slot model. The snapshot borrow preserves the affine
/// packet and allocation authority while making the mode split explicit.
#[cfg(all(creusot, bytes_original_unique_gate))]
#[check(ghost)]
#[requires((*prestate).buffer_layout_valid())]
#[requires(upto@ <= (*prestate).cap@)]
#[ensures((*prestate).unique_layout_valid() ==>
    ((*prestate).buffer_prefix_known(upto@) ==
        (forall<i:Int> 0 <= i && i < upto@ ==>
            slot_known((*prestate).unique_slot(i)))))]
#[ensures((*prestate).shared_view_layout_valid() ==>
    ((*prestate).buffer_prefix_known(upto@) ==
        (*prestate).shared_slots_known_upto(upto@)))]
fn buffer_prefix_mode_equivalence(prestate: Snapshot<BytesMut>, upto: usize) {
    proof_assert!((*prestate).unique_layout_valid() ==>
        ((*prestate).buffer_prefix_known(upto@) ==
            (forall<i:Int> 0 <= i && i < upto@ ==>
                slot_known((*prestate).unique_slot(i)))));
    proof_assert!((*prestate).shared_view_layout_valid() ==>
        ((*prestate).buffer_prefix_known(upto@) ==
            (*prestate).shared_slots_known_upto(upto@)));
}

/// Reindex the initialized visible prefix after moving the current view forward.
/// The final handle exposes the old suffix beginning at `count`, so its slot at
/// `i` is the old handle's slot at `count + i`.
#[cfg(all(creusot, bytes_original_unique_gate))]
#[check(ghost)]
#[requires((*prestate).unique_valid())]
#[requires((*prestate).unique_vec_rebuildable())]
#[requires(count@ <= (*prestate).unique_capacity())]
#[requires(count@ <= (*prestate).unique_length())]
#[ensures(forall<i:Int>
    0 <= i && i < (if count@ <= (*prestate).unique_length() {
        (*prestate).unique_length() - count@
    } else { 0int }) ==>
        slot_known((*prestate).unique_slot(count@ + i)))]
#[ensures(forall<i:Int> 0 <= i && i < (*prestate).unique_offset() + count@ ==>
    slot_known((*prestate).unique_allocation_slot(i)))]
fn unique_initialized_prefix_suffix(prestate: Snapshot<BytesMut>, count: usize) {
    proof_assert!(forall<i:Int>
        0 <= i && i < (if count@ <= (*prestate).unique_length() {
            (*prestate).unique_length() - count@
        } else { 0int }) ==>
        0 <= count@ + i && count@ + i < (*prestate).unique_length()
    );
    proof_assert!(forall<i:Int>
        0 <= i && i < (if count@ <= (*prestate).unique_length() {
            (*prestate).unique_length() - count@
        } else { 0int }) ==>
            slot_known((*prestate).unique_slot(count@ + i))
    );
    proof_assert!(forall<i:Int>
        0 <= i && i < (*prestate).unique_offset() + count@ ==>
            if i < (*prestate).unique_offset() {
                slot_known((*prestate).unique_allocation_slot(i))
            } else {
                slot_known((*prestate).unique_slot(i - (*prestate).unique_offset()))
            }
    );
}


// Thread-safe reference-counted container for the shared storage. This mostly
// the same as `core::sync::Arc` but without the weak counter. The ref counting
// fns are based on the ones found in `std`.
//
// The main reason to use `Shared` instead of `core::sync::Arc` is that it ends
// up making the overall code simpler and easier to reason about. This is due to
// some of the logic around setting `Inner::arc` and other ways the `arc` field
// is used. Using `Arc` ended up requiring a number of funky transmutes and
// other shenanigans to make it work.
struct Shared {
    vec: VecStorage,
    original_capacity_repr: usize,
    ref_count: RefCountCell,
}

#[cfg(test)]
struct OriginalSharedLayout {
    vec: Vec<u8>,
    original_capacity_repr: usize,
    ref_count: AtomicUsize,
}

#[cfg(test)]
const _: [(); mem::size_of::<OriginalSharedLayout>()] = [(); mem::size_of::<Shared>()];
#[cfg(test)]
const _: [(); mem::align_of::<OriginalSharedLayout>()] = [(); mem::align_of::<Shared>()];

// Assert that the alignment of `Shared` is divisible by 2.
// This is a necessary invariant since we depend on allocating `Shared` a
// shared object to implicitly carry the `KIND_ARC` flag in its pointer.
// This flag is set when the LSB is 0.
const _: [(); 0 - mem::align_of::<Shared>() % 2] = []; // Assert that the alignment of `Shared` is divisible by 2.

// Buffer storage strategy flags.
const KIND_ARC: usize = 0b0;
const KIND_VEC: usize = 0b1;
const KIND_MASK: usize = 0b1;

// The max original capacity value. Any `Bytes` allocated with a greater initial
// capacity will default to this.
const MAX_ORIGINAL_CAPACITY_WIDTH: usize = 17;
// The original capacity algorithm will not take effect unless the originally
// allocated capacity was at least 1kb in size.
const MIN_ORIGINAL_CAPACITY_WIDTH: usize = 10;
// The original capacity is stored in powers of 2 starting at 1kb to a max of
// 64kb. Representing it as such requires only 3 bits of storage.
const ORIGINAL_CAPACITY_MASK: usize = 0b11100;
const ORIGINAL_CAPACITY_OFFSET: usize = 2;

const VEC_POS_OFFSET: usize = 5;
// When the storage is in the `Vec` representation, the pointer can be advanced
// at most this value. This is due to the amount of storage available to track
// the offset is usize - number of KIND bits and number of ORIGINAL_CAPACITY
// bits.
const MAX_VEC_POS: usize = usize::MAX >> VEC_POS_OFFSET;
const NOT_VEC_POS_MASK: usize = 0b11111;

#[cfg(target_pointer_width = "64")]
const PTR_WIDTH: usize = 64;
#[cfg(target_pointer_width = "32")]
const PTR_WIDTH: usize = 32;

/*
 *
 * ===== BytesMut =====
 *
 */

impl BytesMut {
    /// Creates a new `BytesMut` with the specified capacity.
    ///
    /// The returned `BytesMut` will be able to hold at least `capacity` bytes
    /// without reallocating.
    ///
    /// It is important to note that this function does not specify the length
    /// of the returned `BytesMut`, but only the capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::{BytesMut, BufMut};
    ///
    /// let mut bytes = BytesMut::with_capacity(64);
    ///
    /// // `bytes` contains no data, even though there is capacity
    /// assert_eq!(bytes.len(), 0);
    ///
    /// bytes.put(&b"hello world"[..]);
    ///
    /// assert_eq!(&bytes[..], b"hello world");
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_length() == 0))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_bytes() == Seq::empty()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < result.unique_capacity() ==> result.unique_slot(i) == Some(None)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(capacity@ <= result.unique_capacity()))]
    #[inline]
    pub fn with_capacity(capacity: usize) -> BytesMut {
        BytesMut::from_vec(Vec::with_capacity(capacity))
    }

    /// Creates a new `BytesMut` with default capacity.
    ///
    /// Resulting object has length 0 and unspecified capacity.
    /// This function does not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::{BytesMut, BufMut};
    ///
    /// let mut bytes = BytesMut::new();
    ///
    /// assert_eq!(0, bytes.len());
    ///
    /// bytes.reserve(2);
    /// bytes.put_slice(b"xy");
    ///
    /// assert_eq!(&b"xy"[..], &bytes[..]);
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_length() == 0))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_bytes() == Seq::empty()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < result.unique_capacity() ==> result.unique_slot(i) == Some(None)))]
    #[inline]
    pub fn new() -> BytesMut {
        BytesMut::with_capacity(0)
    }

    /// Returns the number of bytes contained in this `BytesMut`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let b = BytesMut::from(&b"hello"[..]);
    /// assert_eq!(b.len(), 5);
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == self.unique_length()))]
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the `BytesMut` has a length of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let b = BytesMut::with_capacity(64);
    /// assert!(b.is_empty());
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result == (self.unique_length() == 0)))]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of bytes the `BytesMut` can hold without reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let b = BytesMut::with_capacity(64);
    /// assert_eq!(b.capacity(), 64);
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == self.unique_capacity()))]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Converts `self` into an immutable `Bytes`.
    ///
    /// The conversion is zero cost and is used to indicate that the slice
    /// referenced by the handle will no longer be mutated. Once the conversion
    /// is done, the handle can be cloned and shared across threads.
    ///
    /// # Examples
    ///
    /// ```ignore-wasm
    /// use bytes::{BytesMut, BufMut};
    /// use std::thread;
    ///
    /// let mut b = BytesMut::with_capacity(64);
    /// b.put(&b"hello world"[..]);
    /// let b1 = b.freeze();
    /// let b2 = b1.clone();
    ///
    /// let th = thread::spawn(move || {
    ///     assert_eq!(&b1[..], b"hello world");
    /// });
    ///
    /// assert_eq!(&b2[..], b"hello world");
    /// th.join().unwrap();
    /// ```
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(self.unique_length() < self.unique_capacity()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result.original_frozen_valid()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result.original_frozen_bytes() == self.unique_bytes()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result.original_frozen_pointer() == self.unique_pointer() as *const u8))]
    #[inline]
    // Native freeze installs or transfers a Bytes vtable; its Shared ownership transition is
    // outside this direct BytesMut length-operation proof scope.
    #[cfg_attr(creusot, creusot::no_translate)]
    pub fn freeze(self) -> Bytes {
        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        let bytes = ManuallyDrop::new(self);
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        let mut bytes = self;
        if bytes.kind() == KIND_VEC {
            // Just re-use `Bytes` internal Vec vtable
            unsafe {
                let off = bytes.get_vec_pos();
                #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
                let vec = rebuild_vec(bytes.ptr.as_ptr(), bytes.len, bytes.cap, off);
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                let vec = original_rebuild_unique(bytes.ptr.as_ptr(), bytes.len, bytes.cap, off,
                    bytes.unique_proof.take().unwrap());
                #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
                let mut b: Bytes = vec.into();
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                let mut b: Bytes = Bytes::from(vec);
                b.advance(off);
                b
            }
        } else {
            debug_assert_eq!(bytes.kind(), KIND_ARC);

            #[cfg(all(creusot, bytes_original_freeze_gate))]
            { unreachable!("selected Vec/off0 path"); }
            #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
            {
                let ptr = bytes.ptr.as_ptr();
                let len = bytes.len;
                let data = AtomicPtr::new(bytes.data.cast());
                unsafe { Bytes::with_vtable(ptr, len, data, &SHARED_VTABLE) }
            }
        }
    }

    /// Creates a new `BytesMut` containing `len` zeros.
    ///
    /// The resulting object has a length of `len` and a capacity greater
    /// than or equal to `len`. The entire length of the object will be filled
    /// with zeros.
    ///
    /// On some platforms or allocators this function may be faster than
    /// a manual implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let zeros = BytesMut::zeroed(42);
    ///
    /// assert!(zeros.capacity() >= 42);
    /// assert_eq!(zeros.len(), 42);
    /// zeros.into_iter().for_each(|x| assert_eq!(x, 0));
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_length() == len@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_capacity() >= len@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_bytes() ==
        Seq::create(len@, |_i: Int| 0u8)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        len@ <= i && i < result.unique_capacity() ==> result.unique_slot(i) == Some(None)))]
    pub fn zeroed(len: usize) -> BytesMut {
        BytesMut::from_vec(vec![0; len])
    }

    /// Splits the bytes into two at the given index.
    ///
    /// Afterwards `self` contains elements `[0, at)`, and the returned
    /// `BytesMut` contains elements `[at, capacity)`. It's guaranteed that the
    /// memory does not move, that is, the address of `self` does not change,
    /// and the address of the returned slice is `at` bytes after that.
    ///
    /// This is an `O(1)` operation that just increases the reference count
    /// and sets a few indices.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut a = BytesMut::from(&b"hello world"[..]);
    /// let mut b = a.split_off(5);
    ///
    /// a[0] = b'j';
    /// b[0] = b'!';
    ///
    /// assert_eq!(&a[..], b"jello");
    /// assert_eq!(&b[..], b"!world");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `at > capacity`.
    #[must_use = "consider BytesMut::truncate if you don't need the other half"]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(context.inner_logic().invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^context.inner_logic()).invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        if (self.unique_tag_pointer().addr_logic() & KIND_MASK) == KIND_VEC {
            shared_capsule::header_context_registered_idle(
                context.inner_logic(), ^context.inner_logic(),
                result.shared_proof.unwrap_logic().lease_key()
            )
        } else {
            shared_capsule::header_context_unchanged(
                context.inner_logic(), ^context.inner_logic()
            )
        }
    ))]
    pub fn split_off(
        &mut self,
        at: usize,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        context: Ghost<&mut BorrowedSharedContext<'_, shared_capsule::HeaderLeaseState>>,
    ) -> BytesMut {
        assert!(
            at <= self.capacity(),
            "split_off out of bounds: {:?} <= {:?}",
            at,
            self.capacity(),
        );
        if self.kind() == KIND_VEC {
            // SAFETY: the bounds assertion above proves the split point is
            // within this Vec-backed view's capacity.
            return unsafe {
                self.split_unique_to_shared(
                    at,
                    false,
                    #[cfg(all(creusot, bytes_original_unique_gate))]
                    context,
                )
            };
        }
        unsafe {
            let mut other = self.shallow_clone();
            // SAFETY: We've checked that `at` <= `self.capacity()` above.
            other.advance_unchecked(
                at,
                #[cfg(all(creusot, bytes_original_unique_gate))]
                Some(context),
            );
            self.cap = at;
            self.len = cmp::min(self.len, at);
            other
        }
    }

    /// Removes the bytes from the current view, returning them in a new
    /// `BytesMut` handle.
    ///
    /// Afterwards, `self` will be empty, but will retain any additional
    /// capacity that it had before the operation. This is identical to
    /// `self.split_to(self.len())`.
    ///
    /// This is an `O(1)` operation that just increases the reference count and
    /// sets a few indices.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::{BytesMut, BufMut};
    ///
    /// let mut buf = BytesMut::with_capacity(1024);
    /// buf.put(&b"hello world"[..]);
    ///
    /// let other = buf.split();
    ///
    /// assert!(buf.is_empty());
    /// assert_eq!(1013, buf.capacity());
    ///
    /// assert_eq!(other, b"hello world"[..]);
    /// ```
    #[must_use = "consider BytesMut::clear if you don't need the other half"]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(context.inner_logic().invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^context.inner_logic()).invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        if (self.unique_tag_pointer().addr_logic() & KIND_MASK) == KIND_VEC {
            shared_capsule::header_context_registered_idle(
                context.inner_logic(), ^context.inner_logic(),
                result.shared_proof.unwrap_logic().lease_key()
            )
        } else {
            shared_capsule::header_context_unchanged(
                context.inner_logic(), ^context.inner_logic()
            )
        }
    ))]
    pub fn split(
        &mut self,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        context: Ghost<&mut BorrowedSharedContext<'_, shared_capsule::HeaderLeaseState>>,
    ) -> BytesMut {
        let len = self.len();
        self.split_to(
            len,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            Some(context),
        )
    }

    /// Splits the buffer into two at the given index.
    ///
    /// Afterwards `self` contains elements `[at, len)`, and the returned `BytesMut`
    /// contains elements `[0, at)`.
    ///
    /// This is an `O(1)` operation that just increases the reference count and
    /// sets a few indices.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut a = BytesMut::from(&b"hello world"[..]);
    /// let mut b = a.split_to(5);
    ///
    /// a[0] = b'!';
    /// b[0] = b'j';
    ///
    /// assert_eq!(&a[..], b"!world");
    /// assert_eq!(&b[..], b"jello");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    #[must_use = "consider BytesMut::advance if you don't need the other half"]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        context != None && context.unwrap_logic().inner_logic().invariant()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^context.unwrap_logic().inner_logic()).invariant()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        if (self.unique_tag_pointer().addr_logic() & KIND_MASK) == KIND_VEC {
            shared_capsule::header_context_registered_idle(
                context.unwrap_logic().inner_logic(),
                ^context.unwrap_logic().inner_logic(),
                result.shared_proof.unwrap_logic().lease_key()
            )
        } else {
            shared_capsule::header_context_unchanged(
                context.unwrap_logic().inner_logic(),
                ^context.unwrap_logic().inner_logic()
            )
        }
    ))]
    pub fn split_to(
        &mut self,
        at: usize,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        context: Option<Ghost<&mut BorrowedSharedContext<'_, shared_capsule::HeaderLeaseState>>>,
    ) -> BytesMut {
        assert!(
            at <= self.len(),
            "split_to out of bounds: {:?} <= {:?}",
            at,
            self.len(),
        );

        if self.kind() == KIND_VEC {
            // SAFETY: the bounds assertion above proves the split point is
            // within this Vec-backed view's initialized prefix.
            return unsafe {
                self.split_unique_to_shared(
                    at,
                    true,
                    #[cfg(all(creusot, bytes_original_unique_gate))]
                    context.unwrap(),
                )
            };
        }

        unsafe {
            let mut other = self.shallow_clone();
            // SAFETY: We've checked that `at` <= `self.len()` and we know that `self.len()` <=
            // `self.capacity()`.
            self.advance_unchecked(
                at,
                #[cfg(all(creusot, bytes_original_unique_gate))]
                context,
            );
            other.cap = at;
            other.len = at;
            other
        }
    }

    /// Install two already framed shared views without constructing an
    /// intermediate owning handle for `self`.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        self.unique_proof == None && self.shared_proof == None
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        left.visible_prefix_known() && right.visible_prefix_known()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).shared_layout_valid() && result.shared_layout_valid() &&
        (^self).shared_visible_prefix_known() && result.shared_visible_prefix_known() &&
        (^self).ptr@ == if return_left { right.ptr@ } else { left.ptr@ } &&
        result.ptr@ == if return_left { left.ptr@ } else { right.ptr@ } &&
        (^self).len@ == if return_left { right.len@ } else { left.len@ } &&
        result.len@ == if return_left { left.len@ } else { right.len@ } &&
        (^self).cap@ == if return_left { right.cap@ } else { left.cap@ } &&
        result.cap@ == if return_left { left.cap@ } else { right.cap@ } &&
        (^self).data == if return_left { right.data } else { left.data } &&
        result.data == if return_left { left.data } else { right.data } &&
        (^self).shared_proof.unwrap_logic() ==
            (if return_left { right.packet } else { left.packet }) &&
        result.shared_proof.unwrap_logic() ==
            (if return_left { left.packet } else { right.packet })
    ))]
    unsafe fn install_shared_split_views(
        &mut self,
        left: SharedSplitView,
        right: SharedSplitView,
        return_left: bool,
    ) -> BytesMut {
        let (self_view, other_view) = if return_left {
            (right, left)
        } else {
            (left, right)
        };

        self.ptr = self_view.ptr;
        self.len = self_view.len;
        self.cap = self_view.cap;
        self.data = self_view.data;

        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            self.unique_proof = None;
            self.shared_proof = Some(self_view.packet);
        }

        BytesMut {
            ptr: other_view.ptr,
            len: other_view.len,
            cap: other_view.cap,
            data: other_view.data,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            unique_proof: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            shared_proof: Some(other_view.packet),
        }
    }

    /// Split a unique Vec-backed view by transferring its original allocation
    /// directly into one shared header with two disjoint owners. This is the
    /// same runtime operation used by both original public split methods; the
    /// Creusot branch only moves the matching affine descriptors alongside
    /// the native allocation transfer.
    ///
    /// `return_left` chooses which half is returned. When false, `self` keeps
    /// the left range and the result is the right range (`split_off`); when
    /// true, `self` keeps the right range and the result is the left range
    /// (`split_to`).
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        self.unique_vec_rebuildable() &&
        at@ <= self.unique_capacity() &&
        (!return_left || at@ <= self.unique_length())
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).shared_layout_valid() && result.shared_layout_valid() &&
        (^self).shared_visible_prefix_known() && result.shared_visible_prefix_known() &&
        (^self).data == result.data &&
        (^self).shared_proof.unwrap_logic().capsule_public() ==
            result.shared_proof.unwrap_logic().capsule_public() &&
        (^self).shared_proof.unwrap_logic().ticket.inner_logic().owner_id() !=
            result.shared_proof.unwrap_logic().ticket.inner_logic().owner_id() &&
        (^self).shared_proof.unwrap_logic().ticket.inner_logic().ledger_id() ==
            result.shared_proof.unwrap_logic().ticket.inner_logic().ledger_id() &&
        (^self).shared_proof.unwrap_logic().region.inner_logic().namespace() ==
            self.unique_proof.unwrap_logic().raw.namespace() &&
        result.shared_proof.unwrap_logic().region.inner_logic().namespace() ==
            self.unique_proof.unwrap_logic().raw.namespace() &&
        (^self).shared_proof.unwrap_logic().region.inner_logic().capacity() ==
            self.unique_allocation_capacity() &&
        result.shared_proof.unwrap_logic().region.inner_logic().capacity() ==
            self.unique_allocation_capacity() &&
        (^self).shared_proof.unwrap_logic().base.raw_pointer() ==
            self.unique_proof.unwrap_logic().base.raw_pointer() &&
        result.shared_proof.unwrap_logic().base.raw_pointer() ==
            self.unique_proof.unwrap_logic().base.raw_pointer() &&
        (^self).len@ == if return_left {
            self.unique_length() - at@
        } else if at@ < self.unique_length() {
            at@
        } else {
            self.unique_length()
        } &&
        result.len@ == if return_left {
            if at@ < self.unique_length() { at@ } else { self.unique_length() }
        } else if at@ < self.unique_length() {
            self.unique_length() - at@
        } else {
            0int
        } &&
        (^self).cap@ == if return_left {
            self.unique_capacity() - at@
        } else {
            at@
        } &&
        result.cap@ == if return_left {
            at@
        } else {
            self.unique_capacity() - at@
        } &&
        (^self).ptr.addr_logic()@ == if return_left {
            self.unique_pointer().addr_logic()@ + at@
        } else {
            self.unique_pointer().addr_logic()@
        } &&
        result.ptr.addr_logic()@ == if return_left {
            self.unique_pointer().addr_logic()@
        } else {
            self.unique_pointer().addr_logic()@ + at@
        } &&
        (if return_left { result.ptr@ } else { (^self).ptr@ }) == self.ptr@ &&
        (if return_left {
            result.shared_bytes() == Seq::create(
                if at@ < self.unique_length() { at@ } else { self.unique_length() },
                |i: Int| self.unique_slot(i).unwrap_logic().unwrap_logic(),
            ) &&
            (^self).shared_bytes() == Seq::create(
                if at@ < self.unique_length() { self.unique_length() - at@ } else { 0int },
                |i: Int| self.unique_slot(at@ + i).unwrap_logic().unwrap_logic(),
            )
        } else {
            (^self).shared_bytes() == Seq::create(
                if at@ < self.unique_length() { at@ } else { self.unique_length() },
                |i: Int| self.unique_slot(i).unwrap_logic().unwrap_logic(),
            ) &&
            result.shared_bytes() == Seq::create(
                if at@ < self.unique_length() { self.unique_length() - at@ } else { 0int },
                |i: Int| self.unique_slot(at@ + i).unwrap_logic().unwrap_logic(),
            )
        }) &&
        (if return_left {
            result.shared_proof.unwrap_logic().region.inner_logic().lo() == 0 &&
            result.shared_proof.unwrap_logic().region.inner_logic().hi() ==
                self.unique_offset() + at@ &&
            (^self).shared_proof.unwrap_logic().region.inner_logic().lo() ==
                self.unique_offset() + at@ &&
            (^self).shared_proof.unwrap_logic().region.inner_logic().hi() ==
                self.unique_allocation_capacity()
        } else {
            (^self).shared_proof.unwrap_logic().region.inner_logic().lo() == 0 &&
            (^self).shared_proof.unwrap_logic().region.inner_logic().hi() ==
                self.unique_offset() + at@ &&
            result.shared_proof.unwrap_logic().region.inner_logic().lo() ==
                self.unique_offset() + at@ &&
            result.shared_proof.unwrap_logic().region.inner_logic().hi() ==
                self.unique_allocation_capacity()
        }) &&
        (forall<i: Int> 0 <= i && i < self.unique_allocation_capacity() ==>
            if i < self.unique_offset() + at@ {
                (if return_left {
                    result.shared_proof.unwrap_logic().allocation_slot(i)
                } else {
                    (^self).shared_proof.unwrap_logic().allocation_slot(i)
                }) == self.unique_allocation_slot(i)
            } else {
                (if return_left {
                    (^self).shared_proof.unwrap_logic().allocation_slot(i)
                } else {
                    result.shared_proof.unwrap_logic().allocation_slot(i)
                }) == self.unique_allocation_slot(i)
            }
    )))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(context.inner_logic().invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        shared_capsule::header_context_registered_idle(
            context.inner_logic(), ^context.inner_logic(),
            result.shared_proof.unwrap_logic().lease_key()
        )
    ))]
    unsafe fn split_unique_to_shared(
        &mut self,
        at: usize,
        return_left: bool,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        context: Ghost<&mut BorrowedSharedContext<'_, shared_capsule::HeaderLeaseState>>,
    ) -> BytesMut {
        debug_assert_eq!(self.kind(), KIND_VEC);

        let old_len = self.len;
        let old_cap = self.cap;
        let old_pointer = self.ptr.as_ptr();
        let off = unsafe { self.get_vec_pos() };
        let cut = off + at;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let total_len = off + old_len;
        let original_capacity_repr = crate::capacity_ops::original_capacity_repr_from_data(
            crate::provenance_specs::pointer_addr(self.data),
        );

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (vec, storage_region, base, allocation_binding, original_cursor) = {
            let prestate = snapshot!(*self);
            proof_assert!((*prestate).unique_vec_rebuildable());
            proof_assert!(at@ <= (*prestate).unique_capacity());
            proof_assert!(forall<i: Int> 0 <= i && i < total_len@ ==>
                if i < off@ {
                    slot_known((*prestate).unique_allocation_slot(i))
                } else {
                    slot_known((*prestate).unique_slot(i - off@))
                }
            );

            let proof = self.unique_proof.take().unwrap();
            let original_cursor = proof.cursor;
            let (binding_base, binding_capacity) = proof.raw.bound_ptr_at_zero();
            let binding_base_pointer = binding_base.as_ptr();
            let binding: Ghost<AllocationBinding> = ghost! {
                (
                    proof.raw.namespace_ghost().into_inner(),
                    binding_base_pointer,
                    raw_vec::int_from_usize(binding_capacity).into_inner(),
                )
            };
            let capabilities = ghost! {
                let (recovery, region) = proof.capabilities.into_inner();
                let total_len_int = raw_vec::int_from_usize(total_len).into_inner();
                proof_assert!(forall<i: Int> 0 <= i && i < total_len@ ==>
                    slot_known(region.slot(i)));
                region.prove_prefix_known(total_len_int);
                (recovery, region)
            };
            let (vec, authority, region, base) = unsafe {
                VecStorage::resume_owned(proof.raw, total_len, capabilities)
            };
            let storage_region = ghost! {
                let authority = authority.into_inner();
                let region = region.into_inner();
                let total_len_int = raw_vec::int_from_usize(total_len).into_inner();
                region.prove_prefix_known(total_len_int);
                (authority, region)
            };
            (vec, storage_region, base, binding, original_cursor)
        };

        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let vec = {
            let vec = unsafe { rebuild_vec(self.ptr.as_ptr(), old_len, old_cap, off) };
            VecStorage::hold(vec)
        };

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (left_cursor, right_cursor) = {
            let left_cursor = original_cursor;
            let right_cursor = original_cursor.advance_within(at);
            (left_cursor, right_cursor)
        };

        let left_pointer = old_pointer;
        let right_pointer = crate::provenance_specs::wrapping_offset(old_pointer, at);

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (ref_count, owner) = RefCountCell::new_with_owner(2);
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let ref_count = RefCountCell::new(2);

        let shared = Box::new(Shared {
            vec,
            original_capacity_repr,
            ref_count,
        });

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (header, root) = boxed_alignment::into_raw_aligned(shared);
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let header = Box::into_raw(shared);

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let source_region: Snapshot<PhysicalRegion> =
            snapshot!(storage_region.inner_logic().1);
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let known_end: Snapshot<Int> =
            snapshot!(storage_region.inner_logic().0.stored_len());
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (left_packet, right_packet) = {
            boxed_alignment::aligned_address_has_clear_low_bit(
                crate::provenance_specs::pointer_addr(header),
                core::mem::align_of::<Shared>(),
            );
            let ledger = OwnerLedger::new_promoted_pair(allocation_binding, snapshot!(cut@));
            let phase_action = PhaseAuthority::new();
            shared_capsule::make_count_two_packets(
                header,
                root,
                owner,
                storage_region,
                ledger,
                phase_action,
                base,
                left_cursor,
                right_cursor,
                snapshot!(cut@),
                context,
            )
        };

        debug_assert_eq!(
            crate::provenance_specs::pointer_addr(header) & KIND_MASK,
            KIND_ARC,
            "internal: Box<Shared> should have an aligned pointer",
        );

        let left_len = cmp::min(old_len, at);
        let right_len = old_len.saturating_sub(at);
        let right_cap = old_cap - at;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let _ = ghost! {
            let left_start = raw_vec::int_from_usize(off).into_inner();
            let left_count = raw_vec::int_from_usize(left_len).into_inner();
            let right_start = raw_vec::int_from_usize(cut).into_inner();
            let right_count = raw_vec::int_from_usize(right_len).into_inner();
            let known_end_int = known_end.into_ghost().into_inner();
            left_packet.region.borrow().into_inner().prove_subregion_prefix_known(
                source_region,
                left_start,
                left_count,
                known_end_int,
            );
            right_packet.region.borrow().into_inner().prove_subregion_prefix_known(
                source_region,
                right_start,
                right_count,
                known_end_int,
            );
        };
        let left_view = SharedSplitView {
            ptr: vptr(left_pointer),
            len: left_len,
            cap: at,
            data: header,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            packet: left_packet,
        };
        let right_view = SharedSplitView {
            ptr: vptr(right_pointer),
            len: right_len,
            cap: right_cap,
            data: header,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            packet: right_packet,
        };
        unsafe { self.install_shared_split_views(left_view, right_view, return_left) }
    }

    /// Shortens the buffer, keeping the first `len` bytes and dropping the
    /// rest.
    ///
    /// If `len` is greater than the buffer's current length, this has no
    /// effect.
    ///
    /// Existing underlying capacity is preserved.
    ///
    /// The [split_off](`Self::split_off()`) method can emulate `truncate`, but this causes the
    /// excess bytes to be returned instead of dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut buf = BytesMut::from(&b"hello world"[..]);
    /// buf.truncate(5);
    /// assert_eq!(buf, b"hello"[..]);
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.buffer_view_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).buffer_view_valid() &&
        (^self).len@ == if len@ <= self.len@ { len@ } else { self.len@ } &&
        (^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data &&
        (^self).unique_proof == self.unique_proof &&
        (^self).shared_proof == self.shared_proof &&
        (^self).buffer_allocation_capacity() == self.buffer_allocation_capacity() &&
        (forall<i:Int> 0 <= i && i < self.buffer_allocation_capacity() ==>
            (^self).buffer_allocation_slot(i) == self.buffer_allocation_slot(i)) &&
        (^self).buffer_bytes() == self.buffer_bytes().subsequence(0,
            if len@ <= self.len@ { len@ } else { self.len@ }) &&
        (self.unique_layout_valid() ==>
            ((^self).unique_valid() &&
             (^self).unique_pointer() == self.unique_pointer() &&
             (^self).unique_capacity() == self.unique_capacity() &&
             (^self).unique_offset() == self.unique_offset() &&
             (^self).unique_allocation_capacity() == self.unique_allocation_capacity() &&
             (^self).unique_tag_pointer() == self.unique_tag_pointer() &&
             (^self).unique_bytes() == self.unique_bytes().subsequence(0,
                if len@ <= self.unique_length() { len@ } else { self.unique_length() }) &&
             (forall<i:Int> 0 <= i && i < self.unique_allocation_capacity() ==>
                (^self).unique_allocation_slot(i) == self.unique_allocation_slot(i)))) &&
        (self.unique_vec_rebuildable() ==> (^self).unique_vec_rebuildable()) &&
        (self.shared_view_valid() ==> (^self).shared_view_valid())
    ))]
    pub fn truncate(&mut self, len: usize) {
        if len <= self.len() {
            // SAFETY: Shrinking the buffer cannot expose uninitialized bytes.
            unsafe { self.set_len(len) };
        }
    }

    /// Clears the buffer, removing all data. Existing capacity is preserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut buf = BytesMut::from(&b"hello world"[..]);
    /// buf.clear();
    /// assert!(buf.is_empty());
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.buffer_view_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).buffer_view_valid() && (^self).len@ == 0 &&
        (^self).buffer_bytes() == Seq::empty() &&
        (^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data &&
        (^self).unique_proof == self.unique_proof &&
        (^self).shared_proof == self.shared_proof &&
        (^self).buffer_allocation_capacity() == self.buffer_allocation_capacity() &&
        (forall<i:Int> 0 <= i && i < self.buffer_allocation_capacity() ==>
            (^self).buffer_allocation_slot(i) == self.buffer_allocation_slot(i)) &&
        (self.unique_valid() ==>
            ((^self).unique_valid() &&
             (^self).unique_length() == 0 &&
             (^self).unique_bytes() == Seq::empty() &&
             (^self).unique_pointer() == self.unique_pointer() &&
             (^self).unique_capacity() == self.unique_capacity() &&
             (^self).unique_offset() == self.unique_offset() &&
             (^self).unique_allocation_capacity() == self.unique_allocation_capacity() &&
             (^self).unique_tag_pointer() == self.unique_tag_pointer() &&
             (forall<i:Int> 0 <= i && i < self.unique_capacity() ==>
                (^self).unique_slot(i) == self.unique_slot(i)) &&
             (forall<i:Int> 0 <= i && i < self.unique_allocation_capacity() ==>
                (^self).unique_allocation_slot(i) == self.unique_allocation_slot(i)))) &&
        (self.unique_vec_rebuildable() ==> (^self).unique_vec_rebuildable()) &&
        (self.shared_view_valid() ==> (^self).shared_view_valid())
    ))]
    pub fn clear(&mut self) {
        // SAFETY: Setting the length to zero cannot expose uninitialized bytes.
        unsafe { self.set_len(0) };
    }

    /// Resizes the buffer so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the buffer is extended by the
    /// difference with each additional byte set to `value`. If `new_len` is
    /// less than `len`, the buffer is simply truncated.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut buf = BytesMut::new();
    ///
    /// buf.resize(3, 0x1);
    /// assert_eq!(&buf[..], &[0x1, 0x1, 0x1]);
    ///
    /// buf.resize(2, 0x2);
    /// assert_eq!(&buf[..], &[0x1, 0x1]);
    ///
    /// buf.resize(4, 0x3);
    /// assert_eq!(&buf[..], &[0x1, 0x1, 0x3, 0x3]);
    /// ```
    pub fn resize(&mut self, new_len: usize, value: u8) {
        let additional = if let Some(additional) = new_len.checked_sub(self.len()) {
            additional
        } else {
            self.truncate(new_len);
            return;
        };

        if additional == 0 {
            return;
        }

        self.reserve(additional);
        let dst = self.spare_capacity_mut().as_mut_ptr();
        // SAFETY: `spare_capacity_mut` returns a valid, properly aligned pointer and we've
        // reserved enough space to write `additional` bytes.
        unsafe { ptr::write_bytes(dst, value, additional) };

        // SAFETY: There are at least `new_len` initialized bytes in the buffer so no
        // uninitialized bytes are being exposed.
        unsafe { self.set_len(new_len) };
    }

    /// Complete a length-only transition after the new visible prefix has
    /// already been shown initialized. This also restores `unique_valid` after
    /// a private cursor/capacity transition temporarily left the old length in
    /// place.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        self.buffer_layout_valid() && new_len@ <= self.cap@ &&
        (forall<i:Int> 0 <= i && i < new_len@ ==>
            slot_known(self.buffer_slot(i)))
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).buffer_view_valid() && (^self).len@ == new_len@ &&
        (^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data &&
        (^self).unique_proof == self.unique_proof &&
        (^self).shared_proof == self.shared_proof &&
        (^self).buffer_allocation_capacity() == self.buffer_allocation_capacity() &&
        (forall<i:Int> 0 <= i && i < self.buffer_allocation_capacity() ==>
            (^self).buffer_allocation_slot(i) == self.buffer_allocation_slot(i)) &&
        (^self).buffer_bytes() == Seq::create(new_len@, |i:Int|
            self.buffer_slot(i).unwrap_logic().unwrap_logic()) &&
        (self.unique_layout_valid() ==>
            ((^self).unique_valid() &&
             (^self).unique_pointer() == self.unique_pointer() &&
             (^self).unique_capacity() == self.unique_capacity() &&
             (^self).unique_offset() == self.unique_offset() &&
             (^self).unique_allocation_capacity() == self.unique_allocation_capacity() &&
             (^self).unique_tag_pointer() == self.unique_tag_pointer() &&
             (^self).unique_proof == self.unique_proof &&
             (forall<i:Int> 0 <= i && i < self.unique_allocation_capacity() ==>
                (^self).unique_allocation_slot(i) == self.unique_allocation_slot(i)))) &&
        (self.unique_hidden_prefix_known() ==> (^self).unique_vec_rebuildable()) &&
        (self.unique_hidden_prefix_known() ==> (^self).unique_hidden_prefix_known()) &&
        (self.shared_view_layout_valid() ==> (^self).shared_view_valid())
    ))]
    #[inline]
    fn set_visible_len(&mut self, new_len: usize) {
        self.len = new_len;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let _ = ghost! {
            buffer_prefix_mode_equivalence(snapshot!(*self), new_len);
        };
    }

    /// Advance a Vec-backed view without changing its visible length. This is
    /// the layout phase; `set_visible_len` follows and restores the full
    /// length/prefix invariant. The helper only runs on the non-promotion path.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(count@ <= self.unique_capacity()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(count@ <= self.unique_length()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(pos@ == self.unique_offset() + count@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(pos@ <= MAX_VEC_POS@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_layout_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_hidden_prefix_known()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_length() == self.unique_length()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_capacity() == self.unique_capacity() - count@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_offset() == self.unique_offset() + count@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_allocation_capacity() == self.unique_allocation_capacity()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_pointer().addr_logic()@ == self.unique_pointer().addr_logic()@ + count@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(((^self).unique_tag_pointer().addr_logic() >> VEC_POS_OFFSET)@ == pos@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(((^self).unique_tag_pointer().addr_logic() & KIND_MASK) == KIND_VEC))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        ((^self).unique_tag_pointer().addr_logic() & NOT_VEC_POS_MASK) ==
            (self.unique_tag_pointer().addr_logic() & NOT_VEC_POS_MASK)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_proof.unwrap_logic().base == self.unique_proof.unwrap_logic().base))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_proof.unwrap_logic().raw == self.unique_proof.unwrap_logic().raw))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_proof.unwrap_logic().capabilities == self.unique_proof.unwrap_logic().capabilities))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < self.unique_allocation_capacity() ==>
            (^self).unique_allocation_slot(i) == self.unique_allocation_slot(i)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < (^self).unique_capacity() ==>
            (^self).unique_slot(i) == self.unique_slot(count@ + i)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < (if count@ <= self.unique_length() {
            self.unique_length() - count@
        } else { 0int }) ==>
            slot_known((^self).unique_slot(i))))]
    #[inline]
    unsafe fn advance_vec_layout(&mut self, count: usize, pos: usize) {
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let prestate = snapshot!(*self);
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let _ = ghost! {
            unique_initialized_prefix_suffix(prestate, count);
        };

        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            let cursor = self.unique_proof.as_ref().unwrap().cursor.advance_within(count);
            let pointer = cursor.as_non_null();
            self.set_vec_pos(pos);
            self.ptr = pointer;
            self.unique_proof.as_mut().unwrap().cursor = cursor;
        }
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        {
            self.set_vec_pos(pos);
            self.ptr = vptr(crate::provenance_specs::wrapping_offset(
                self.ptr.as_ptr(), count,
            ));
        }

        // The old length is preserved until the next private phase, so it may
        // temporarily exceed this reduced capacity; no public observer can run
        // between these non-panicking updates under the caller's `&mut` borrow.
        self.cap -= count;
    }

    /// Move a shared handle's view within its owned physical subregion. The
    /// packet's region, ticket, capsule, and allocation slots are unchanged;
    /// only the cursor and the native view fields advance.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.shared_layout_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.shared_visible_prefix_known()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(count@ <= self.cap@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_packet_header_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_cursor_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_extent_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_visible_slots_known()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_proof == self.unique_proof))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).data == self.data))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).len@ ==
        (if count@ <= self.len@ { self.len@ - count@ } else { 0int })))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).cap@ + count@ == self.cap@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).ptr.addr_logic()@ == self.ptr.addr_logic()@ + count@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).shared_proof.unwrap_logic().capsule.inner_logic() ==
            self.shared_proof.unwrap_logic().capsule.inner_logic()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).shared_proof.unwrap_logic().ticket.inner_logic() ==
            self.shared_proof.unwrap_logic().ticket.inner_logic()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).shared_proof.unwrap_logic().region.inner_logic() ==
            self.shared_proof.unwrap_logic().region.inner_logic()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).shared_proof.unwrap_logic().base == self.shared_proof.unwrap_logic().base))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).shared_proof.unwrap_logic().cursor@.unwrap_logic().2 ==
            self.shared_proof.unwrap_logic().cursor@.unwrap_logic().2 + count@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < self.shared_proof.unwrap_logic().region.inner_logic().capacity() ==>
            (^self).shared_proof.unwrap_logic().allocation_slot(i) ==
                self.shared_proof.unwrap_logic().allocation_slot(i)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_bytes() ==
        self.shared_bytes().subsequence(
            if count@ <= self.len@ { count@ } else { self.len@ }, self.len@)))]
    #[inline]
    unsafe fn advance_shared_layout(&mut self, count: usize) {
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let next_packet = self.shared_proof.take().unwrap().advance_cursor(count);

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let pointer = next_packet.cursor.as_ptr();
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let pointer = crate::provenance_specs::wrapping_offset(self.ptr.as_ptr(), count);

        self.ptr = vptr(pointer);
        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            self.shared_proof = Some(next_packet);
        }
        self.len = self.len.saturating_sub(count);
        self.cap -= count;
    }

    /// Sets the length of the buffer.
    ///
    /// This will explicitly set the size of the buffer without actually
    /// modifying the data, so it is up to the caller to ensure that the data
    /// has been initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut b = BytesMut::from(&b"hello world"[..]);
    ///
    /// unsafe {
    ///     b.set_len(5);
    /// }
    ///
    /// assert_eq!(&b[..], b"hello");
    ///
    /// unsafe {
    ///     b.set_len(11);
    /// }
    ///
    /// assert_eq!(&b[..], b"hello world");
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        self.buffer_view_valid() && len@ <= self.cap@ &&
        (forall<i:Int> self.len@ <= i && i < len@ ==>
            slot_known(self.buffer_slot(i)))
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^self).buffer_view_valid() && (^self).len@ == len@ &&
        (^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data &&
        (^self).unique_proof == self.unique_proof &&
        (^self).shared_proof == self.shared_proof &&
        (^self).buffer_allocation_capacity() == self.buffer_allocation_capacity() &&
        (forall<i:Int> 0 <= i && i < self.buffer_allocation_capacity() ==>
            (^self).buffer_allocation_slot(i) == self.buffer_allocation_slot(i)) &&
        (^self).buffer_bytes() == Seq::create(len@, |i:Int|
            self.buffer_slot(i).unwrap_logic().unwrap_logic()) &&
        (self.unique_valid() ==>
            ((^self).unique_valid() &&
             (^self).unique_pointer() == self.unique_pointer() &&
             (^self).unique_capacity() == self.unique_capacity() &&
             (^self).unique_offset() == self.unique_offset() &&
             (^self).unique_allocation_capacity() == self.unique_allocation_capacity() &&
             (^self).unique_tag_pointer() == self.unique_tag_pointer() &&
             (^self).unique_proof == self.unique_proof &&
             (forall<i:Int> 0 <= i && i < self.unique_allocation_capacity() ==>
                (^self).unique_allocation_slot(i) == self.unique_allocation_slot(i)))) &&
        (self.unique_vec_rebuildable() ==> (^self).unique_vec_rebuildable()) &&
        (self.shared_view_valid() ==> (^self).shared_view_valid())
    ))]
    #[inline]
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.cap, "set_len out of bounds");
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let _ = ghost! {
            buffer_initialized_prefix_extend(snapshot!(*self), self.len, len);
        };
        self.set_visible_len(len);
    }

    /// Reserves capacity for at least `additional` more bytes to be inserted
    /// into the given `BytesMut`.
    ///
    /// More than `additional` bytes may be reserved in order to avoid frequent
    /// reallocations. A call to `reserve` may result in an allocation.
    ///
    /// Before allocating new buffer space, the function will attempt to reclaim
    /// space in the existing buffer. If the current handle references a view
    /// into a larger original buffer, and all other handles referencing part
    /// of the same original buffer have been dropped, then the current view
    /// can be copied/shifted to the front of the buffer and the handle can take
    /// ownership of the full buffer, provided that the full buffer is large
    /// enough to fit the requested additional capacity.
    ///
    /// This optimization will only happen if shifting the data from the current
    /// view to the front of the buffer is not too expensive in terms of the
    /// (amortized) time required. The precise condition is subject to change;
    /// as of now, the length of the data being shifted needs to be at least as
    /// large as the distance that it's shifted by. If the current view is empty
    /// and the original buffer is large enough to fit the requested additional
    /// capacity, then reallocations will never happen.
    ///
    /// # Examples
    ///
    /// In the following example, a new buffer is allocated.
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut buf = BytesMut::from(&b"hello"[..]);
    /// buf.reserve(64);
    /// assert!(buf.capacity() >= 69);
    /// ```
    ///
    /// In the following example, the existing buffer is reclaimed.
    ///
    /// ```
    /// use bytes::{BytesMut, BufMut};
    ///
    /// let mut buf = BytesMut::with_capacity(128);
    /// buf.put(&[0; 64][..]);
    ///
    /// let ptr = buf.as_ptr();
    /// let other = buf.split();
    ///
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.capacity(), 64);
    ///
    /// drop(other);
    /// buf.reserve(128);
    ///
    /// assert_eq!(buf.capacity(), 128);
    /// assert_eq!(buf.as_ptr(), ptr);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(additional@ <= self.unique_capacity() - self.unique_length()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(^self == *self))]
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let len = self.len();
        let rem = self.capacity() - len;

        if additional <= rem {
            // The handle can already store at least `additional` more bytes, so
            // there is no further work needed to be done.
            return;
        }

        // will always succeed
        let _ = self.reserve_inner(additional, true);
    }

    // In separate function to allow the short-circuits in `reserve` and `try_reclaim` to
    // be inline-able. Significantly helps performance. Returns false if it did not succeed.
    #[cfg_attr(creusot, creusot::no_translate)]
    fn reserve_inner(&mut self, additional: usize, allocate: bool) -> bool {
        let len = self.len();
        let kind = self.kind();

        if kind == KIND_VEC {
            // If there's enough free space before the start of the buffer, then
            // just copy the data backwards and reuse the already-allocated
            // space.
            //
            // Otherwise, since backed by a vector, use `Vec::reserve`
            //
            // We need to make sure that this optimization does not kill the
            // amortized runtimes of BytesMut's operations.
            unsafe {
                let off = self.get_vec_pos();

                // Only reuse space if we can satisfy the requested additional space.
                //
                // Also check if the value of `off` suggests that enough bytes
                // have been read to account for the overhead of shifting all
                // the data (in an amortized analysis).
                // Hence the condition `off >= self.len()`.
                //
                // This condition also already implies that the buffer is going
                // to be (at least) half-empty in the end; so we do not break
                // the (amortized) runtime with future resizes of the underlying
                // `Vec`.
                //
                // [For more details check issue #524, and PR #525.]
                if self.capacity() - self.len() + off >= additional && off >= self.len() {
                    // There's enough space, and it's not too much overhead:
                    // reuse the space!
                    //
                    // Just move the pointer back to the start after copying
                    // data back.
                    let base_ptr = self.ptr.as_ptr().sub(off);
                    // Since `off >= self.len()`, the two regions don't overlap.
                    ptr::copy_nonoverlapping(self.ptr.as_ptr(), base_ptr, self.len);
                    self.ptr = vptr(base_ptr);
                    self.set_vec_pos(0);

                    // Length stays constant, but since we moved backwards we
                    // can gain capacity back.
                    self.cap += off;
                } else {
                    if !allocate {
                        return false;
                    }
                    // Not enough space, or reusing might be too much overhead:
                    // allocate more space!
                    let mut v =
                        ManuallyDrop::new(rebuild_vec(self.ptr.as_ptr(), self.len, self.cap, off));
                    v.reserve(additional);

                    // Update the info
                    self.ptr = vptr(v.as_mut_ptr().add(off));
                    self.cap = v.capacity() - off;
                    debug_assert_eq!(self.len, v.len() - off);
                }

                return true;
            }
        }

        debug_assert_eq!(kind, KIND_ARC);
        let shared: *mut Shared = self.data;

        // Reserving involves abandoning the currently shared buffer and
        // allocating a new vector with the requested capacity.
        //
        // Compute the new capacity
        let mut new_cap = match len.checked_add(additional) {
            Some(new_cap) => new_cap,
            None if !allocate => return false,
            None => panic!("overflow"),
        };

        unsafe {
            // First, try to reclaim the buffer. This is possible if the current
            // handle is the only outstanding handle pointing to the buffer.
            if (*shared).is_unique() {
                // This is the only handle to the buffer. It can be reclaimed.
                // However, before doing the work of copying data, check to make
                // sure that the vector has enough capacity.
                let v = &mut (*shared).vec;

                let (ptr, v_len, v_capacity) = v.metadata();

                let offset = self.ptr.as_ptr().offset_from(ptr) as usize;

                let new_cap_plus_offset = match new_cap.checked_add(offset) {
                    Some(new_cap_plus_offset) => new_cap_plus_offset,
                    None if !allocate => return false,
                    None => panic!("overflow"),
                };

                // Compare the condition in the `kind == KIND_VEC` case above
                // for more details.
                if v_capacity >= new_cap_plus_offset {
                    self.cap = new_cap;
                    // no copy is necessary
                    return true;
                } else if v_capacity >= new_cap && offset >= len {
                    // The capacity is sufficient, and copying is not too much
                    // overhead: reclaim the buffer!

                    // `offset >= len` means: no overlap
                    ptr::copy_nonoverlapping(self.ptr.as_ptr(), ptr, len);

                    self.ptr = vptr(ptr);
                    self.cap = v.metadata().2;
                    return true;
                } else if v_len < offset {
                    // The shared Vec does not initialize the gap between its
                    // current prefix and this view. In particular, `split_off`
                    // may start a zero-length right view beyond the Vec's
                    // length. Let the copy-to-new-Vec path below handle this
                    // case instead of extending the Vec across uninitialized
                    // bytes.
                } else {
                    if !allocate {
                        return false;
                    }

                    // `new_cap` is calculated in terms of `BytesMut`, not the
                    // underlying `Vec`, so it does not take the offset into
                    // account.
                    //
                    // Thus we have to manually add it here.
                    new_cap = new_cap_plus_offset;

                    // The vector capacity is not sufficient. The reserve request is
                    // asking for more than the initial buffer capacity. Allocate more
                    // than requested if `new_cap` is not much bigger than the current
                    // capacity.
                    //
                    // There are some situations, using `reserve_exact` that the
                    // buffer capacity could be below `original_capacity`, so do a
                    // check.
                    let double = v_capacity.checked_shl(1).unwrap_or(new_cap);

                    new_cap = cmp::max(double, new_cap);

                    // No space - allocate more
                    //
                    // The Shared Vec may contain an uninitialized hidden prefix:
                    // another disjoint handle can safely overwrite that prefix
                    // through its spare capacity before being dropped. Compact
                    // only this handle's initialized visible bytes to the base
                    // before growing, so Vec::reserve receives an initialized
                    // element prefix; any spare capacity remains uninitialized.
                    debug_assert!(offset + len <= v.metadata().2);
                    ptr::copy(self.ptr.as_ptr(), ptr, len);
                    v.set_len(len);
                    v.reserve(new_cap - len);

                    // Update the info
                    let (base, _, vec_cap) = v.metadata();
                    self.ptr = vptr(base);
                    self.cap = vec_cap;
                    return true;
                }
            }
        }
        if !allocate {
            return false;
        }

        let original_capacity_repr = unsafe { (*shared).original_capacity_repr };
        let original_capacity = original_capacity_from_repr(original_capacity_repr);

        new_cap = cmp::max(new_cap, original_capacity);

        // Create a new vector to store the data
        let mut v = ManuallyDrop::new(Vec::with_capacity(new_cap));

        // Copy the bytes
        v.extend_from_slice(self.as_ref());

        // Release the shared handle. This must be done *after* the bytes are
        // copied.
        unsafe { release_shared(shared) };

        // Update self
        let data = (original_capacity_repr << ORIGINAL_CAPACITY_OFFSET) | KIND_VEC;

        self.data = invalid_ptr(data);
        self.ptr = vptr(v.as_mut_ptr());
        self.cap = v.capacity();
        debug_assert_eq!(self.len, v.len());
        true
    }

    /// Attempts to cheaply reclaim already allocated capacity for at least `additional` more
    /// bytes to be inserted into the given `BytesMut` and returns `true` if it succeeded.
    ///
    /// `try_reclaim` behaves exactly like `reserve`, except that it never allocates new storage
    /// and returns a `bool` indicating whether it was successful in doing so:
    ///
    /// `try_reclaim` returns false under these conditions:
    ///  - The spare capacity left is less than `additional` bytes AND
    ///  - The existing allocation cannot be reclaimed cheaply or it was less than
    ///    `additional` bytes in size
    ///
    /// Reclaiming the allocation cheaply is possible if the `BytesMut` has no outstanding
    /// references through other `BytesMut`s or `Bytes` which point to the same underlying
    /// storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut buf = BytesMut::with_capacity(64);
    /// assert_eq!(true, buf.try_reclaim(64));
    /// assert_eq!(64, buf.capacity());
    ///
    /// buf.extend_from_slice(b"abcd");
    /// let mut split = buf.split();
    /// assert_eq!(60, buf.capacity());
    /// assert_eq!(4, split.capacity());
    /// assert_eq!(false, split.try_reclaim(64));
    /// assert_eq!(false, buf.try_reclaim(64));
    /// // The split buffer is filled with "abcd"
    /// assert_eq!(false, split.try_reclaim(4));
    /// // buf is empty and has capacity for 60 bytes
    /// assert_eq!(true, buf.try_reclaim(60));
    ///
    /// drop(buf);
    /// assert_eq!(false, split.try_reclaim(64));
    ///
    /// split.clear();
    /// assert_eq!(4, split.capacity());
    /// assert_eq!(true, split.try_reclaim(64));
    /// assert_eq!(64, split.capacity());
    /// ```
    // I tried splitting out try_reclaim_inner after the short circuits, but it was inlined
    // regardless with Rust 1.78.0 so probably not worth it
    #[inline]
    #[must_use = "consider BytesMut::reserve if you need an infallible reservation"]
    pub fn try_reclaim(&mut self, additional: usize) -> bool {
        let len = self.len();
        let rem = self.capacity() - len;

        if additional <= rem {
            // The handle can already store at least `additional` more bytes, so
            // there is no further work needed to be done.
            return true;
        }

        self.reserve_inner(additional, false)
    }

    /// Appends given bytes to this `BytesMut`.
    ///
    /// If this `BytesMut` object does not have enough capacity, it is resized
    /// first.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut buf = BytesMut::with_capacity(0);
    /// buf.extend_from_slice(b"aaabbb");
    /// buf.extend_from_slice(b"cccddd");
    ///
    /// assert_eq!(b"aaabbbcccddd", &buf[..]);
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(extend@.len() <= self.unique_capacity() - self.unique_length()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_length() == self.unique_length() + extend@.len()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_bytes() == self.unique_bytes().concat(extend@)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_pointer() == self.unique_pointer() && (^self).unique_capacity() == self.unique_capacity() && (^self).unique_tag_pointer() == self.unique_tag_pointer()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int> 0 <= i && i < self.unique_length() ==> (^self).unique_slot(i) == self.unique_slot(i)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int> 0 <= i && i < extend@.len() ==> (^self).unique_slot(self.unique_length() + i) == Some(Some(extend@[i]))))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int> self.unique_length() + extend@.len() <= i && i < self.unique_capacity() ==> (^self).unique_slot(i) == self.unique_slot(i)))]
    #[inline]
    pub fn extend_from_slice(&mut self, extend: &[u8]) {
        let cnt = extend.len();
        self.reserve(cnt);

        unsafe {
            let dst = self.spare_capacity_mut();
            // Reserved above
            debug_assert!(dst.len() >= cnt);

            crate::storage_ops::copy_to_uninit_prefix_raw(dst, extend);
        }

        unsafe {
            self.advance_mut(cnt);
        }
    }

    /// Absorbs a `BytesMut` that was previously split off.
    ///
    /// If the two `BytesMut` objects were previously contiguous and not mutated
    /// in a way that causes re-allocation i.e., if `other` was created by
    /// calling `split_off` on this `BytesMut`, then this is an `O(1)` operation
    /// that just decreases a reference count and sets a few indices.
    /// Otherwise this method degenerates to
    /// `self.extend_from_slice(other.as_ref())`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// let mut buf = BytesMut::with_capacity(64);
    /// buf.extend_from_slice(b"aaabbbcccddd");
    ///
    /// let split = buf.split_off(6);
    /// assert_eq!(b"aaabbb", &buf[..]);
    /// assert_eq!(b"cccddd", &split[..]);
    ///
    /// buf.unsplit(split);
    /// assert_eq!(b"aaabbbcccddd", &buf[..]);
    /// ```
    pub fn unsplit(&mut self, other: BytesMut) {
        if self.is_empty() {
            *self = other;
            return;
        }

        if let Err(other) = self.try_unsplit(other) {
            self.extend_from_slice(other.as_ref());
        }
    }

    // private

    // For now, use a `Vec` to manage the memory for us, but we may want to
    // change that in the future to some alternate allocator strategy.
    //
    // Thus, we don't expose an easy way to construct from a `Vec` since an
    // internal change could make a simple pattern (`BytesMut::from(vec)`)
    // suddenly a lot more expensive.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_length() == vec@.len()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_bytes() == vec@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_capacity() == creusot_std::std::vec::capacity_model(vec)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int> result.unique_length() <= i && i < result.unique_capacity() ==> result.unique_slot(i) == Some(None)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int> 0 <= i && i < vec@.len() ==> result.unique_slot(i) == Some(Some(vec@[i]))))]
    #[inline]
    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {
        let parts = VecRawParts::take(vec);
        let native_ptr = parts.native_ptr();
        let len = parts.logical_len();
        let cap = parts.logical_capacity();
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (raw, capabilities) = parts.into_proof_authority();
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        parts.discard_unmodeled();
        let ptr = vptr(native_ptr.as_ptr());
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let unique_proof = {
            // The B1 descriptor is bound to the native pointer returned by
            // the same VecRawParts extraction used in production builds.
            let (base, _) = raw.bound_ptr_at_zero();
            Some(OriginalUniqueProof {
                base,
                cursor: base,
                raw,
                capabilities,
            })
        };

        let original_capacity_repr = original_capacity_to_repr(cap);
        let data = crate::capacity_ops::pack_vec_metadata(original_capacity_repr);

        BytesMut {
            ptr,
            len,
            cap,
            data: invalid_ptr(data),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            unique_proof,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            shared_proof: None,
        }
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Advance the buffer without bounds checking.
    ///
    /// # SAFETY
    ///
    /// The caller must ensure that `count` <= `self.cap`.
    // The selected proof paths are the rebuildable, no-promotion Vec case and
    // the packet-backed ARC cursor case. They establish these private state
    // transitions from their respective invariants; this does not prove the
    // public producer chain or actual Shared owner/refcount lifecycle updates.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(count@ <= self.cap@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        (self.unique_valid() && self.unique_vec_rebuildable() &&
            count@ <= self.unique_length() &&
            self.unique_offset() + count@ <= MAX_VEC_POS@) ||
        (self.shared_layout_valid() && self.shared_visible_prefix_known())
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.unique_vec_rebuildable() ==> (^self).unique_valid()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.unique_vec_rebuildable() ==> (^self).unique_vec_rebuildable()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.unique_vec_rebuildable() ==>
            ((^self).unique_offset() == self.unique_offset() + count@)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.unique_vec_rebuildable() ==>
            ((^self).unique_capacity() == self.unique_capacity() - count@)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.unique_vec_rebuildable() ==>
            ((^self).unique_length() == if count@ <= self.unique_length() {
                self.unique_length() - count@
            } else { 0 })
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.unique_vec_rebuildable() ==>
            ((^self).unique_allocation_capacity() == self.unique_allocation_capacity())
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.unique_vec_rebuildable() ==>
            (((^self).unique_tag_pointer().addr_logic() >> VEC_POS_OFFSET)@ ==
                self.unique_offset() + count@)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.unique_vec_rebuildable() ==>
            ((^self).unique_pointer().addr_logic()@ ==
                self.unique_pointer().addr_logic()@ + count@)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        self.unique_vec_rebuildable() ==>
            (0 <= i && i < (^self).unique_capacity() ==>
                (^self).unique_slot(i) == self.unique_slot(count@ + i))))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        self.unique_vec_rebuildable() ==>
            (0 <= i && i < self.unique_allocation_capacity() ==>
                (^self).unique_allocation_slot(i) == self.unique_allocation_slot(i))))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.unique_vec_rebuildable() ==>
            ((^self).unique_bytes() == self.unique_bytes().subsequence(
                if count@ <= self.unique_length() { count@ } else { self.unique_length() },
                self.unique_length()))
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> (^self).shared_layout_valid()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_visible_prefix_known() ==> (^self).shared_visible_prefix_known()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> (^self).shared_proof.unwrap_logic().capsule.inner_logic() ==
            self.shared_proof.unwrap_logic().capsule.inner_logic()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> (^self).shared_proof.unwrap_logic().ticket.inner_logic() ==
            self.shared_proof.unwrap_logic().ticket.inner_logic()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> (^self).shared_proof.unwrap_logic().region.inner_logic() ==
            self.shared_proof.unwrap_logic().region.inner_logic()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> (^self).shared_proof.unwrap_logic().base ==
            self.shared_proof.unwrap_logic().base
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> (^self).shared_proof.unwrap_logic().cursor@.unwrap_logic().2 ==
            self.shared_proof.unwrap_logic().cursor@.unwrap_logic().2 + count@
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> ((^self).data == self.data)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> ((^self).len@ ==
            (if count@ <= self.len@ { self.len@ - count@ } else { 0int }))
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> ((^self).cap@ + count@ == self.cap@)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==> ((^self).ptr.addr_logic()@ == self.ptr.addr_logic()@ + count@)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        self.shared_layout_valid() ==>
            (0 <= i && i < self.shared_proof.unwrap_logic().region.inner_logic().capacity() ==>
                (^self).shared_proof.unwrap_logic().allocation_slot(i) ==
                    self.shared_proof.unwrap_logic().allocation_slot(i))))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        self.shared_layout_valid() ==>
            ((^self).shared_bytes() == self.shared_bytes().subsequence(
                if count@ <= self.len@ { count@ } else { self.len@ }, self.len@))
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        context != None && context.unwrap_logic().inner_logic().invariant()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^context.unwrap_logic().inner_logic()).invariant()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        if (self.unique_tag_pointer().addr_logic() & KIND_MASK) == KIND_VEC &&
            self.unique_offset() + count@ > MAX_VEC_POS@ {
            shared_capsule::header_context_registered_idle(
                context.unwrap_logic().inner_logic(),
                ^context.unwrap_logic().inner_logic(),
                (^self).shared_proof.unwrap_logic().lease_key()
            )
        } else {
            shared_capsule::header_context_unchanged(
                context.unwrap_logic().inner_logic(),
                ^context.unwrap_logic().inner_logic()
            )
        }
    ))]
    pub(crate) unsafe fn advance_unchecked(
        &mut self,
        count: usize,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        context: Option<Ghost<&mut BorrowedSharedContext<'_, shared_capsule::HeaderLeaseState>>>,
    ) {
        // Setting the start to 0 is a no-op, so return early if this is the
        // case.
        if count == 0 {
            return;
        }

        debug_assert!(count <= self.cap, "internal: set_start out of bounds");

        let kind = self.kind();
        if kind == KIND_VEC {
            // Setting the start when in vec representation is a little more
            // complicated. First, we have to track how far ahead the
            // "start" of the byte buffer from the beginning of the vec. We
            // also have to ensure that we don't exceed the maximum shift.
            let off = self.get_vec_pos();
            let pos = off + count;

            if pos <= MAX_VEC_POS {
                // The private layout helper updates the Vec cursor, pointer,
                // packed position and capacity together. It keeps the old
                // length until the next non-panicking assignment restores the
                // full invariant.
                unsafe { self.advance_vec_layout(count, pos) };
                let new_len = self.len.saturating_sub(count);
                #[cfg(all(creusot, bytes_original_unique_gate))]
                let _ = ghost! {
                    buffer_prefix_mode_equivalence(snapshot!(*self), new_len);
                };
                self.set_visible_len(new_len);
                return;
            } else {
                // The repr must be upgraded to ARC. This is unusual on 64-bit
                // systems, but the packed Vec offset still has a finite bound.
                unsafe {
                    self.promote_to_shared_count_one(
                        off,
                        pos,
                        count,
                        #[cfg(all(creusot, bytes_original_unique_gate))]
                        context.unwrap(),
                    )
                };
            }
        } else if kind == KIND_ARC {
            unsafe { self.advance_shared_layout(count) };
            return;
        } else {
            self.ptr = vptr(crate::provenance_specs::wrapping_offset(
                self.ptr.as_ptr(),
                count,
            ));
        }
        self.len = self.len.saturating_sub(count);
        self.cap -= count;
    }

    /// Promote the unique Vec-backed owner while its packed offset is about to
    /// overflow. The Vec is resumed from the current B1 authority, immediately
    /// recaptured by B1 in opaque Shared storage, and the handle cursor is
    /// derived from that returned allocation base. In native builds the same
    /// Vec transfer and pointer offset are performed directly.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(off@ == self.unique_offset()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(pos@ == self.unique_offset() + count@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(count@ <= self.unique_capacity()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(pos@ <= self.unique_allocation_capacity()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_proof == None))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_proof != None))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_proof.unwrap_logic().invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_proof.unwrap_logic().capsule_public().0 == (^self).data as *mut ()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_proof.unwrap_logic().cursor.raw_pointer() == (^self).ptr@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_proof.unwrap_logic().cursor@ == Some((
        (^self).shared_proof.unwrap_logic().region.inner_logic().namespace(),
        (^self).shared_proof.unwrap_logic().region.inner_logic().capacity(), pos@
    ))))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_pointer().addr_logic()@ == self.unique_pointer().addr_logic()@ + count@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(((^self).unique_tag_pointer().addr_logic() & KIND_MASK) == KIND_ARC))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < self.unique_offset() + self.unique_length() ==>
            (^self).shared_proof.unwrap_logic().allocation_slot(i) == self.unique_allocation_slot(i)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(context.inner_logic().invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^context.inner_logic()).invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        shared_capsule::header_context_registered_idle(
            context.inner_logic(), ^context.inner_logic(),
            (^self).shared_proof.unwrap_logic().lease_key()
        )
    ))]
    unsafe fn promote_to_shared_count_one(
        &mut self,
        off: usize,
        pos: usize,
        count: usize,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        context: Ghost<&mut BorrowedSharedContext<'_, shared_capsule::HeaderLeaseState>>,
    ) {
        debug_assert_eq!(self.kind(), KIND_VEC);

        let original_capacity_repr = crate::capacity_ops::original_capacity_repr_from_data(
            crate::provenance_specs::pointer_addr(self.data),
        );

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (vec, authority_region, base, allocation_binding) = {
            let prestate = snapshot!(*self);
            proof_assert!((*prestate).unique_vec_rebuildable());
            proof_assert!(off@ == (*prestate).unique_offset());
            proof_assert!(pos@ == off@ + count@);
            proof_assert!(pos@ <= (*prestate).unique_allocation_capacity());
            proof_assert!(forall<i:Int> 0 <= i && i < off@ + self.len@ ==>
                if i < off@ {
                    slot_known((*prestate).unique_allocation_slot(i))
                } else {
                    slot_known((*prestate).unique_slot(i - off@))
                }
            );

            let proof = self.unique_proof.take().unwrap();
            let vec_len = off + self.len;
            let vec = unsafe { raw_vec::resume_vec(proof.raw, vec_len, proof.capabilities) };
            VecStorage::hold_owned(vec)
        };

        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let (vec, base_pointer) = {
            let mut vec = unsafe { rebuild_vec(self.ptr.as_ptr(), self.len, self.cap, off) };
            let base = vec.as_mut_ptr();
            (VecStorage::hold(vec), base)
        };

        // The sealed proof descriptor is used only to derive the proof-side
        // cursor. Its program accessor supplies the actual pointer value; the
        // native build performs the same provenance-preserving offset from
        // the Vec base directly.
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (cursor, pointer) = {
            let cursor = base.advance_within(pos);
            let pointer = cursor.as_ptr();
            (cursor, pointer)
        };
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let pointer = crate::provenance_specs::wrapping_offset(base_pointer, pos);

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (ref_count, owner) = RefCountCell::new_with_owner(1);
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let ref_count = RefCountCell::new(1);

        let shared = Box::new(Shared { vec, original_capacity_repr, ref_count });

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (header, root) = boxed_alignment::into_raw_aligned(shared);
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let header = Box::into_raw(shared);

        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            boxed_alignment::aligned_address_has_clear_low_bit(
                crate::provenance_specs::pointer_addr(header),
                core::mem::align_of::<Shared>(),
            );
            let ledger_ticket = OwnerLedger::new_root(allocation_binding);
            let phase_action = PhaseAuthority::new();
            let packet = shared_capsule::make_count_one_packet(
                header,
                root,
                owner,
                authority_region,
                ledger_ticket,
                phase_action,
                base,
                cursor,
                1,
                context,
            );
            self.shared_proof = Some(packet);
        }

        debug_assert_eq!(
            crate::provenance_specs::pointer_addr(header) & KIND_MASK,
            KIND_ARC,
            "internal: Box<Shared> should have an aligned pointer",
        );
        self.ptr = vptr(pointer);
        self.data = header;
    }

    fn try_unsplit(&mut self, other: BytesMut) -> Result<(), BytesMut> {
        if other.capacity() == 0 {
            return Ok(());
        }

        let ptr = unsafe { self.ptr.as_ptr().add(self.len) };
        if ptr == other.ptr.as_ptr()
            && self.kind() == KIND_ARC
            && other.kind() == KIND_ARC
            && self.data == other.data
        {
            // Contiguous blocks, just combine directly
            self.len += other.len;
            self.cap += other.cap;
            Ok(())
        } else {
            Err(other)
        }
    }

    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result == (self.unique_tag_pointer().addr_logic() & KIND_MASK)))]
    #[inline]
    fn kind(&self) -> usize {
        #[cfg(creusot)]
        let addr = crate::provenance_specs::pointer_addr(self.data);
        #[cfg(not(creusot))]
        let addr = self.data as usize;
        addr & KIND_MASK
    }

    #[cfg_attr(creusot, creusot::no_translate)]
    unsafe fn promote_to_shared(&mut self, ref_cnt: usize) {
        debug_assert_eq!(self.kind(), KIND_VEC);
        debug_assert!(ref_cnt == 1 || ref_cnt == 2);

        let original_capacity_repr =
            (self.data as usize & ORIGINAL_CAPACITY_MASK) >> ORIGINAL_CAPACITY_OFFSET;

        // The vec offset cannot be concurrently mutated, so there
        // should be no danger reading it.
        let off = (self.data as usize) >> VEC_POS_OFFSET;

        // First, allocate a new `Shared` instance containing the
        // `Vec` fields. It's important to note that `ptr`, `len`,
        // and `cap` cannot be mutated without having `&mut self`.
        // This means that these fields will not be concurrently
        // updated and since the buffer hasn't been promoted to an
        // `Arc`, those three fields still are the components of the
        // vector.
        let shared = Box::new(Shared {
            vec: VecStorage::hold(rebuild_vec(self.ptr.as_ptr(), self.len, self.cap, off)),
            original_capacity_repr,
            ref_count: RefCountCell::new(ref_cnt),
        });

        let shared = Box::into_raw(shared);

        // The pointer should be aligned, so this assert should
        // always succeed.
        debug_assert_eq!(shared as usize & KIND_MASK, KIND_ARC);

        self.data = shared;
    }

    /// Makes an exact shallow clone of `self`.
    ///
    /// The kind of `self` doesn't matter, but this is unsafe
    /// because the clone will have the same offsets. You must
    /// be sure the returned value to the user doesn't allow
    /// two views into the same range.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    unsafe fn shallow_clone(&mut self) -> BytesMut {
        if self.kind() == KIND_ARC {
            increment_shared(self.data);
            ptr::read(self)
        } else {
            self.promote_to_shared(/*ref_count = */ 2);
            ptr::read(self)
        }
    }

    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == self.unique_offset()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result@ == self.unique_offset()))]
    #[inline]
    unsafe fn get_vec_pos(&self) -> usize {
        #[cfg(not(creusot))]
        debug_assert_eq!(self.kind(), KIND_VEC);

        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            let addr = crate::provenance_specs::pointer_addr(self.data);
            addr >> VEC_POS_OFFSET
        }
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        { self.data as usize >> VEC_POS_OFFSET }
    }

    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(pos@ <= MAX_VEC_POS@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((self.unique_valid() && pos@ <= MAX_VEC_POS@) ==> (^self).unique_offset() == self.unique_offset()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((self.unique_valid() && pos@ <= MAX_VEC_POS@) ==> (^self).unique_pointer() == self.unique_pointer()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((self.unique_valid() && pos@ <= MAX_VEC_POS@) ==> (^self).unique_capacity() == self.unique_capacity()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((self.unique_valid() && pos@ <= MAX_VEC_POS@) ==> (^self).unique_length() == self.unique_length()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((self.unique_valid() && pos@ <= MAX_VEC_POS@) ==> (^self).unique_allocation_capacity() == self.unique_allocation_capacity()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((self.unique_valid() && pos@ <= MAX_VEC_POS@) ==> (^self).unique_tag_pointer().addr_logic() >> VEC_POS_OFFSET == pos))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((self.unique_valid() && pos@ <= MAX_VEC_POS@) ==> ((^self).unique_tag_pointer().addr_logic() & KIND_MASK) == KIND_VEC))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((self.unique_valid() && pos@ <= MAX_VEC_POS@) ==> ((^self).unique_tag_pointer().addr_logic() & NOT_VEC_POS_MASK) ==
        (self.unique_tag_pointer().addr_logic() & NOT_VEC_POS_MASK)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(((^self).unique_tag_pointer().addr_logic() & 3usize) ==
        (self.unique_tag_pointer().addr_logic() & 3usize)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_proof == self.unique_proof))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).shared_proof == self.shared_proof))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((self.unique_valid() && pos@ <= MAX_VEC_POS@) ==> forall<i:Int>
        0 <= i && i < self.unique_allocation_capacity() ==>
            (^self).unique_allocation_slot(i) == self.unique_allocation_slot(i)))]
    #[inline]
    unsafe fn set_vec_pos(&mut self, pos: usize) {
        #[cfg(not(creusot))]
        debug_assert_eq!(self.kind(), KIND_VEC);
        debug_assert!(pos <= MAX_VEC_POS);

        let old_addr = crate::provenance_specs::pointer_addr(self.data);
        let new_addr = crate::capacity_ops::set_vec_pos_in_data(old_addr, pos);
        self.data = invalid_ptr(new_addr);
    }

    /// Returns the remaining spare capacity of the buffer as a slice of `MaybeUninit<u8>`.
    ///
    /// The returned slice can be used to fill the buffer with data (e.g. by
    /// reading from a file) before marking the data as initialized using the
    /// [`set_len`] method.
    ///
    /// [`set_len`]: BytesMut::set_len
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::BytesMut;
    ///
    /// // Allocate buffer big enough for 10 bytes.
    /// let mut buf = BytesMut::with_capacity(10);
    ///
    /// // Fill in the first 3 elements.
    /// let uninit = buf.spare_capacity_mut();
    /// uninit[0].write(0);
    /// uninit[1].write(1);
    /// uninit[2].write(2);
    ///
    /// // Mark the first 3 bytes of the buffer as being initialized.
    /// unsafe {
    ///     buf.set_len(3);
    /// }
    ///
    /// assert_eq!(&buf[..], &[0, 1, 2]);
    /// ```
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@.len() == self.unique_capacity() - self.unique_length() && (^result)@.len() == self.unique_capacity() - self.unique_length()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_pointer() == self.unique_pointer() && (^self).unique_capacity() == self.unique_capacity() && (^self).unique_tag_pointer() == self.unique_tag_pointer() && (^self).unique_length() == self.unique_length()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int> 0 <= i && i < self.unique_length() ==> (^self).unique_slot(i) == self.unique_slot(i)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int> self.unique_length() <= i && i < self.unique_capacity() ==> self.unique_slot(i) == Some(result@[i-self.unique_length()]@)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int> self.unique_length() <= i && i < self.unique_capacity() ==> (^self).unique_slot(i) == Some((^result)@[i-self.unique_length()]@)))]
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            let proof = self.unique_proof.as_mut().unwrap();
            unsafe { raw_vec::borrow_spare_preserving_prefix(
                proof.cursor, self.len, self.cap, ghost! { &mut proof.capabilities.1 }
            ) }
        }
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        unsafe {
            let ptr = self.ptr.as_ptr().add(self.len);
            let len = self.cap - self.len;

            slice::from_raw_parts_mut(ptr.cast(), len)
        }
    }
}

impl Drop for BytesMut {
    fn drop(&mut self) {
        let kind = self.kind();

        if kind == KIND_VEC {
            unsafe {
                let off = self.get_vec_pos();

                // Vector storage, free the vector
                let _ = rebuild_vec(self.ptr.as_ptr(), self.len, self.cap, off);
            }
        } else if kind == KIND_ARC {
            unsafe { release_shared(self.data) };
        }
    }
}

impl Buf for BytesMut {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn advance(&mut self, cnt: usize) {
        assert!(
            cnt <= self.remaining(),
            "cannot advance past `remaining`: {:?} <= {:?}",
            cnt,
            self.remaining(),
        );
        unsafe {
            // SAFETY: We've checked that `cnt` <= `self.remaining()` and we know that
            // `self.remaining()` <= `self.cap`.
            self.advance_unchecked(
                cnt,
                #[cfg(all(creusot, bytes_original_unique_gate))]
                None,
            );
        }
    }

    #[cfg_attr(creusot, creusot::no_translate)]
    fn copy_to_bytes(&mut self, len: usize) -> Bytes {
        self.split_to(
            len,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            None,
        ).freeze()
    }
}

unsafe impl BufMut for BytesMut {
    #[inline]
    fn remaining_mut(&self) -> usize {
        // Max allocation size is isize::MAX.
        isize::MAX as usize - self.len()
    }

    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(cnt@ <= self.unique_capacity() - self.unique_length()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(forall<i:Int> self.unique_length() <= i && i < self.unique_length() + cnt@ ==> slot_known(self.unique_slot(i))))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(self.unique_vec_rebuildable() ==> (^self).unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_length() == self.unique_length() + cnt@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_pointer() == self.unique_pointer() &&
        (^self).unique_capacity() == self.unique_capacity() &&
        (^self).unique_offset() == self.unique_offset() &&
        (^self).unique_allocation_capacity() == self.unique_allocation_capacity() &&
        (^self).unique_tag_pointer() == self.unique_tag_pointer()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^self).unique_proof == self.unique_proof))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int> (^self).unique_slot(i) == self.unique_slot(i)))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < self.unique_allocation_capacity() ==>
            (^self).unique_allocation_slot(i) == self.unique_allocation_slot(i)))]
    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        let remaining = self.cap - self.len();
        if cnt > remaining {
            super::panic_advance(&TryGetError {
                requested: cnt,
                available: remaining,
            });
        }
        // Addition won't overflow since it is at most `self.cap`.
        let new_len = self.len() + cnt;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let _ = ghost! {
            unique_initialized_prefix_extend(snapshot!(*self), self.len, new_len);
            buffer_prefix_mode_equivalence(snapshot!(*self), new_len);
        };
        self.set_visible_len(new_len);
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        if self.capacity() == self.len() {
            self.reserve(64);
        }
        self.spare_capacity_mut().into()
    }

    // Specialize these methods so they can skip checking `remaining_mut`
    // and `advance_mut`.

    fn put<T: Buf>(&mut self, mut src: T)
    where
        Self: Sized,
    {
        if !src.has_remaining() {
            // prevent calling `copy_to_bytes`->`put`->`copy_to_bytes` infintely when src is empty
            return;
        } else if self.capacity() == 0 {
            // When capacity is zero, try reusing allocation of `src`.
            let src_copy = src.copy_to_bytes(src.remaining());
            drop(src);
            match src_copy.try_into_mut() {
                Ok(bytes_mut) => *self = bytes_mut,
                Err(bytes) => self.extend_from_slice(&bytes),
            }
        } else {
            // In case the src isn't contiguous, reserve upfront.
            self.reserve(src.remaining());

            while src.has_remaining() {
                let s = src.chunk();
                let l = s.len();
                self.extend_from_slice(s);
                src.advance(l);
            }
        }
    }

    fn put_slice(&mut self, src: &[u8]) {
        self.extend_from_slice(src);
    }

    fn put_bytes(&mut self, val: u8, cnt: usize) {
        self.reserve(cnt);
        unsafe {
            let dst = self.spare_capacity_mut();
            // Reserved above
            debug_assert!(dst.len() >= cnt);

            ptr::write_bytes(dst.as_mut_ptr(), val, cnt);

            self.advance_mut(cnt);
        }
    }
}

impl AsRef<[u8]> for BytesMut {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Deref for BytesMut {
    type Target = [u8];

    #[inline]
    #[cfg_attr(creusot, creusot_std::prelude::check(ghost))]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsMut<[u8]> for BytesMut {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl DerefMut for BytesMut {
    #[inline]
    #[cfg_attr(creusot, creusot_std::prelude::check(ghost))]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl<'a> From<&'a [u8]> for BytesMut {
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_length() == src@.len()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_capacity() >= src@.len()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_bytes() == src@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        src@.len() <= i && i < result.unique_capacity() ==> result.unique_slot(i) == Some(None)))]
    fn from(src: &'a [u8]) -> BytesMut {
        BytesMut::from_vec(src.to_vec())
    }
}

impl<'a> From<&'a str> for BytesMut {
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_length() == src@.to_bytes().len()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_capacity() >= src@.to_bytes().len()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_bytes() == src@.to_bytes()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        src@.to_bytes().len() <= i && i < result.unique_capacity() ==> result.unique_slot(i) == Some(None)))]
    fn from(src: &'a str) -> BytesMut {
        BytesMut::from(src.as_bytes())
    }
}

impl From<BytesMut> for Bytes {
    fn from(src: BytesMut) -> Bytes {
        src.freeze()
    }
}

#[cfg(not(creusot))]
impl PartialEq for BytesMut {
    fn eq(&self, other: &BytesMut) -> bool {
        self.as_slice() == other.as_slice()
    }
}

#[cfg(not(creusot))]
impl PartialOrd for BytesMut {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(not(creusot))]
impl Ord for BytesMut {
    fn cmp(&self, other: &BytesMut) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

#[cfg(not(creusot))]
impl Eq for BytesMut {}

impl Default for BytesMut {
    #[inline]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_length() == 0))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_bytes() == Seq::empty()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < result.unique_capacity() ==> result.unique_slot(i) == Some(None)))]
    fn default() -> BytesMut {
        BytesMut::new()
    }
}

impl hash::Hash for BytesMut {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let s: &[u8] = self.as_ref();
        s.hash(state);
    }
}

#[cfg(not(creusot))]
impl Borrow<[u8]> for BytesMut {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(not(creusot))]
impl BorrowMut<[u8]> for BytesMut {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl fmt::Write for BytesMut {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.remaining_mut() >= s.len() {
            self.put_slice(s.as_bytes());
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }

    #[inline]
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        fmt::write(self, args)
    }
}

impl Clone for BytesMut {
    fn clone(&self) -> BytesMut {
        BytesMut::from(&self[..])
    }
}

#[cfg(not(creusot))]
impl IntoIterator for BytesMut {
    type Item = u8;
    type IntoIter = IntoIter<BytesMut>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<'a> IntoIterator for &'a BytesMut {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().iter()
    }
}

impl Extend<u8> for BytesMut {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = u8>,
    {
        let iter = iter.into_iter();

        let (lower, _) = iter.size_hint();
        self.reserve(lower);

        // TODO: optimize
        // 1. If self.kind() == KIND_VEC, use Vec::extend
        for b in iter {
            self.put_u8(b);
        }
    }
}

impl<'a> Extend<&'a u8> for BytesMut {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a u8>,
    {
        self.extend(iter.into_iter().copied())
    }
}

impl Extend<Bytes> for BytesMut {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Bytes>,
    {
        for bytes in iter {
            self.extend_from_slice(&bytes)
        }
    }
}

impl FromIterator<u8> for BytesMut {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn from_iter<T: IntoIterator<Item = u8>>(into_iter: T) -> Self {
        BytesMut::from_vec(Vec::from_iter(into_iter))
    }
}

impl<'a> FromIterator<&'a u8> for BytesMut {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn from_iter<T: IntoIterator<Item = &'a u8>>(into_iter: T) -> Self {
        BytesMut::from_iter(into_iter.into_iter().copied())
    }
}

/*
 *
 * ===== Inner =====
 *
 */

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn increment_shared(ptr: *mut Shared) {
    crate::ref_count_ops::increment(&(*ptr).ref_count);
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn release_shared(ptr: *mut Shared) {
    // `Shared` storage... follow the drop steps from Arc.
    if (*ptr).ref_count.fetch_sub_release() != 1 {
        return;
    }

    // This fence is needed to prevent reordering of use of the data and
    // deletion of the data.  Because it is marked `Release`, the decreasing
    // of the reference count synchronizes with this `Acquire` fence. This
    // means that use of the data happens before decreasing the reference
    // count, which happens before this fence, which happens before the
    // deletion of the data.
    //
    // As explained in the [Boost documentation][1],
    //
    // > It is important to enforce any possible access to the object in one
    // > thread (through an existing reference) to *happen before* deleting
    // > the object in a different thread. This is achieved by a "release"
    // > operation after dropping a reference (any access to the object
    // > through this reference must obviously happened before), and an
    // > "acquire" operation before deleting the object.
    //
    // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
    //
    // Thread sanitizer does not support atomic fences. Use an atomic load
    // instead.
    (*ptr).ref_count.load_acquire();

    // Transfer and retire the native Vec once; the empty replacement is safe
    // for the Shared record's ordinary field drop below.
    let storage = core::mem::take(&mut (*ptr).vec);
    drop(unsafe { storage.take_for_retirement() });

    // Drop the data
    drop(Box::from_raw(ptr));
}

impl Shared {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn is_unique(&self) -> bool {
        // The goal is to check if the current handle is the only handle
        // that currently has access to the buffer. This is done by
        // checking if the `ref_count` is currently 1.
        //
        // The `Acquire` ordering synchronizes with the `Release` as
        // part of the `fetch_sub` in `release_shared`. The `fetch_sub`
        // operation guarantees that any mutations done in other threads
        // are ordered before the `ref_count` is decremented. As such,
        // this `Acquire` will guarantee that those mutations are
        // visible to the current thread.
        self.ref_count.load_acquire() == 1
    }
}

#[cfg_attr(all(creusot, bytes_original_unique_gate), bitwise_proof)]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ <= 7))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    (result@ == 0) == (cap@ < 1024)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    (result@ == 1) == (1024 <= cap@ && cap@ < 2048)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    (result@ == 2) == (2048 <= cap@ && cap@ < 4096)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    (result@ == 3) == (4096 <= cap@ && cap@ < 8192)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    (result@ == 4) == (8192 <= cap@ && cap@ < 16384)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    (result@ == 5) == (16384 <= cap@ && cap@ < 32768)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    (result@ == 6) == (32768 <= cap@ && cap@ < 65536)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    (result@ == 7) == (65536 <= cap@)
))]
#[inline]
fn original_capacity_to_repr(cap: usize) -> usize {
    let width = PTR_WIDTH - ((cap >> MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros() as usize);
    cmp::min(
        width,
        MAX_ORIGINAL_CAPACITY_WIDTH - MIN_ORIGINAL_CAPACITY_WIDTH,
    )
}

#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(repr@ <= 7))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), bitwise_proof)]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    (repr@ == 0) == (result@ == 0)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    repr@ > 0 ==> result@ == (1usize << ((repr as usize) + 9usize))@
))]
fn original_capacity_from_repr(repr: usize) -> usize {
    if repr == 0 {
        return 0;
    }

    1 << (repr + (MIN_ORIGINAL_CAPACITY_WIDTH - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserve_after_split_past_initialized_prefix_does_not_expose_gap() {
        let mut left = BytesMut::with_capacity(16);
        let mut right = left.split_off(1);
        drop(left);

        right.extend_from_slice(b"0123456789");
        right.reserve(16);

        assert!(
            right.capacity() >= right.len() + 16,
            "capacity {} len {}",
            right.capacity(),
            right.len()
        );
        assert_eq!(&right[..], b"0123456789");
    }

    #[test]
    fn reserve_compacts_visible_bytes_before_growing_after_hidden_prefix_is_uninitialized() {
        let original: Vec<u8> = (0..80).map(|i| i as u8).collect();
        let expected = original[32..].to_vec();
        let mut left = BytesMut::from(original.as_slice());
        let mut right = left.split_off(32);

        left.clear();
        for slot in left.spare_capacity_mut() {
            *slot = MaybeUninit::uninit();
        }
        drop(left);

        right.reserve(100);

        assert!(right.capacity() >= right.len() + 100);
        assert_eq!(&right[..], expected.as_slice());
    }

    #[test]
    fn count_one_promotion_preserves_nonzero_offset_view() {
        let original: Vec<u8> = (0..32).collect();
        let expected = original[3..].to_vec();
        let mut buffer = BytesMut::from_vec(original);
        Buf::advance(&mut buffer, 3);

        let offset = unsafe { buffer.get_vec_pos() };
        assert_eq!(offset, 3);
        let pointer = buffer.as_ptr();
        let len = buffer.len();
        let cap = buffer.capacity();

        // Exercise the common native promotion body with no further cursor
        // movement, so the visible view stays valid throughout the check.
        unsafe { buffer.promote_to_shared_count_one(offset, offset, 0) };

        assert_eq!(buffer.kind(), KIND_ARC);
        assert_eq!(buffer.as_ptr(), pointer);
        assert_eq!(buffer.len(), len);
        assert_eq!(buffer.capacity(), cap);
        assert_eq!(&buffer[..], expected.as_slice());
    }

    #[test]
    fn shared_advance_past_len_within_capacity_keeps_empty_view() {
        let mut left = BytesMut::with_capacity(20);
        left.extend_from_slice(b"hello");
        let original_capacity = left.capacity();
        assert!(original_capacity >= 7);

        // split_off promotes the Vec to shared storage before advancing the
        // returned view. Here the cursor count exceeds len but remains within
        // capacity, so the shared path produces an empty view with a suffix.
        let right = left.split_off(7);

        assert_eq!(left.len(), 5);
        assert_eq!(left.capacity(), 7);
        assert_eq!(&left[..], b"hello");
        assert_eq!(right.len(), 0);
        assert_eq!(right.capacity(), original_capacity - 7);
        assert!(right.is_empty());
    }

    #[test]
    fn vec_split_off_past_len_after_advance_keeps_disjoint_capacity() {
        let mut original = Vec::with_capacity(64);
        original.extend(0u8..16);
        let mut left = BytesMut::from_vec(original);
        Buf::advance(&mut left, 3);

        let old_capacity = left.capacity();
        let old_pointer = left.as_ptr();
        let old_len = left.len();
        let split_at = old_len + 4;
        assert!(split_at <= old_capacity);

        let mut right = left.split_off(split_at);

        assert_eq!(left.len(), old_len);
        assert_eq!(left.capacity(), split_at);
        assert_eq!(right.len(), 0);
        assert_eq!(right.capacity(), old_capacity - split_at);
        assert_eq!(left.as_ptr(), old_pointer);
        assert_eq!(right.as_ptr() as usize, old_pointer as usize + split_at);
        assert_eq!(&left[..], &(3u8..16).collect::<Vec<_>>());

        left[0] = 99;
        right.extend_from_slice(b"x");
        assert_eq!(left[0], 99);
        assert_eq!(&right[..], b"x");
    }

    #[test]
    fn vec_split_to_after_advance_keeps_halves_independently_writable() {
        let mut original = Vec::with_capacity(40);
        original.extend(0u8..20);
        let mut right = BytesMut::from_vec(original);
        Buf::advance(&mut right, 4);

        let old_capacity = right.capacity();
        let old_pointer = right.as_ptr();
        let mut left = right.split_to(6);

        assert_eq!(left.len(), 6);
        assert_eq!(left.capacity(), 6);
        assert_eq!(right.len(), 10);
        assert_eq!(right.capacity(), old_capacity - 6);
        assert_eq!(left.as_ptr(), old_pointer);
        assert_eq!(right.as_ptr() as usize, old_pointer as usize + 6);
        assert_eq!(&left[..], &(4u8..10).collect::<Vec<_>>());
        assert_eq!(&right[..], &(10u8..20).collect::<Vec<_>>());

        left[0] = 99;
        right[0] = 88;
        assert_eq!(left[0], 99);
        assert_eq!(right[0], 88);
    }

    #[test]
    fn vec_split_off_at_capacity_and_with_zero_capacity() {
        let mut end = BytesMut::with_capacity(8);
        end.extend_from_slice(b"abc");
        let capacity = end.capacity();
        let tail = end.split_off(capacity);

        assert_eq!(&end[..], b"abc");
        assert_eq!(end.capacity(), capacity);
        assert!(tail.is_empty());
        assert_eq!(tail.capacity(), 0);

        let mut empty = BytesMut::new();
        assert_eq!(empty.capacity(), 0);
        let tail = empty.split_off(0);
        assert!(empty.is_empty() && tail.is_empty());
        assert_eq!(empty.capacity(), 0);
        assert_eq!(tail.capacity(), 0);
    }

    #[test]
    fn split_after_truncate_preserves_initialized_capacity_suffix() {
        let mut original = Vec::with_capacity(24);
        original.extend(0u8..16);
        let mut left = BytesMut::from_vec(original);
        left.truncate(4);

        let right = left.split_off(10);
        assert_eq!(left.len(), 4);
        assert_eq!(right.len(), 0);

        // The Vec allocation still contains the initialized suffix even
        // though truncate hid it. Each split half owns a disjoint part of it.
        unsafe {
            left.set_len(10);
            let mut right = right;
            right.set_len(6);
            assert_eq!(&left[..], &(0u8..10).collect::<Vec<_>>());
            assert_eq!(&right[..], &(10u8..16).collect::<Vec<_>>());
        }
    }

    #[test]
    fn shared_record_preserves_native_vec_field_layout() {
        use core::mem::MaybeUninit;

        let original = MaybeUninit::<OriginalSharedLayout>::uninit();
        let wrapped = MaybeUninit::<Shared>::uninit();
        let original_base = original.as_ptr() as usize;
        let wrapped_base = wrapped.as_ptr() as usize;

        // `addr_of!` computes field addresses without creating references to
        // uninitialized records. This works with the crate's Rust 1.57 MSRV.
        unsafe {
            assert_eq!(
                core::ptr::addr_of!((*original.as_ptr()).vec) as usize - original_base,
                core::ptr::addr_of!((*wrapped.as_ptr()).vec) as usize - wrapped_base,
            );
            assert_eq!(
                core::ptr::addr_of!((*original.as_ptr()).original_capacity_repr) as usize
                    - original_base,
                core::ptr::addr_of!((*wrapped.as_ptr()).original_capacity_repr) as usize
                    - wrapped_base,
            );
            assert_eq!(
                core::ptr::addr_of!((*original.as_ptr()).ref_count) as usize - original_base,
                core::ptr::addr_of!((*wrapped.as_ptr()).ref_count) as usize - wrapped_base,
            );
        }
    }

    #[test]
    fn test_original_capacity_to_repr() {
        assert_eq!(original_capacity_to_repr(0), 0);

        let max_width = 32;

        for width in 1..(max_width + 1) {
            let cap = 1 << width - 1;

            let expected = if width < MIN_ORIGINAL_CAPACITY_WIDTH {
                0
            } else if width < MAX_ORIGINAL_CAPACITY_WIDTH {
                width - MIN_ORIGINAL_CAPACITY_WIDTH
            } else {
                MAX_ORIGINAL_CAPACITY_WIDTH - MIN_ORIGINAL_CAPACITY_WIDTH
            };

            assert_eq!(original_capacity_to_repr(cap), expected);

            if width > 1 {
                assert_eq!(original_capacity_to_repr(cap + 1), expected);
            }

            //  MIN_ORIGINAL_CAPACITY_WIDTH must be bigger than 7 to pass tests below
            if width == MIN_ORIGINAL_CAPACITY_WIDTH + 1 {
                assert_eq!(original_capacity_to_repr(cap - 24), expected - 1);
                assert_eq!(original_capacity_to_repr(cap + 76), expected);
            } else if width == MIN_ORIGINAL_CAPACITY_WIDTH + 2 {
                assert_eq!(original_capacity_to_repr(cap - 1), expected - 1);
                assert_eq!(original_capacity_to_repr(cap - 48), expected - 1);
            }
        }
    }

    #[test]
    fn test_original_capacity_from_repr() {
        assert_eq!(0, original_capacity_from_repr(0));

        let min_cap = 1 << MIN_ORIGINAL_CAPACITY_WIDTH;

        assert_eq!(min_cap, original_capacity_from_repr(1));
        assert_eq!(min_cap * 2, original_capacity_from_repr(2));
        assert_eq!(min_cap * 4, original_capacity_from_repr(3));
        assert_eq!(min_cap * 8, original_capacity_from_repr(4));
        assert_eq!(min_cap * 16, original_capacity_from_repr(5));
        assert_eq!(min_cap * 32, original_capacity_from_repr(6));
        assert_eq!(min_cap * 64, original_capacity_from_repr(7));
    }
}

#[cfg(not(creusot))]
unsafe impl Send for BytesMut {}
#[cfg(not(creusot))]
unsafe impl Sync for BytesMut {}

/*
 *
 * ===== PartialEq / PartialOrd =====
 *
 */

#[cfg(not(creusot))]
impl PartialEq<[u8]> for BytesMut {
    fn eq(&self, other: &[u8]) -> bool {
        &**self == other
    }
}

#[cfg(not(creusot))]
impl PartialOrd<[u8]> for BytesMut {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<BytesMut> for [u8] {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<BytesMut> for [u8] {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<str> for BytesMut {
    fn eq(&self, other: &str) -> bool {
        &**self == other.as_bytes()
    }
}

#[cfg(not(creusot))]
impl PartialOrd<str> for BytesMut {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other.as_bytes())
    }
}

#[cfg(not(creusot))]
impl PartialEq<BytesMut> for str {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<BytesMut> for str {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<Vec<u8>> for BytesMut {
    fn eq(&self, other: &Vec<u8>) -> bool {
        *self == other[..]
    }
}

#[cfg(not(creusot))]
impl PartialOrd<Vec<u8>> for BytesMut {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
        (**self).partial_cmp(&other[..])
    }
}

#[cfg(not(creusot))]
impl PartialEq<BytesMut> for Vec<u8> {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<BytesMut> for Vec<u8> {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        other.partial_cmp(self).map(cmp::Ordering::reverse)
    }
}

#[cfg(not(creusot))]
impl PartialEq<String> for BytesMut {
    fn eq(&self, other: &String) -> bool {
        *self == other[..]
    }
}

#[cfg(not(creusot))]
impl PartialOrd<String> for BytesMut {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other.as_bytes())
    }
}

#[cfg(not(creusot))]
impl PartialEq<BytesMut> for String {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<BytesMut> for String {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

#[cfg(not(creusot))]
impl<'a, T: ?Sized> PartialEq<&'a T> for BytesMut
where
    BytesMut: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

#[cfg(not(creusot))]
impl<'a, T: ?Sized> PartialOrd<&'a T> for BytesMut
where
    BytesMut: PartialOrd<T>,
{
    fn partial_cmp(&self, other: &&'a T) -> Option<cmp::Ordering> {
        self.partial_cmp(*other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<BytesMut> for &[u8] {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<BytesMut> for &[u8] {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<BytesMut> for &str {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<BytesMut> for &str {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        other.partial_cmp(self)
    }
}

#[cfg(not(creusot))]
impl PartialEq<BytesMut> for Bytes {
    fn eq(&self, other: &BytesMut) -> bool {
        other[..] == self[..]
    }
}

#[cfg(not(creusot))]
impl PartialEq<Bytes> for BytesMut {
    fn eq(&self, other: &Bytes) -> bool {
        other[..] == self[..]
    }
}

impl From<BytesMut> for Vec<u8> {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn from(bytes: BytesMut) -> Self {
        let kind = bytes.kind();
        let bytes = ManuallyDrop::new(bytes);

        let mut vec = if kind == KIND_VEC {
            unsafe {
                let off = bytes.get_vec_pos();
                rebuild_vec(bytes.ptr.as_ptr(), bytes.len, bytes.cap, off)
            }
        } else {
            let shared = bytes.data;

            if unsafe { (*shared).is_unique() } {
                let storage = core::mem::take(unsafe { &mut (*shared).vec });
                let vec = unsafe { storage.take_for_restore() };

                unsafe { release_shared(shared) };

                vec
            } else {
                return ManuallyDrop::into_inner(bytes).deref().to_vec();
            }
        };

        let len = bytes.len;

        unsafe {
            ptr::copy(bytes.ptr.as_ptr(), vec.as_mut_ptr(), len);
            vec.set_len(len);
        }

        vec
    }
}

#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(!ptr.is_null_logic()))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == ptr))]
#[inline]
fn vptr(ptr: *mut u8) -> NonNull<u8> {
    if cfg!(debug_assertions) {
        NonNull::new(ptr).expect("Vec pointer should be non-null")
    } else {
        unsafe { NonNull::new_unchecked(ptr) }
    }
}

/// Returns a dangling pointer with the given address. This is used to store
/// integer data in pointer fields.
///
/// It is equivalent to `addr as *mut T`, but this fails on miri when strict
/// provenance checking is enabled.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.addr_logic() == addr))]
#[inline]
fn invalid_ptr<T>(addr: usize) -> *mut T {
    let ptr = core::ptr::null_mut::<u8>().wrapping_add(addr);
    debug_assert_eq!(crate::provenance_specs::pointer_addr(ptr), addr);
    ptr.cast::<T>()
}

#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(off == 0usize))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(proof.raw.invariant()))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(ptr == proof.raw.raw_pointer() && cap@ == proof.raw.capacity()))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(proof.capabilities.inner_logic().0.invariant() && proof.capabilities.inner_logic().1.invariant()))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(proof.capabilities.inner_logic().0.namespace() == proof.raw.namespace()))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(proof.capabilities.inner_logic().1.namespace() == proof.raw.namespace()))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(proof.capabilities.inner_logic().1.resource_id() == proof.raw.namespace()))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(proof.capabilities.inner_logic().0.capacity() == cap@ && proof.capabilities.inner_logic().1.capacity() == cap@))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(proof.capabilities.inner_logic().1.lo() == 0 && proof.capabilities.inner_logic().1.hi() == cap@))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(len <= cap))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(forall<i:Int> 0 <= i && i < len@ ==> slot_known(proof.capabilities.inner_logic().1.slot(i))))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result@.len() == len@))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(creusot_std::std::vec::capacity_model(result) == cap@))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(creusot_std::std::vec::pointer_model(result) == ptr))]
#[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(forall<i:Int> 0 <= i && i < len@ ==> proof.capabilities.inner_logic().1.slot(i) == Some(Some(result@[i]))))]
#[cfg(all(creusot, bytes_original_freeze_gate))]
unsafe fn original_rebuild_unique(ptr: *mut u8, len: usize, cap: usize, off: usize,
    proof: OriginalUniqueProof,
) -> Vec<u8> {
    unsafe { raw_vec::resume_vec(proof.raw, len, proof.capabilities) }
}

unsafe fn rebuild_vec(ptr: *mut u8, mut len: usize, mut cap: usize, off: usize) -> Vec<u8> {
    let ptr = ptr.sub(off);
    len += off;
    cap += off;

    Vec::from_raw_parts(ptr, len, cap)
}

// ===== impl SharedVtable =====

#[cfg(not(creusot))]
static SHARED_VTABLE: Vtable = Vtable {
    clone: shared_v_clone,
    into_vec: shared_v_to_vec,
    into_mut: shared_v_to_mut,
    is_unique: shared_v_is_unique,
    drop: shared_v_drop,
};

// The native static identity cannot be translated by Creusot. This const is only a typechecking
// stand-in; every BytesMut Shared transition that observes or installs it is excluded above.
#[cfg(creusot)]
const SHARED_VTABLE: Vtable = Vtable {
    clone: shared_v_clone,
    into_vec: shared_v_to_vec,
    into_mut: shared_v_to_mut,
    is_unique: shared_v_is_unique,
    drop: shared_v_drop,
};

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_v_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = data.load(Ordering::Relaxed) as *mut Shared;
    increment_shared(shared);

    let data = AtomicPtr::new(shared as *mut ());
    Bytes::with_vtable(ptr, len, data, &SHARED_VTABLE)
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_v_to_vec(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Vec<u8> {
    let shared: *mut Shared = data.load(Ordering::Relaxed).cast();

    if (*shared).is_unique() {
        let shared = &mut *shared;

        // Drop shared
        let storage = core::mem::take(&mut shared.vec);
        let mut vec = unsafe { storage.take_for_restore() };
        release_shared(shared);

        // Copy back buffer
        ptr::copy(ptr, vec.as_mut_ptr(), len);
        vec.set_len(len);

        vec
    } else {
        let v = slice::from_raw_parts(ptr, len).to_vec();
        release_shared(shared);
        v
    }
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_v_to_mut(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> BytesMut {
    let shared: *mut Shared = data.load(Ordering::Relaxed).cast();

    if (*shared).is_unique() {
        let shared = &mut *shared;

        // The capacity is always the original capacity of the buffer
        // minus the offset from the start of the buffer
        let (v_ptr, _, v_capacity) = shared.vec.metadata();
        let offset = ptr.offset_from(v_ptr) as usize;
        let cap = v_capacity - offset;

        let ptr = vptr(ptr as *mut u8);

        BytesMut {
            ptr,
            len,
            cap,
            data: shared,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            unique_proof: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            shared_proof: None,
        }
    } else {
        let v = slice::from_raw_parts(ptr, len).to_vec();
        release_shared(shared);
        BytesMut::from_vec(v)
    }
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_v_is_unique(data: &AtomicPtr<()>) -> bool {
    let shared = data.load(Ordering::Acquire);
    let ref_count = (*shared.cast::<Shared>()).ref_count.load_relaxed();
    ref_count == 1
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_v_drop(data: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    data.with_mut(|shared| {
        release_shared(*shared as *mut Shared);
    });
}

// compile-fails

/// ```compile_fail
/// use bytes::BytesMut;
/// #[deny(unused_must_use)]
/// {
///     let mut b1 = BytesMut::from("hello world");
///     b1.split_to(6);
/// }
/// ```
fn _split_to_must_use() {}

/// ```compile_fail
/// use bytes::BytesMut;
/// #[deny(unused_must_use)]
/// {
///     let mut b1 = BytesMut::from("hello world");
///     b1.split_off(6);
/// }
/// ```
fn _split_off_must_use() {}

/// ```compile_fail
/// use bytes::BytesMut;
/// #[deny(unused_must_use)]
/// {
///     let mut b1 = BytesMut::from("hello world");
///     b1.split();
/// }
/// ```
fn _split_must_use() {}

// fuzz tests
#[cfg(all(test, loom))]
mod fuzz {
    use loom::sync::Arc;
    use loom::thread;

    use super::BytesMut;
    use crate::Bytes;

    #[test]
    fn bytes_mut_cloning_frozen() {
        loom::model(|| {
            let a = BytesMut::from(&b"abcdefgh"[..]).split().freeze();
            let addr = a.as_ptr() as usize;

            // test the Bytes::clone is Sync by putting it in an Arc
            let a1 = Arc::new(a);
            let a2 = a1.clone();

            let t1 = thread::spawn(move || {
                let b: Bytes = (*a1).clone();
                assert_eq!(b.as_ptr() as usize, addr);
            });

            let t2 = thread::spawn(move || {
                let b: Bytes = (*a2).clone();
                assert_eq!(b.as_ptr() as usize, addr);
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }
}
