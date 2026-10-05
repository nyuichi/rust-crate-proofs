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
#[allow(unused)]
use crate::loom::sync::atomic::AtomicMut;
use crate::loom::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use crate::{Buf, BufMut, Bytes, TryGetError};
#[cfg(creusot)]
use creusot_std::prelude::*;

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
// The sequential lifecycle uses explicit ownership predicates. The separate
// valid-handle trait gate additionally enforces its checked type invariant.
pub struct BytesMut {
    #[cfg(not(any(creusot, bytes_proof_probe)))]
    ptr: NonNull<u8>,
    #[cfg(any(creusot, bytes_proof_probe))]
    ptr: crate::ownership_proof::raw_vec::BoundPtr,
    len: usize,
    cap: usize,
    data: *mut Shared,
    // Present only in proof builds. Other construction paths remain outside
    // the offset-zero unique ownership gate until their protocols are proved.
    #[cfg(any(creusot, bytes_proof_probe))]
    unique_at_zero: Ghost<Option<(
        crate::ownership_proof::raw_vec::Recovery,
        crate::ownership_proof::raw_vec::PhysicalRegion,
    )>>,
    #[cfg(any(creusot, bytes_proof_probe))]
    pending_control: Ghost<Option<sequential_shared_control::PendingControl>>,
    #[cfg(any(creusot, bytes_proof_probe))]
    shared_registration: Ghost<Option<sequential_shared_control::HandleRegistration>>,
    #[cfg(any(creusot, bytes_proof_probe))]
    shared_context: Ghost<Option<sequential_shared_control::ControlContext>>,
}

// BEGIN EXACT BYTESMUT INVARIANT
#[cfg(all(creusot, bytes_proof_valid_handle))]
impl creusot_std::invariant::Invariant for BytesMut {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self.proof_initialized() && self.shared_registration.inner_logic() == None }
    }
}
// END EXACT BYTESMUT INVARIANT

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
    buffer: SharedBuffer,
    original_capacity_repr: usize,
    #[cfg(not(any(creusot, bytes_proof_probe)))]
    ref_count: AtomicUsize,
    #[cfg(any(creusot, bytes_proof_probe))]
    ref_count: crate::ownership_proof::sequential_counter::SequentialCounter,
}

// The shared allocation has one raw descriptor, not a live ordinary Vec.
// Handles determine initialized lengths; this owner only retains allocation metadata.
struct SharedBuffer {
    #[cfg(not(any(creusot, bytes_proof_probe)))]
    base: NonNull<u8>,
    #[cfg(any(creusot, bytes_proof_probe))]
    base: crate::ownership_proof::raw_vec::BoundPtr,
    capacity: usize,
}

// Unsupported allocation paths retain metadata without manufacturing a binding.
#[cfg(not(any(creusot, bytes_proof_probe)))]
fn shared_buffer_unbound(base: NonNull<u8>) -> NonNull<u8> { base }
#[cfg(any(creusot, bytes_proof_probe))]
fn shared_buffer_unbound(base: NonNull<u8>) -> crate::ownership_proof::raw_vec::BoundPtr {
    crate::ownership_proof::raw_vec::BoundPtr::unbound(base)
}

// These old call paths remain outside the sequential-counter ownership gate.
// Their false preconditions prevent silently using them in a verified caller.
#[cfg(creusot)]
trait UnverifiedSharedCounterAccess {
    #[requires(false)]
    fn fetch_add(&self, value: usize, order: Ordering) -> usize;
    #[requires(false)]
    fn fetch_sub(&self, value: usize, order: Ordering) -> usize;
    #[requires(false)]
    fn load(&self, order: Ordering) -> usize;
}
#[cfg(creusot)]
impl UnverifiedSharedCounterAccess for crate::ownership_proof::sequential_counter::SequentialCounter {
    #[requires(false)]
    fn fetch_add(&self, _value: usize, _order: Ordering) -> usize { panic!("unverified counter path") }
    #[requires(false)]
    fn fetch_sub(&self, _value: usize, _order: Ordering) -> usize { panic!("unverified counter path") }
    #[requires(false)]
    fn load(&self, _order: Ordering) -> usize { panic!("unverified counter path") }
}

impl SharedBuffer {
    // SAFETY: the caller transfers ownership of the entire allocation, including
    // the prefix preceding this handle. No other owner may deallocate it.
    unsafe fn from_handle(ptr: *mut u8, capacity: usize, offset: usize) -> Self {
        Self { base: shared_buffer_unbound(NonNull::new_unchecked(ptr.sub(offset))), capacity: capacity + offset }
    }

    fn capacity(&self) -> usize { self.capacity }
    fn as_mut_ptr(&self) -> *mut u8 { self.base.as_ptr() }

    // Transfer the allocation, leaving an empty descriptor safe to destroy.
    // Length zero avoids claiming initialization of bytes changed by other handles.
    unsafe fn take_vec(&mut self) -> Vec<u8> {
        let empty = Self { base: shared_buffer_unbound(NonNull::dangling()), capacity: 0 };
        let old = ManuallyDrop::new(mem::replace(self, empty));
        Vec::from_raw_parts(old.base.as_ptr(), 0, old.capacity)
    }

    // SAFETY: the caller has the only live handle. Reallocate the whole raw
    // allocation, including potentially uninitialized gaps before that handle;
    // constructing a Vec with offset + len initialized bytes would be invalid.
    unsafe fn reserve(&mut self, capacity: usize) {
        use alloc::alloc::{alloc, handle_alloc_error, realloc, Layout};
        // Match Vec<u8>'s minimum growth capacity. Layout rejects capacities
        // exceeding isize::MAX before changing the existing descriptor.
        let capacity = cmp::max(capacity, 8);
        let layout = Layout::array::<u8>(capacity).expect("capacity overflow");
        let base = if self.capacity == 0 {
            alloc(layout)
        } else {
            let old_layout = Layout::array::<u8>(self.capacity).unwrap();
            realloc(self.base.as_ptr(), old_layout, capacity)
        };
        let base = match NonNull::new(base) {
            Some(base) => base,
            None => handle_alloc_error(layout),
        };
        self.base = shared_buffer_unbound(base);
        self.capacity = capacity;
    }
}

impl Drop for SharedBuffer {
    fn drop(&mut self) {
        // u8 has no destructor; deallocation does not require initialized bytes.
        unsafe { drop(Vec::from_raw_parts(self.base.as_ptr(), 0, self.capacity)); }
    }
}

// Assert that the alignment of `Shared` is divisible by 2.
// This is a necessary invariant since we depend on allocating `Shared` a
// shared object to implicitly carry the `KIND_ARC` flag in its pointer.
// This flag is set when the LSB is 0.
const _: [(); 0 - mem::align_of::<Shared>() % 2] = []; // Assert that the alignment of `Shared` is divisible by 2.

// Buffer storage strategy flags.
const KIND_ARC: usize = 0b0;
const KIND_VEC: usize = 0b1;
const KIND_MASK: usize = 0b1;

use crate::capacity_ops::{original_capacity_from_repr, original_capacity_to_repr, MAX_VEC_POS};
#[cfg(test)]
use crate::capacity_ops::{MAX_ORIGINAL_CAPACITY_WIDTH, MIN_ORIGINAL_CAPACITY_WIDTH};

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
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.len))]
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
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.cap))]
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
    #[inline]
    pub fn freeze(self) -> Bytes {
        let bytes = ManuallyDrop::new(self);
        if bytes.kind() == KIND_VEC {
            // Just re-use `Bytes` internal Vec vtable
            unsafe {
                let off = bytes.get_vec_pos();
                let vec = rebuild_vec(bytes.ptr.as_ptr(), bytes.len, bytes.cap, off);
                let mut b: Bytes = vec.into();
                b.advance(off);
                b
            }
        } else {
            debug_assert_eq!(bytes.kind(), KIND_ARC);

            let ptr = bytes.ptr.as_ptr();
            let len = bytes.len;
            let data = AtomicPtr::new(bytes.data.cast());
            unsafe { Bytes::with_vtable(ptr, len, data, &SHARED_VTABLE) }
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
    #[cfg_attr(creusot, ensures(result.proof_unique_at_zero_valid()))]
    #[cfg_attr(creusot, ensures(result.len@ == len@))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < result.len@ ==>
        result.proof_unique_slot(index) == Some(Some(0u8))))]
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
    // BEGIN EXACT SPLIT_OFF
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(at <= self.cap))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.proof_split_pair_valid(^self)))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).ptr@ == self.ptr@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, at@))))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).len@ == if self.len < at { self.len@ } else { at@ }))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.len@ == if self.len < at { 0int } else { self.len@ - at@ }))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).cap == at && result.cap@ == self.cap@ - at@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).proof_initialized() && result.proof_initialized()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(forall<index: Int> 0 <= index && index < (^self).cap@ ==>
        (^self).proof_view_slot(index) == self.proof_unique_slot(index)))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(forall<index: Int> 0 <= index && index < result.cap@ ==>
        result.proof_view_slot(index) == self.proof_unique_slot(index + at@)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(self.proof_carrier_ready() && at <= self.cap))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.proof_carrier_pair((^self))))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).ptr@ == self.ptr@ && result.ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + at@))))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).len@ == (if self.len < at { self.len@ } else { at@ }) && result.len@ == (if self.len < at { 0int } else { self.len@ - at@ }) && (^self).cap == at && result.cap@ == self.cap@ - at@))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.proof_initialized() && (^self).proof_initialized()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> 0 <= index && index < at@ ==> (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> 0 <= index && index < self.cap@ - at@ ==> result.proof_view_slot(index) == self.proof_view_slot(index + at@)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.matches(self.shared_context.inner_logic().unwrap_logic()) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches(result.shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((* result.shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.shared_context.inner_logic() == None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.matches(self.shared_context.inner_logic().unwrap_logic()) ==>
            registration.packet.0.logical_id() != result.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.proof_carrier_initialized() && (^self).proof_carrier_initialized()))]
    #[must_use = "consider BytesMut::truncate if you don't need the other half"]
    pub fn split_off(&mut self, at: usize) -> BytesMut {
        assert!(
            at <= self.capacity(),
            "split_off out of bounds: {:?} <= {:?}",
            at,
            self.capacity(),
        );
        unsafe {
            let mut other = self.shallow_clone();
            #[cfg(bytes_proof_repeated_split)]
            { self.proof_finish_carrier_split_off(other, at) }
            #[cfg(not(bytes_proof_repeated_split))]
            {
            #[cfg(all(any(creusot, bytes_proof_probe), not(bytes_proof_repeated_split)))]
            {
                // The returned right handle owns the sole coordinator, as in split_to.
                other.pending_control = ghost! { self.pending_control.take() };
                other.proof_register_split(self, at);
            }
            // SAFETY: We've checked that `at` <= `self.capacity()` above.
            other.advance_unchecked(at);
            self.cap = at;
            self.len = cmp::min(self.len, at);
            other
            }
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
    pub fn split(&mut self) -> BytesMut {
        let len = self.len();
        self.split_to(len)
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
    // BEGIN EXACT SPLIT_TO
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(at <= self.len))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).proof_split_pair_valid(result)))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.ptr@ == self.ptr@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, at@))))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.len == at && (^self).len@ == self.len@ - at@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.cap == at && (^self).cap@ == self.cap@ - at@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).proof_initialized() && result.proof_initialized()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(forall<index: Int> 0 <= index && index < at@ ==>
        result.proof_view_slot(index) == self.proof_unique_slot(index)))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(forall<index: Int> 0 <= index && index < self.len@ - at@ ==>
        (^self).proof_view_slot(index) == self.proof_unique_slot(index + at@)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(self.proof_carrier_ready() && at <= self.len))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).proof_carrier_pair(result)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.ptr@ == self.ptr@ && (^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + at@))))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.len == at && (^self).len@ == self.len@ - at@ && result.cap == at && (^self).cap@ == self.cap@ - at@))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).proof_initialized() && result.proof_initialized()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> 0 <= index && index < at@ ==> result.proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> 0 <= index && index < self.len@ - at@ ==> (^self).proof_view_slot(index) == self.proof_view_slot(index + at@)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.matches(self.shared_context.inner_logic().unwrap_logic()) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((* (^self).shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.shared_context.inner_logic() == None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.matches(self.shared_context.inner_logic().unwrap_logic()) ==>
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != result.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).proof_carrier_initialized() && result.proof_carrier_initialized()))]
    #[must_use = "consider BytesMut::advance if you don't need the other half"]
    pub fn split_to(&mut self, at: usize) -> BytesMut {
        assert!(
            at <= self.len(),
            "split_to out of bounds: {:?} <= {:?}",
            at,
            self.len(),
        );

        unsafe {
            let mut other = self.shallow_clone();
            #[cfg(all(any(creusot, bytes_proof_probe), not(bytes_proof_repeated_split)))]
            self.proof_register_split(&mut other, at);
            #[cfg(bytes_proof_repeated_split)]
            { self.proof_finish_carrier_split(other, at) }
            #[cfg(not(bytes_proof_repeated_split))]
            {
                // SAFETY: We've checked that `at` <= `self.len()` and we know that `self.len()` <=
                // `self.capacity()`.
                self.advance_unchecked(at);
                other.cap = at;
                other.len = at;
                other
            }
        }
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
    // BEGIN EXACT TRUNCATE
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures((^self).len@ == if len <= self.len { len@ } else { self.len@ }))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized()))]
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
    // BEGIN EXACT CLEAR
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures((^self).len == 0usize))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized()))]
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
    // BEGIN EXACT RESIZE
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures(self.proof_same_storage(^self)))]
    #[cfg_attr(creusot, requires(new_len <= self.cap))]
    #[cfg_attr(creusot, ensures((^self).len == new_len))]
    #[cfg_attr(creusot, ensures(forall<index: Int> self.len@ <= index && index < new_len@ ==>
        (^self).proof_view_slot(index) == Some(Some(value))))]
    #[cfg_attr(creusot, ensures(forall<index: Int> !(self.len@ <= index && index < new_len@) ==>
        (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_registration.inner_logic() != None ==>
        (^self).shared_registration.inner_logic().unwrap_logic().packet.0 == self.shared_registration.inner_logic().unwrap_logic().packet.0 &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.lo() == self.shared_registration.inner_logic().unwrap_logic().packet.1.lo() &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.shared_registration.inner_logic().unwrap_logic().packet.1.hi()))]
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
        crate::storage_ops::fill_uninit_prefix(self.spare_capacity_mut(), additional, value);

        // SAFETY: There are at least `new_len` initialized bytes in the buffer so no
        // uninitialized bytes are being exposed.
        unsafe { self.set_len(new_len) };
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
    #[inline]
    // BEGIN EXACT SET_LEN
    #[cfg_attr(creusot, requires(self.proof_owned_valid()))]
    #[cfg_attr(creusot, requires(len <= self.cap))]
    #[cfg_attr(creusot, requires(forall<index: Int> 0 <= index && index < len@ ==>
        crate::ownership_proof::raw_vec::slot_known(self.proof_view_slot(index))))]
    #[cfg_attr(creusot, ensures((^self).len == len))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized()))]
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.cap, "set_len out of bounds");
        self.len = len;
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
    #[inline]
    // BEGIN EXACT RESERVE
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, requires(additional@ <= self.cap@ - self.len@))]
    #[cfg_attr(creusot, ensures(^self == *self))]
    pub fn reserve(&mut self, additional: usize) {
        let len = self.len();
        let rem = self.capacity() - len;

        if additional <= rem {
            // The handle can already store at least `additional` more bytes, so
            // there is no further work needed to be done.
            return;
        }

        // The proof gate covers only the native fast path above.
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        { let _ = self.reserve_inner(additional, true); }
        #[cfg(any(creusot, bytes_proof_probe))]
        panic!("growing reserve is outside the in-capacity proof gate");
    }

    // In separate function to allow the short-circuits in `reserve` and `try_reclaim` to
    // be inline-able. Significantly helps performance. Returns false if it did not succeed.
    fn reserve_inner(&mut self, additional: usize, allocate: bool) -> bool {
        // This path is not connected to the physical resource protocol yet.
        // Discard proof authority rather than retain a stale byte/owner claim.
        #[cfg(creusot)]
        { self.unique_at_zero = ghost! { None }; }
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
                let v = &mut (*shared).buffer;

                let v_capacity = v.capacity();
                let ptr = v.as_mut_ptr();

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
                } else if v_capacity >= new_cap && offset >= len {
                    // The capacity is sufficient, and copying is not too much
                    // overhead: reclaim the buffer!

                    // `offset >= len` means: no overlap
                    ptr::copy_nonoverlapping(self.ptr.as_ptr(), ptr, len);

                    self.ptr = vptr(ptr);
                    self.cap = v.capacity();
                } else {
                    if !allocate {
                        return false;
                    }

                    // new_cap is calculated in terms of `BytesMut`, not the underlying
                    // `Vec`, so it does not take the offset into account.
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
                    let double = v.capacity().checked_shl(1).unwrap_or(new_cap);

                    new_cap = cmp::max(double, new_cap);

                    // No space - allocate more
                    //
                    // Reallocate the raw allocation without asserting that the
                    // discarded prefix or spare capacity is initialized. The
                    // allocator preserves the current handle's bytes at offset.
                    debug_assert!(offset + len <= v.capacity());
                    v.reserve(new_cap);

                    // Update the info
                    self.ptr = vptr(v.as_mut_ptr().add(offset));
                    self.cap = v.capacity() - offset;
                }

                return true;
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
        let data = crate::capacity_ops::pack_vec_metadata(original_capacity_repr);
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
    #[inline]
    // BEGIN EXACT EXTEND_FROM_SLICE
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures(self.proof_same_storage(^self)))]
    #[cfg_attr(creusot, requires(extend@.len() <= self.cap@ - self.len@))]
    #[cfg_attr(creusot, ensures((^self).len@ == self.len@ + extend@.len()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < extend@.len() ==>
        (^self).proof_view_slot(self.len@ + index) == Some(Some(extend@[index]))))]
    #[cfg_attr(creusot, ensures(forall<index: Int> !(self.len@ <= index && index < self.len@ + extend@.len()) ==>
        (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_registration.inner_logic() != None ==>
        (^self).shared_registration.inner_logic().unwrap_logic().packet.0 == self.shared_registration.inner_logic().unwrap_logic().packet.0 &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.lo() == self.shared_registration.inner_logic().unwrap_logic().packet.1.lo() &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.shared_registration.inner_logic().unwrap_logic().packet.1.hi()))]
    pub fn extend_from_slice(&mut self, extend: &[u8]) {
        let cnt = extend.len();
        self.reserve(cnt);
        let new_len = self.len() + cnt;
        crate::storage_ops::copy_to_uninit_prefix(self.spare_capacity_mut(), extend);
        // SAFETY: the reserved prefix was initialized by the shared native helper.
        unsafe { self.set_len(new_len); }
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
    #[inline]
    #[cfg_attr(creusot, ensures(result.proof_unique_at_zero_valid()))]
    #[cfg_attr(creusot, ensures(result.len@ == vec@.len()))]
    #[cfg_attr(creusot, ensures(result.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, ensures(result.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(creusot, ensures(
        ((result.data.addr_logic() & crate::capacity_ops::ORIGINAL_CAPACITY_MASK)
            >> crate::capacity_ops::ORIGINAL_CAPACITY_OFFSET)@
            == crate::capacity_ops::original_capacity_class_logic(result.cap)
    ))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < vec@.len() ==>
        result.proof_unique_slot(index) == Some(Some(vec@[index]))))]
    #[cfg_attr(creusot, ensures(forall<index: Int> vec@.len() <= index && index < result.cap@ ==>
        result.proof_unique_slot(index) == Some(None)))]
    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        let (ptr, len, cap) = {
            let mut vec = ManuallyDrop::new(vec);
            (vptr(vec.as_mut_ptr()), vec.len(), vec.capacity())
        };
        #[cfg(any(creusot, bytes_proof_probe))]
        let (ptr, len, cap, capabilities) =
            crate::ownership_proof::bound_ptr::detach_bound_vec(vec);

        let original_capacity_repr = original_capacity_to_repr(cap);
        let data = crate::capacity_ops::pack_vec_metadata(original_capacity_repr);

        BytesMut {
            ptr,
            len,
            cap,
            data: invalid_ptr(data),
            #[cfg(any(creusot, bytes_proof_probe))]
            unique_at_zero: ghost! { Some(capabilities.into_inner()) },
            #[cfg(any(creusot, bytes_proof_probe))]
            pending_control: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_registration: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_context: ghost! { None },
        }
    }

    // BEGIN EXACT BYTESMUT SEQUENTIAL SPLIT METHODS
    #[cfg(creusot)]
    #[logic]
    pub(crate) fn proof_view_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                Some((_, region)) => region.slot(self.ptr@.unwrap_logic().2 + index),
                None => match self.shared_registration.inner_logic() {
                    None => None,
                    Some(registration) => registration.packet.1.slot(self.ptr@.unwrap_logic().2 + index),
                },
            }
        }
    }
    // Absolute packet slots remain owned even after the visible pointer advances.
    #[cfg(creusot)]
    #[logic]
    pub(crate) fn proof_owned_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                Some((_, region)) => region.slot(index),
                None => match self.shared_registration.inner_logic() {
                    None => None,
                    Some(registration) => registration.packet.1.slot(index),
                },
            }
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_empty_valid(self) -> bool {
        pearlite! {
            self.len == 0usize && self.cap == 0usize &&
            self.data.addr_logic() == KIND_VEC && self.ptr.invariant() && self.ptr@ == None &&
            self.unique_at_zero.inner_logic() == None && self.pending_control.inner_logic() == None &&
            self.shared_registration.inner_logic() == None && self.shared_context.inner_logic() == None
        }
    }
    #[cfg(all(creusot, not(bytes_proof_valid_handle)))]
    #[logic(prophetic)]
    fn proof_owned_valid(self) -> bool {
        pearlite! { self.proof_unique_owned() || self.proof_registered_valid() }
    }
    #[cfg(all(creusot, bytes_proof_valid_handle))]
    #[logic(prophetic)]
    fn proof_owned_valid(self) -> bool {
        pearlite! {
            self.proof_unique_at_zero_owned() || self.proof_registered_valid() || self.proof_empty_valid()
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_initialized(self) -> bool {
        pearlite! {
            self.proof_owned_valid() &&
            forall<index: Int> 0 <= index && index < self.len@ ==>
                crate::ownership_proof::raw_vec::slot_known(self.proof_view_slot(index))
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_pending_valid(self) -> bool {
        pearlite! {
            self.unique_at_zero.inner_logic() == None &&
            self.shared_registration.inner_logic() == None && self.shared_context.inner_logic() == None &&
            self.len <= self.cap && self.data.addr_logic() & KIND_MASK == KIND_ARC &&
            match self.pending_control.inner_logic() {
                Some(pending) => pending.valid(self.data, self.ptr, self.cap), None => false,
            }
        }
    }
    // A matching affine registration protects the allocation even while a
    // split is still adjusting the visible descriptor. This intentionally does
    // not require the completed view's role bounds/capacity geometry.
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_shared_allocation_registered(self) -> bool {
        pearlite! {
            self.data.addr_logic() & KIND_MASK == KIND_ARC && self.ptr@ != None &&
            match self.shared_registration.inner_logic() {
                None => false,
                Some(registration) => registration.valid() &&
                    registration.control.pointer == self.data &&
                    self.ptr@.unwrap_logic().0 == registration.status.allocation &&
                    self.ptr@.unwrap_logic().1 == registration.status.capacity,
            }
        }
    }

    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_registered_valid(self) -> bool {
        pearlite! {
            self.unique_at_zero.inner_logic() == None && self.pending_control.inner_logic() == None &&
            self.len <= self.cap && self.ptr.invariant() &&
            self.data.addr_logic() & KIND_MASK == KIND_ARC &&
            match self.shared_registration.inner_logic() {
                None => false,
                Some(registration) => registration.valid() && registration.control.pointer == self.data &&
                    self.ptr@ != None &&
                    self.ptr@.unwrap_logic().0 == registration.status.allocation &&
                    self.ptr@.unwrap_logic().1 == registration.status.capacity &&
                    self.ptr@.unwrap_logic().2 + self.cap@ <= registration.status.capacity &&
                    registration.view_lo() <= self.ptr@.unwrap_logic().2 &&
                    self.ptr@.unwrap_logic().2 + self.cap@ ==
                        registration.view_hi(),
            }
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    pub(crate) fn proof_split_pair_valid(self, left: Self) -> bool {
        pearlite! {
            self.proof_registered_valid() && left.proof_registered_valid() &&
            !self.shared_registration.inner_logic().unwrap_logic().left &&
            left.shared_registration.inner_logic().unwrap_logic().left &&
            left.shared_context.inner_logic() == None &&
            match self.shared_context.inner_logic() {
                None => false,
                Some(context) =>
                    self.shared_registration.inner_logic().unwrap_logic().matches(context) &&
                    left.shared_registration.inner_logic().unwrap_logic().matches(context) &&
                    context.status.left_pending && context.status.right_pending,
            }
        }
    }
    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(self.proof_pending_valid()))]
    #[cfg_attr(creusot, requires(at <= self.cap))]
    #[cfg_attr(creusot, requires(other.ptr == self.ptr && other.len == self.len && other.cap == self.cap && other.data == self.data))]
    #[cfg_attr(creusot, requires(other.unique_at_zero.inner_logic() == None && other.pending_control.inner_logic() == None && other.shared_registration.inner_logic() == None && other.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^other).ptr == other.ptr && (^other).len == other.len && (^other).cap == other.cap && (^other).data == other.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero.inner_logic() == None && (^self).pending_control.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^other).unique_at_zero.inner_logic() == None && (^other).pending_control.inner_logic() == None && (^other).shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic() != None && (^other).shared_registration.inner_logic() != None && (^self).shared_context.inner_logic() != None))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().valid() && (^other).shared_registration.inner_logic().unwrap_logic().valid()))]
    #[cfg_attr(creusot, ensures(!(^self).shared_registration.inner_logic().unwrap_logic().left && (^other).shared_registration.inner_logic().unwrap_logic().left))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().matches((^self).shared_context.inner_logic().unwrap_logic()) && (^other).shared_registration.inner_logic().unwrap_logic().matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((^self).shared_context.inner_logic().unwrap_logic().status.left_pending && (^self).shared_context.inner_logic().unwrap_logic().status.right_pending))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().control.pointer == self.data && (^other).shared_registration.inner_logic().unwrap_logic().control.pointer == self.data))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().status.capacity == self.cap@ && (^self).shared_registration.inner_logic().unwrap_logic().status.split == at@))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().same_registration((^other).shared_registration.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures(self.ptr@ == Some(((^self).shared_registration.inner_logic().unwrap_logic().status.allocation, self.cap@, 0int))))]
    #[cfg_attr(creusot, ensures(forall<index: Int> (^other).shared_registration.inner_logic().unwrap_logic().packet.1.slot(index) ==
        if 0 <= index && index < at@ { self.pending_control.inner_logic().unwrap_logic().caps.1.slot(index) } else { None }))]
    #[cfg_attr(creusot, ensures(forall<index: Int> (^self).shared_registration.inner_logic().unwrap_logic().packet.1.slot(index) ==
        if at@ <= index && index < self.cap@ { self.pending_control.inner_logic().unwrap_logic().caps.1.slot(index) } else { None }))]
    fn proof_register_split(&mut self, other: &mut Self, at: usize) {
        let pending = ghost! { self.pending_control.take().unwrap() };
        let (context, left, right) = sequential_shared_control::activate(self.data, self.ptr, self.cap, at, pending);
        self.shared_context = ghost! { Some(context.into_inner()) };
        self.shared_registration = ghost! { Some(right.into_inner()) };
        other.shared_registration = ghost! { Some(left.into_inner()) };
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(self.shared_context.inner_logic() != None))]
    #[cfg_attr(creusot, ensures(result.inner_logic() == self.shared_context.inner_logic().unwrap_logic()))]
    #[cfg_attr(creusot, ensures((^self).shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration))]
    fn proof_take_coordinator(&mut self) -> Ghost<sequential_shared_control::ControlContext> {
        ghost! { self.shared_context.take().unwrap() }
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(self.proof_registered_valid()))]
    #[cfg_attr(creusot, requires(self.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(self.shared_registration.inner_logic().unwrap_logic().left == left))]
    #[cfg_attr(creusot, requires(self.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic())))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).valid(self.shared_registration.inner_logic().unwrap_logic().control)))]
    #[cfg_attr(creusot, ensures(result == !(^context.inner_logic()).active()))]
    #[cfg_attr(creusot, ensures(result == (!(^context.inner_logic()).status.left_pending && !(^context.inner_logic()).status.right_pending)))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).status.allocation == context.inner_logic().status.allocation && (^context.inner_logic()).status.capacity == context.inner_logic().status.capacity && (^context.inner_logic()).status.split == context.inner_logic().status.split && (^context.inner_logic()).status.registration == context.inner_logic().status.registration))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).status.left_pending == (if left { false } else { context.inner_logic().status.left_pending })))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).status.right_pending == (if left { context.inner_logic().status.right_pending } else { false })))]
    fn proof_release(mut self, context: Ghost<&mut sequential_shared_control::ControlContext>, left: bool) -> bool {
        let registration = ghost! { self.shared_registration.take().unwrap() };
        let identity = ghost! { registration.control.identity.into_inner() };
        let control = sequential_shared_control::ControlPtr { pointer: self.data, identity };
        let packet = ghost! { registration.into_inner().packet };
        let last = sequential_shared_control::release(control, context, packet, left);
        #[cfg(bytes_proof_valid_handle)]
        {
            // Authority has already been consumed. Normalize individual fields;
            // assigning a whole handle here would run the old armed destructor.
            proof_assert!(self.unique_at_zero.inner_logic() == None && self.pending_control.inner_logic() == None && self.shared_registration.inner_logic() == None && self.shared_context.inner_logic() == None);
            self.ptr = crate::ownership_proof::raw_vec::BoundPtr::unbound(NonNull::dangling());
            self.len = 0;
            self.cap = 0;
            self.data = invalid_ptr(KIND_VEC);
        }
        mem::forget(self);
        last
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(at@ <= input@.len()))]
    pub(crate) fn proof_split_then_release(input: Vec<u8>, at: usize, right_first: bool) {
        let mut right = Self::from_vec(input);
        let left = right.split_to(at);
        let mut context = right.proof_take_coordinator();
        if right_first {
            let first = right.proof_release(context.borrow_mut(), false);
            assert!(!first);
            let last = left.proof_release(context.borrow_mut(), true);
            assert!(last);
        } else {
            let first = left.proof_release(context.borrow_mut(), true);
            assert!(!first);
            let last = right.proof_release(context.borrow_mut(), false);
            assert!(last);
        }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), feature = "negative_missing_split_ticket"))]
    pub(crate) fn proof_reject_missing_empty_ticket(input: Vec<u8>) {
        let mut right = Self::from_vec(input);
        let left = right.split_to(0);
        let mut context = right.proof_take_coordinator();
        mem::forget(left);
        let last = right.proof_release(context.borrow_mut(), false);
        assert!(!last);
        proof_assert!(context.status.left_pending && !context.status.right_pending);
        // Full byte coverage cannot replace the missing empty-handle ticket.
        let _forbidden = crate::ownership_proof::shared_protocol::finish(
            ghost! { context.registry.take().unwrap() },
        );
    }
    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(0 < at@ && at@ < input@.len()))]
    pub(crate) fn proof_mutate_split_then_release(input: Vec<u8>, at: usize, left_value: u8, right_value: u8, right_first: bool) {
        let mut right = Self::from_vec(input);
        let mut left = right.split_to(at);
        {
            let left_slice = left.as_slice_mut();
            let right_slice = right.as_slice_mut();
            left_slice[0] = left_value;
            right_slice[0] = right_value;
            assert!(left_slice[0] == left_value);
            assert!(right_slice[0] == right_value);
        }
        proof_assert!(left.proof_view_slot(0int) == Some(Some(left_value)));
        proof_assert!(right.proof_view_slot(0int) == Some(Some(right_value)));
        let mut context = right.proof_take_coordinator();
        if right_first {
            let first = right.proof_release(context.borrow_mut(), false);
            assert!(!first);
            let last = left.proof_release(context.borrow_mut(), true);
            assert!(last);
        } else {
            let first = left.proof_release(context.borrow_mut(), true);
            assert!(!first);
            let last = right.proof_release(context.borrow_mut(), false);
            assert!(last);
        }
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(at@ <= input@.len()))]
    pub(crate) fn proof_access_split_then_release(input: Vec<u8>, at: usize, right_first: bool) {
        let mut right = Self::from_vec(input);
        let mut left = right.split_to(at);
        {
            let left_slice = left.as_slice_mut();
            let right_slice = right.as_slice_mut();
            assert!(left_slice.len() == at);
            let _ = right_slice.len();
        }
        let mut context = right.proof_take_coordinator();
        if right_first {
            assert!(!right.proof_release(context.borrow_mut(), false));
            assert!(left.proof_release(context.borrow_mut(), true));
        } else {
            assert!(!left.proof_release(context.borrow_mut(), true));
            assert!(right.proof_release(context.borrow_mut(), false));
        }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), feature = "negative_pending_access"))]
    #[cfg_attr(creusot, requires(input@.len() >= 1))]
    pub(crate) fn proof_reject_pending_access(input: Vec<u8>) {
        let mut owner = Self::from_vec(input);
        let mut pending = unsafe { owner.shallow_clone() };
        // Metadata alone does not provide a registered initialized region.
        let _forbidden = pending.as_slice_mut();
    }

    #[cfg(all(any(creusot, bytes_proof_probe), feature = "negative_stale_contents"))]
    #[cfg_attr(creusot, requires(input@.len() >= 2 && input@[0] != 17u8))]
    pub(crate) fn proof_reject_stale_contents(input: Vec<u8>) {
        let original: Snapshot<u8> = snapshot!(input@[0]);
        let mut right = Self::from_vec(input);
        let mut left = right.split_to(1);
        left.as_slice_mut()[0] = 17;
        // B4 writeback replaces the old byte; the stale value is unavailable.
        proof_assert!(left.proof_view_slot(0int) == Some(Some(*original)));
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    pub(crate) fn proof_split_off_then_release(input: Vec<u8>, requested: usize, right_first: bool) {
        let mut left = Self::from_vec(input);
        // Vec's unchanged sequence model has no capacity field. Select a native
        // in-capacity position; every legal split_off position is covered.
        let at = cmp::min(requested, left.capacity());
        let old_len = left.len();
        let old_cap = left.capacity();
        let mut right = left.split_off(at);
        assert!(left.len() == cmp::min(old_len, at));
        assert!(right.len() == old_len.saturating_sub(at));
        assert!(left.capacity() == at);
        assert!(right.capacity() == old_cap - at);
        {
            let left_slice = left.as_slice_mut();
            let right_slice = right.as_slice_mut();
            if left_slice.len() > 0 { left_slice[0] = 71; }
            if right_slice.len() > 0 { right_slice[0] = 93; }
            if left_slice.len() > 0 { assert!(left_slice[0] == 71); }
            if right_slice.len() > 0 { assert!(right_slice[0] == 93); }
        }
        let mut context = right.proof_take_coordinator();
        if right_first {
            assert!(!right.proof_release(context.borrow_mut(), false));
            assert!(left.proof_release(context.borrow_mut(), true));
        } else {
            assert!(!left.proof_release(context.borrow_mut(), true));
            assert!(right.proof_release(context.borrow_mut(), false));
        }
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    pub(crate) fn proof_shrink_split_off_then_release(input: Vec<u8>, requested: usize, keep: usize, right_first: bool) {
        let mut left = Self::from_vec(input);
        let at = cmp::min(requested, left.capacity());
        let mut right = left.split_off(at);
        let old_left_len = left.len();
        let old_right_len = right.len();
        left.truncate(keep);
        right.clear();
        assert!(left.len() == cmp::min(old_left_len, keep));
        assert!(right.len() == 0);
        // Length-only shrinking retained initialized slot ownership. Restore
        // exactly the old visible prefixes; no spare Unknown byte is exposed.
        unsafe {
            left.set_len(old_left_len);
            right.set_len(old_right_len);
        }
        {
            let left_slice = left.as_slice_mut();
            let right_slice = right.as_slice_mut();
            if left_slice.len() > 0 { left_slice[0] = 71; }
            assert!(right_slice.len() == old_right_len);
        }
        let mut context = right.proof_take_coordinator();
        if right_first {
            assert!(!right.proof_release(context.borrow_mut(), false));
            assert!(left.proof_release(context.borrow_mut(), true));
        } else {
            assert!(!left.proof_release(context.borrow_mut(), true));
            assert!(right.proof_release(context.borrow_mut(), false));
        }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), feature = "negative_split_off_unknown"))]
    pub(crate) fn proof_reject_split_off_unknown() {
        let mut left = Self::from_vec(Vec::with_capacity(4));
        if left.capacity() > 1 {
            let mut right = left.split_off(1);
            assert!(right.len() == 0);
            proof_assert!(right.proof_view_slot(0int) == Some(None));
            proof_assert!(right.proof_registered_valid() && right.cap >= 1usize);
            // The actual unsafe API must not expose an Unknown spare byte.
            // Capacity and registration suffice; initialized-prefix authority does not.
            unsafe { right.set_len(1); }
        }
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    pub(crate) fn proof_advance_split_off_then_release(input: Vec<u8>, requested: usize, left_count: usize, right_count: usize, right_first: bool) {
        let mut left = Self::from_vec(input);
        let at = cmp::min(requested, left.capacity());
        let mut right = left.split_off(at);
        let left_count = cmp::min(left_count, left.capacity());
        let right_count = cmp::min(right_count, right.capacity());
        let left_len = left.len();
        let right_len = right.len();
        let left_before = snapshot!(left);
        let right_before = snapshot!(right);
        unsafe {
            left.advance_unchecked(left_count);
            right.advance_unchecked(right_count);
        }
        assert!(left.len() == left_len.saturating_sub(left_count));
        assert!(right.len() == right_len.saturating_sub(right_count));
        proof_assert!(forall<index: Int> left.proof_owned_slot(index) == left_before.proof_owned_slot(index));
        proof_assert!(forall<index: Int> right.proof_owned_slot(index) == right_before.proof_owned_slot(index));
        {
            let left_slice = left.as_slice_mut();
            let right_slice = right.as_slice_mut();
            if left_slice.len() > 0 { left_slice[0] = 71; }
            if right_slice.len() > 0 { right_slice[0] = 93; }
        }
        // The borrow frame preserves discarded prefixes as well as spare slots.
        proof_assert!(forall<index: Int> 0 <= index && index < left_count@ ==>
            left.proof_owned_slot(index) == left_before.proof_owned_slot(index));
        proof_assert!(forall<index: Int> at@ <= index && index < at@ + right_count@ ==>
            right.proof_owned_slot(index) == right_before.proof_owned_slot(index));
        let mut context = right.proof_take_coordinator();
        if right_first {
            assert!(!right.proof_release(context.borrow_mut(), false));
            assert!(left.proof_release(context.borrow_mut(), true));
        } else {
            assert!(!left.proof_release(context.borrow_mut(), true));
            assert!(right.proof_release(context.borrow_mut(), false));
        }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), feature = "negative_advanced_unknown"))]
    pub(crate) fn proof_reject_advanced_unknown() {
        let mut left = Self::from_vec(Vec::with_capacity(4));
        if left.capacity() > 1 {
            let mut right = left.split_off(0);
            unsafe { right.advance_unchecked(1); }
            assert!(right.len() == 0 && right.cap >= 1);
            proof_assert!(right.proof_registered_valid());
            proof_assert!(right.proof_view_slot(0int) == Some(None));
            // Advancing a valid owned view does not initialize its spare bytes.
            unsafe { right.set_len(1); }
        }
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, requires(self.len < self.cap ==> self.proof_view_slot(self.len@) == Some(None)))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures((^self).len@ == self.len@ + if self.len < self.cap { 1int } else { 0int }))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures(((^self).unique_at_zero.inner_logic() == None) == (self.unique_at_zero.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures(self.unique_at_zero.inner_logic() != None ==> (^self).unique_at_zero.inner_logic().unwrap_logic().0 == self.unique_at_zero.inner_logic().unwrap_logic().0))]
    #[cfg_attr(creusot, ensures(((^self).shared_registration.inner_logic() == None) == (self.shared_registration.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures((^self).pending_control == self.pending_control && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().control == self.shared_registration.inner_logic().unwrap_logic().control))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().status == self.shared_registration.inner_logic().unwrap_logic().status))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().left == self.shared_registration.inner_logic().unwrap_logic().left))]
    #[cfg_attr(creusot, ensures(self.len < self.cap ==> (^self).proof_view_slot(self.len@) == Some(Some(value))))]
    fn proof_initialize_spare_byte(&mut self, value: u8, use_write: bool) {
        let old_len = self.len;
        let grow = old_len < self.cap;
        {
            let spare = self.spare_capacity_mut();
            if grow {
                if use_write { spare[0].write(value); }
                else { spare[0] = MaybeUninit::new(value); }
            }
        }
        if grow { unsafe { self.set_len(old_len + 1); } }
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    pub(crate) fn proof_initialize_split_spare(input: Vec<u8>, requested: usize, right_first: bool) {
        let mut left = Self::from_vec(input);
        let at = cmp::min(requested, left.capacity());
        let mut right = left.split_off(at);
        let left_len = left.len();
        let right_len = right.len();
        left.proof_initialize_spare_byte(41, true);
        right.proof_initialize_spare_byte(57, false);
        {
            let left_slice = left.as_slice_mut();
            let right_slice = right.as_slice_mut();
            if left_slice.len() > left_len { assert!(left_slice[left_len] == 41); }
            if right_slice.len() > right_len { assert!(right_slice[right_len] == 57); }
        }
        let mut context = right.proof_take_coordinator();
        if right_first {
            assert!(!right.proof_release(context.borrow_mut(), false));
            assert!(left.proof_release(context.borrow_mut(), true));
        } else {
            assert!(!left.proof_release(context.borrow_mut(), true));
            assert!(right.proof_release(context.borrow_mut(), false));
        }
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(input@.len() >= 1))]
    pub(crate) fn proof_reinitialize_split_prefix(input: Vec<u8>, right_first: bool) {
        let mut right = Self::from_vec(input);
        let mut left = right.split_to(1);
        left.truncate(0);
        left.spare_capacity_mut()[0] = MaybeUninit::uninit();
        proof_assert!(left.proof_view_slot(0int) == Some(None));
        left.proof_initialize_spare_byte(55, true);
        assert!(left.as_slice_mut()[0] == 55);
        let mut context = right.proof_take_coordinator();
        if right_first {
            assert!(!right.proof_release(context.borrow_mut(), false));
            assert!(left.proof_release(context.borrow_mut(), true));
        } else {
            assert!(!left.proof_release(context.borrow_mut(), true));
            assert!(right.proof_release(context.borrow_mut(), false));
        }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), feature = "negative_reuninitialized_growth"))]
    #[cfg_attr(creusot, requires(input@.len() >= 1))]
    pub(crate) fn proof_reject_reuninitialized_growth(input: Vec<u8>) {
        let mut right = Self::from_vec(input);
        let mut left = right.split_to(1);
        left.truncate(0);
        left.spare_capacity_mut()[0] = MaybeUninit::uninit();
        proof_assert!(left.proof_registered_valid() && left.cap == 1usize);
        proof_assert!(left.proof_view_slot(0int) == Some(None));
        // Previously initialized bytes are Unknown after explicit re-uninit.
        unsafe { left.set_len(1); }
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    pub(crate) fn proof_readonly_split(input: Vec<u8>, requested: usize, right_first: bool) {
        let left_value = if input.len() > 0 { input[0] } else { 0 };
        let right_value = if requested < input.len() { input[requested] } else { 0 };
        let mut left = Self::from_vec(input);
        let at = cmp::min(requested, left.capacity());
        let mut right = left.split_off(at);
        {
            let left_read = left.as_slice();
            let left_again = left.as_slice();
            let right_read = right.as_slice();
            if left_read.len() > 0 { assert!(left_read[0] == left_value); }
            if right_read.len() > 0 { assert!(right_read[0] == right_value); }
            let right_write = right.as_slice_mut();
            if right_write.len() > 0 { right_write[0] = 91; }
            // Both shared left borrows remain valid during a disjoint mutation.
            if left_read.len() > 0 {
                assert!(left_read[0] == left_value);
                assert!(left_again[0] == left_value);
            }
        }
        let mut context = right.proof_take_coordinator();
        if right_first {
            assert!(!right.proof_release(context.borrow_mut(), false));
            assert!(left.proof_release(context.borrow_mut(), true));
        } else {
            assert!(!left.proof_release(context.borrow_mut(), true));
            assert!(right.proof_release(context.borrow_mut(), false));
        }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), feature = "negative_read_after_release"))]
    #[cfg_attr(creusot, requires(input@.len() >= 1))]
    pub(crate) fn proof_reject_read_after_release(input: Vec<u8>) {
        let mut right = Self::from_vec(input);
        let left = right.split_to(1);
        let read = left.as_slice();
        let mut context = right.proof_take_coordinator();
        let _ = left.proof_release(context.borrow_mut(), true);
        // This runtime use makes the shared borrow live across owner retirement.
        assert!(read.len() == 1);
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    pub(crate) fn proof_unique_access(input: Vec<u8>, keep: usize, value: u8) {
        let original_len = input.len();
        let mut owner = Self::from_vec(input);
        assert!(owner.as_slice().len() == original_len);
        owner.truncate(keep);
        let old_len = owner.len();
        let grow = old_len < owner.capacity();
        {
            let spare = owner.spare_capacity_mut();
            if grow { spare[0] = MaybeUninit::new(value); }
        }
        if grow { unsafe { owner.set_len(old_len + 1); } }
        if owner.len() > 0 {
            owner.as_slice_mut()[0] = value;
            assert!(owner.as_slice()[0] == value);
        }
        owner.proof_release_unique_at_zero();
    }

    #[cfg(all(any(creusot, bytes_proof_probe), feature = "negative_unique_uninitialized_growth"))]
    #[cfg_attr(creusot, requires(input@.len() >= 1))]
    pub(crate) fn proof_reject_unique_uninitialized_growth(input: Vec<u8>) {
        let mut owner = Self::from_vec(input);
        owner.truncate(0);
        owner.spare_capacity_mut()[0] = MaybeUninit::uninit();
        proof_assert!(owner.proof_owned_valid());
        proof_assert!(owner.proof_view_slot(0int) == Some(None));
        // A unique allocation retains ownership, but re-uninitialization removes
        // the Known-byte authority required to publish this byte again.
        unsafe { owner.set_len(1); }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle))]
    fn proof_generic_as_ref<T: AsRef<[u8]>>(owner: &T) -> Option<u8> {
        let bytes = owner.as_ref();
        if bytes.len() > 0 { Some(bytes[0]) } else { None }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle))]
    fn proof_generic_as_mut<T: AsMut<[u8]>>(owner: &mut T, value: u8) {
        let bytes = owner.as_mut();
        if bytes.len() > 0 { bytes[0] = value; }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle))]
    pub(crate) fn proof_traits_unique(input: Vec<u8>, value: u8) {
        let mut owner = Self::from_vec(input);
        let length = owner.len();
        assert!(AsRef::<[u8]>::as_ref(&owner).len() == length);
        let _ = Self::proof_generic_as_ref(&owner);
        if length > 0 {
            AsMut::<[u8]>::as_mut(&mut owner)[0] = value;
            assert!(AsRef::<[u8]>::as_ref(&owner)[0] == value);
        }
        owner.proof_release_unique_at_zero();
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle, feature = "negative_unregistered_as_ref"))]
    #[cfg_attr(creusot, requires(creusot_std::invariant::inv(owner.ptr)))]
    #[cfg_attr(creusot, requires(owner.len > 0usize && owner.len <= owner.cap))]
    #[cfg_attr(creusot, requires(owner.unique_at_zero.inner_logic() == None && owner.pending_control.inner_logic() == None && owner.shared_registration.inner_logic() == None && owner.shared_context.inner_logic() == None))]
    pub(crate) fn proof_reject_unregistered_as_ref(#[cfg_attr(creusot, creusot::open_inv)] owner: &Self) {
        // Structurally valid pointer metadata alone is not a valid positive-
        // length handle. The real safe trait call must establish its invariant.
        let _ = AsRef::<[u8]>::as_ref(owner);
    }

    // Unique ownership retains the entire original allocation, including any
    // prefix discarded from the visible view. Packed offset and pointer binding
    // describe the same view into that allocation.
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_unique_owned(self) -> bool {
        pearlite! {
            self.data.addr_logic() & KIND_MASK == KIND_VEC &&
            self.pending_control.inner_logic() == None &&
            self.shared_registration.inner_logic() == None &&
            self.shared_context.inner_logic() == None &&
            match self.unique_at_zero.inner_logic() {
                None => false,
                Some((recovery, region)) =>
                    self.ptr.invariant() && self.ptr@ != None &&
                    self.ptr@.unwrap_logic().0 == recovery.namespace() &&
                    self.ptr@.unwrap_logic().1 == recovery.capacity() &&
                    self.ptr@.unwrap_logic().2 == (self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET)@ &&
                    self.ptr@.unwrap_logic().2 + self.cap@ == recovery.capacity() &&
                    recovery.invariant() && region.invariant() &&
                    region.capacity() == recovery.capacity() &&
                    region.namespace() == recovery.namespace() &&
                    region.resource_id() == recovery.namespace() &&
                    region.lo() == 0 && region.hi() == recovery.capacity() &&
                    self.len <= self.cap
            }
        }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_unique_advance))]
    #[cfg_attr(creusot, requires(self.proof_unique_owned()))]
    pub(crate) fn proof_release_unique(mut self) {
        let offset = unsafe { self.get_vec_pos() };
        let base = self.ptr.retreat_within(offset);
        let capacity = self.cap + offset;
        let state = mem::replace(&mut self.unique_at_zero, ghost! { None });
        let capabilities = ghost! { state.into_inner().unwrap() };
        // SAFETY: retreat retains the sealed original provenance; full original
        // capacity and all allocation authority, including the prefix, are held.
        unsafe { crate::ownership_proof::raw_vec::deallocate_bound_vec(base, capacity, capabilities); }
        mem::forget(self);
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_unique_advance))]
    pub(crate) fn proof_unique_advance(input: Vec<u8>, first: usize, second: usize, value: u8) {
        let mut owner = Self::from_vec(input);
        let original_capacity = owner.capacity();
        let original_len = owner.len();
        let first = cmp::min(cmp::min(first, owner.capacity()), crate::capacity_ops::MAX_VEC_POS);
        unsafe { owner.advance_unchecked(first); }
        assert!(owner.len() == original_len.saturating_sub(first));
        assert!(owner.capacity() == original_capacity - first);
        let second = cmp::min(cmp::min(second, owner.capacity()), crate::capacity_ops::MAX_VEC_POS - first);
        unsafe { owner.advance_unchecked(second); }
        assert!(owner.capacity() == original_capacity - first - second);
        assert!(owner.len() == original_len.saturating_sub(first).saturating_sub(second));
        if owner.len() > 0 {
            owner.as_slice_mut()[0] = value;
            assert!(owner.as_slice()[0] == value);
        }
        let old_len = owner.len();
        if old_len < owner.capacity() {
            owner.spare_capacity_mut()[0] = MaybeUninit::new(value);
            unsafe { owner.set_len(old_len + 1); }
            assert!(owner.as_slice()[old_len] == value);
        }
        owner.proof_release_unique();
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_unique_advance, feature = "negative_advanced_uninitialized_growth"))]
    pub(crate) fn proof_reject_unique_advanced_growth() {
        let mut owner = Self::from_vec(Vec::with_capacity(2));
        if owner.capacity() > 1 && crate::capacity_ops::MAX_VEC_POS >= 1 {
            unsafe { owner.advance_unchecked(1); }
            proof_assert!(owner.proof_unique_owned());
            proof_assert!(owner.proof_view_slot(0int) == Some(None));
            // Retaining the original full allocation cannot turn the new
            // visible Unknown slot into an initialized byte.
            unsafe { owner.set_len(1); }
        }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_unique_advance, feature = "negative_unregistered_arc_advance"))]
    pub(crate) fn proof_reject_unregistered_arc_advance() {
        let owner = Self::from_vec(Vec::with_capacity(2));
        let bound = owner.ptr;
        let capacity = owner.cap;
        owner.proof_release_unique_at_zero();
        // A copied sealed descriptor survives as metadata after recovery. It
        // cannot supply the registration required for native pointer addition.
        let mut metadata_only = Self {
            ptr: bound, len: 0, cap: capacity, data: invalid_ptr(KIND_ARC),
            unique_at_zero: ghost! { None }, pending_control: ghost! { None },
            shared_registration: ghost! { None }, shared_context: ghost! { None },
        };
        if capacity > 0 { unsafe { metadata_only.advance_unchecked(1); } }
        mem::forget(metadata_only);
    }

    // This relation frames the native descriptor and authority identities while
    // permitting byte-slot values and the visible length to change.
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_same_storage(self, other: Self) -> bool {
        pearlite! {
            self.ptr == other.ptr && self.cap == other.cap && self.data == other.data &&
            self.pending_control == other.pending_control && self.shared_context == other.shared_context &&
            ((self.unique_at_zero.inner_logic() == None) == (other.unique_at_zero.inner_logic() == None)) &&
            (self.unique_at_zero.inner_logic() != None ==>
                other.unique_at_zero.inner_logic().unwrap_logic().0 == self.unique_at_zero.inner_logic().unwrap_logic().0) &&
            ((self.shared_registration.inner_logic() == None) == (other.shared_registration.inner_logic() == None)) &&
            (self.shared_registration.inner_logic() != None ==>
                other.shared_registration.inner_logic().unwrap_logic().control == self.shared_registration.inner_logic().unwrap_logic().control &&
                other.shared_registration.inner_logic().unwrap_logic().status == self.shared_registration.inner_logic().unwrap_logic().status &&
                other.shared_registration.inner_logic().unwrap_logic().left == self.shared_registration.inner_logic().unwrap_logic().left)
        }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_noalloc))]
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, requires(new_len <= self.cap && extend@.len() <= self.cap@ - new_len@))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized() && self.proof_same_storage(^self)))]
    #[cfg_attr(creusot, ensures((^self).len@ == new_len@ + extend@.len()))]
    fn proof_noalloc_update(&mut self, new_len: usize, value: u8, extend: &[u8]) {
        let old_len = self.len();
        let old_cap = self.capacity();
        let original_first = if old_len > 0 { Some(self.as_slice()[0]) } else { None };
        self.resize(new_len, value);
        assert!(self.len() == new_len && self.capacity() == old_cap);
        if old_len > 0 && new_len > 0 { assert!(self.as_slice()[0] == original_first.unwrap()); }
        if new_len > old_len { assert!(self.as_slice()[new_len - 1] == value); }
        self.extend_from_slice(extend);
        assert!(self.len() == new_len + extend.len() && self.capacity() == old_cap);
        if extend.len() > 0 {
            assert!(self.as_slice()[new_len] == extend[0]);
            assert!(self.as_slice()[self.len() - 1] == extend[extend.len() - 1]);
        }
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_noalloc))]
    pub(crate) fn proof_noalloc_unique(input: Vec<u8>, new_len: usize, value: u8, extend: &[u8]) {
        let mut owner = Self::from_vec(input);
        let new_len = cmp::min(new_len, owner.capacity());
        let count = cmp::min(extend.len(), owner.capacity() - new_len);
        owner.proof_noalloc_update(new_len, value, &extend[..count]);
        owner.proof_release_unique_at_zero();
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_noalloc))]
    pub(crate) fn proof_noalloc_split(input: Vec<u8>, split: usize, new_len: usize, value: u8, extend: &[u8], left_first: bool) {
        let split = cmp::min(split, input.len());
        let mut right = Self::from_vec(input);
        let left = right.split_to(split);
        let left_first_byte = if split > 0 { Some(left.as_slice()[0]) } else { None };
        let new_len = cmp::min(new_len, right.capacity());
        let count = cmp::min(extend.len(), right.capacity() - new_len);
        right.proof_noalloc_update(new_len, value, &extend[..count]);
        assert!(left.len() == split);
        if split > 0 { assert!(left.as_slice()[0] == left_first_byte.unwrap()); }
        let mut context = right.proof_take_coordinator();
        if left_first {
            assert!(!left.proof_release(context.borrow_mut(), true));
            assert!(right.proof_release(context.borrow_mut(), false));
        } else {
            assert!(!right.proof_release(context.borrow_mut(), false));
            assert!(left.proof_release(context.borrow_mut(), true));
        }
    }

    // BEGIN EXACT CARRIER SPLIT METHODS
    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic(prophetic)]
    fn proof_carrier_initialized(self) -> bool {
        pearlite! { self.proof_owned_valid() &&
            forall<index: Int> self.ptr@.unwrap_logic().2 <= index &&
                index < self.ptr@.unwrap_logic().2 + self.len@ ==>
                crate::ownership_proof::raw_vec::slot_known(self.proof_owned_slot(index)) }
    }
    /// A change of coordinates, with the same physical slots and ownership.
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures((*handle).proof_initialized() == (*handle).proof_carrier_initialized()))]
    fn proof_initialization_coordinates(handle: Snapshot<Self>) {
        proof_assert!(forall<index: Int> (*handle).proof_view_slot(index) ==
            (*handle).proof_owned_slot(handle.ptr@.unwrap_logic().2 + index));
        proof_assert!(forall<index: Int> (*handle).proof_owned_slot(index) ==
            (*handle).proof_view_slot(index - handle.ptr@.unwrap_logic().2));
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(self.proof_carrier_pending() && at <= self.len))]
    #[cfg_attr(creusot, ensures((^self).proof_carrier_pair(result)))]
    #[cfg_attr(creusot, ensures(result.ptr@ == self.ptr@ && (^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + at@))))]
    #[cfg_attr(creusot, ensures(result.len == at && (^self).len@ == self.len@ - at@ && result.cap == at && (^self).cap@ == self.cap@ - at@))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized() && result.proof_initialized()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < at@ ==> result.proof_view_slot(index) == self.proof_carrier_slot(self.ptr@.unwrap_logic().2 + index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ - at@ ==> (^self).proof_view_slot(index) == self.proof_carrier_slot(self.ptr@.unwrap_logic().2 + index + at@)))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() && registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((* (^self).shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.pending_control.inner_logic() != None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() && registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) ==>
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != result.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(creusot, requires(other.ptr == self.ptr && other.len == self.len && other.cap == self.cap && other.data == self.data))]
    #[cfg_attr(creusot, requires(other.unique_at_zero.inner_logic() == None && other.pending_control.inner_logic() == None && other.shared_registration.inner_logic() == None && other.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(forall<index: Int> self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@ ==>
        crate::ownership_proof::raw_vec::slot_known(self.proof_carrier_slot(index))))]
    #[cfg_attr(creusot, ensures((^self).proof_carrier_initialized() && result.proof_carrier_initialized()))]
    fn proof_finish_carrier_split(&mut self, mut other: Self, at: usize) -> Self {
        self.proof_register_carrier_split(&mut other, at);
        unsafe { self.advance_unchecked(at); }
        other.cap = at;
        other.len = at;
        ghost! { Self::proof_initialization_coordinates(snapshot!(*self)); };
        ghost! { Self::proof_initialization_coordinates(snapshot!(other)); };
        other
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(self.proof_carrier_pending() && at <= self.cap))]
    #[cfg_attr(creusot, ensures(result.proof_carrier_pair((^self))))]
    #[cfg_attr(creusot, ensures((^self).ptr@ == self.ptr@ && result.ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + at@))))]
    #[cfg_attr(creusot, ensures((^self).len@ == (if self.len < at { self.len@ } else { at@ }) && result.len@ == (if self.len < at { 0int } else { self.len@ - at@ }) && (^self).cap == at && result.cap@ == self.cap@ - at@))]
    #[cfg_attr(creusot, ensures(result.proof_initialized() && (^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < at@ ==> (^self).proof_view_slot(index) == self.proof_carrier_slot(self.ptr@.unwrap_logic().2 + index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.cap@ - at@ ==> result.proof_view_slot(index) == self.proof_carrier_slot(self.ptr@.unwrap_logic().2 + index + at@)))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() && registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches(result.shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((* result.shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.pending_control.inner_logic() != None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() && registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) ==>
            registration.packet.0.logical_id() != result.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(creusot, requires(other.ptr == self.ptr && other.len == self.len && other.cap == self.cap && other.data == self.data))]
    #[cfg_attr(creusot, requires(other.unique_at_zero.inner_logic() == None && other.pending_control.inner_logic() == None && other.shared_registration.inner_logic() == None && other.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(forall<index: Int> self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@ ==>
        crate::ownership_proof::raw_vec::slot_known(self.proof_carrier_slot(index))))]
    #[cfg_attr(creusot, ensures(result.proof_carrier_initialized() && (^self).proof_carrier_initialized()))]
    fn proof_finish_carrier_split_off(&mut self, mut other: Self, at: usize) -> Self {
        // Move the affine state, never copy it with the equal descriptor.
        other.pending_control = ghost! { self.pending_control.take() };
        other.shared_registration = ghost! { self.shared_registration.take() };
        other.shared_context = ghost! { self.shared_context.take() };
        other.proof_register_carrier_split(self, at);
        unsafe { other.advance_unchecked(at); }
        self.cap = at;
        self.len = cmp::min(self.len, at);
        ghost! { Self::proof_initialization_coordinates(snapshot!(*self)); };
        ghost! { Self::proof_initialization_coordinates(snapshot!(other)); };
        other
    }

    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic(prophetic)]
    fn proof_carrier_ready(self) -> bool {
        pearlite! {
            self.proof_carrier_initialized() && (self.proof_unique_at_zero_valid() ||
                match (self.shared_registration.inner_logic(), self.shared_context.inner_logic()) {
                    (Some(registration), Some(context)) => registration.matches(context) &&
                        (*context.status.pending).len() < isize::MAX@,
                    _ => false,
                })
        }
    }
    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic(prophetic)]
    fn proof_carrier_pending(self) -> bool {
        pearlite! {
            self.proof_pending_valid() || (self.proof_registered_valid() &&
                match (self.shared_registration.inner_logic(), self.shared_context.inner_logic()) {
                    (Some(registration), Some(context)) => registration.matches_incremented(context),
                    _ => false,
                })
        }
    }
    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic]
    fn proof_carrier_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.pending_control.inner_logic() {
                Some(pending) => pending.caps.1.slot(index),
                None => self.proof_owned_slot(index),
            }
        }
    }
    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic(prophetic)]
    fn proof_carrier_pair(self, other: Self) -> bool {
        pearlite! {
            self.proof_registered_valid() && other.proof_registered_valid() &&
            other.shared_context.inner_logic() == None &&
            self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() !=
                other.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            match self.shared_context.inner_logic() {
                Some(context) => self.shared_registration.inner_logic().unwrap_logic().matches(context) &&
                    other.shared_registration.inner_logic().unwrap_logic().matches(context),
                None => false,
            }
        }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(self.proof_carrier_pending() && at <= self.cap))]
    #[cfg_attr(creusot, requires(other.ptr == self.ptr && other.len == self.len && other.cap == self.cap && other.data == self.data))]
    #[cfg_attr(creusot, requires(other.unique_at_zero.inner_logic() == None && other.pending_control.inner_logic() == None && other.shared_registration.inner_logic() == None && other.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^other).ptr == other.ptr && (^other).len == other.len && (^other).cap == other.cap && (^other).data == other.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero.inner_logic() == None && (^self).pending_control.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^other).unique_at_zero.inner_logic() == None && (^other).pending_control.inner_logic() == None && (^other).shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).shared_context.inner_logic() != None && (^self).shared_registration.inner_logic() != None && (^other).shared_registration.inner_logic() != None))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().matches((^self).shared_context.inner_logic().unwrap_logic()) &&
        (^other).shared_registration.inner_logic().unwrap_logic().matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().control.pointer == self.data &&
        (^other).shared_registration.inner_logic().unwrap_logic().control.pointer == self.data))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().status.allocation == self.ptr@.unwrap_logic().0 &&
        (^self).shared_registration.inner_logic().unwrap_logic().status.capacity == self.ptr@.unwrap_logic().1 &&
        (^other).shared_registration.inner_logic().unwrap_logic().status.allocation == self.ptr@.unwrap_logic().0 &&
        (^other).shared_registration.inner_logic().unwrap_logic().status.capacity == self.ptr@.unwrap_logic().1))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().packet.1.lo() == self.ptr@.unwrap_logic().2 + at@ &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.ptr@.unwrap_logic().2 + self.cap@))]
    #[cfg_attr(creusot, ensures((^other).shared_registration.inner_logic().unwrap_logic().packet.1.lo() <= self.ptr@.unwrap_logic().2 &&
        (^other).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.ptr@.unwrap_logic().2 + at@))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() !=
        (^other).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + at@ ==>
        (^other).proof_owned_slot(index) == self.proof_carrier_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> self.ptr@.unwrap_logic().2 + at@ <= index && index < self.ptr@.unwrap_logic().2 + self.cap@ ==>
        (^self).proof_owned_slot(index) == self.proof_carrier_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() &&
        registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((* (^self).shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.pending_control.inner_logic() != None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) ==>
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != (^other).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    fn proof_register_carrier_split(&mut self, other: &mut Self, at: usize) {
        let pointer = self.data;
        let base = self.ptr;
        let capacity = self.cap;
        let cut = snapshot!(self.ptr@.unwrap_logic().2 + at@);
        let assigned = ghost! {
            if let Some(pending) = self.pending_control.take() {
                sequential_shared_control::activate(pointer, base, capacity, cut, Ghost::new(pending)).into_inner()
            } else {
                let parent = self.shared_registration.take().unwrap();
                let mut context = self.shared_context.take().unwrap();
                let children = sequential_shared_control::split_more(Ghost::new(&mut context), Ghost::new(parent), cut);
                let (left, right) = children.into_inner();
                (context, left, right)
            }
        };
        let (context, left, right) = assigned.split();
        self.shared_context = ghost! { Some(context.into_inner()) };
        other.shared_registration = ghost! { Some(left.into_inner()) };
        self.shared_registration = ghost! { Some(right.into_inner()) };
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(self.proof_registered_valid() && self.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(self.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic())))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).valid(self.shared_registration.inner_logic().unwrap_logic().control)))]
    #[cfg_attr(creusot, ensures(result == !(^context.inner_logic()).active()))]
    #[cfg_attr(creusot, ensures((* (^context.inner_logic()).status.pending).len() + 1 == (*context.inner_logic().status.pending).len()))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        registration.matches(*context.inner_logic()) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches(^context.inner_logic())))]
    #[cfg_attr(creusot, ensures(result == ((*context.inner_logic().status.pending).len() == 1)))]
    fn proof_carrier_release(mut self, context: Ghost<&mut sequential_shared_control::ControlContext>) -> bool {
        let registration = ghost! { self.shared_registration.take().unwrap() };
        let identity = ghost! { registration.control.identity.into_inner() };
        let control = sequential_shared_control::ControlPtr { pointer: self.data, identity };
        let last = sequential_shared_control::release(control, context, registration);
        mem::forget(self);
        last
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(a.proof_registered_valid() && b.proof_registered_valid() && c.proof_registered_valid()))]
    #[cfg_attr(creusot, requires(a.shared_context.inner_logic() == None && b.shared_context.inner_logic() == None && c.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(a.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic()) &&
        b.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic()) &&
        c.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic())))]
    #[cfg_attr(creusot, requires(a.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() != b.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
        a.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() != c.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
        b.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() != c.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(creusot, requires((*context.inner_logic().status.pending).len() == 3))]
    fn proof_carrier_release_three(a: Self, b: Self, c: Self,
        mut context: Ghost<&mut sequential_shared_control::ControlContext>, order: u8) {
        let (first, second, third) = match order % 6 {
            0 => (a, b, c), 1 => (a, c, b), 2 => (b, a, c),
            3 => (b, c, a), 4 => (c, a, b), _ => (c, b, a),
        };
        assert!(!first.proof_carrier_release(ghost! { &mut **context }));
        assert!(!second.proof_carrier_release(ghost! { &mut **context }));
        assert!(third.proof_carrier_release(context));
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    pub(crate) fn proof_carrier_split(input: Vec<u8>, first: usize, second: usize, order: u8, value: u8) {
        let mut carrier = Self::from_vec(input);
        ghost! { Self::proof_initialization_coordinates(snapshot!(carrier)); };
        let first = cmp::min(first, carrier.len());
        let mut left = carrier.split_to(first);
        let second = cmp::min(second, carrier.len());
        let mut middle = carrier.split_to(second);
        if left.len() > 0 { left.as_slice_mut()[0] = value; assert!(left.as_slice()[0] == value); }
        if middle.len() > 0 { middle.as_slice_mut()[0] = value; assert!(middle.as_slice()[0] == value); }
        if carrier.len() > 0 { carrier.as_slice_mut()[0] = value; assert!(carrier.as_slice()[0] == value); }
        let mut context = carrier.proof_take_coordinator();
        Self::proof_carrier_release_three(left, middle, carrier, context.borrow_mut(), order);
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    pub(crate) fn proof_carrier_split_off(input: Vec<u8>, first: usize, second: usize, order: u8, value: u8) {
        let mut carrier = Self::from_vec(input);
        ghost! { Self::proof_initialization_coordinates(snapshot!(carrier)); };
        let first = cmp::min(first, carrier.len());
        let mut left = carrier.split_to(first);
        let second = cmp::min(second, carrier.capacity());
        let mut right = carrier.split_off(second);
        if left.len() > 0 { left.as_slice_mut()[0] = value; assert!(left.as_slice()[0] == value); }
        if right.len() > 0 { right.as_slice_mut()[0] = value; assert!(right.as_slice()[0] == value); }
        if carrier.len() > 0 { carrier.as_slice_mut()[0] = value; assert!(carrier.as_slice()[0] == value); }
        let mut context = right.proof_take_coordinator();
        Self::proof_carrier_release_three(left, carrier, right, context.borrow_mut(), order);
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split, feature = "negative_carrier_missing_ticket"))]
    pub(crate) fn proof_carrier_missing_empty_ticket() {
        let mut carrier = Self::from_vec(Vec::new());
        ghost! { Self::proof_initialization_coordinates(snapshot!(carrier)); };
        let left = carrier.split_to(0);
        let middle = carrier.split_to(0);
        let mut context = carrier.proof_take_coordinator();
        mem::forget(left);
        assert!(!middle.proof_carrier_release(context.borrow_mut()));
        // The unreleased empty left handle still owns one affine ticket.
        assert!(carrier.proof_carrier_release(context.borrow_mut()));
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split, feature = "negative_carrier_unknown_publication"))]
    pub(crate) fn proof_carrier_unknown_publication() {
        let mut carrier = Self::from_vec(Vec::with_capacity(2));
        ghost! { Self::proof_initialization_coordinates(snapshot!(carrier)); };
        if carrier.capacity() > 1 {
            let _left = carrier.split_to(0);
            let mut right = carrier.split_off(1);
            assert!(right.len() == 0);
            // The right capacity contains original Unknown slots; no write occurred.
            // Publishing one of them must fail set_len's initialized-byte guard.
            unsafe { right.set_len(1); }
        }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    pub(crate) fn proof_carrier_storage(input: Vec<u8>, first: usize, second: usize, order: u8, value: u8) {
        let mut middle = Self::from_vec(input);
        ghost! { Self::proof_initialization_coordinates(snapshot!(middle)); };
        let mut left = middle.split_to(cmp::min(first, middle.len()));
        let mut right = middle.split_off(cmp::min(second, middle.capacity()));
        let old_len = right.len();
        if old_len < right.capacity() {
            {
                let spare = right.spare_capacity_mut();
                spare[0] = MaybeUninit::uninit();
                spare[0].write(value);
            }
            // Only the just-written spare byte extends the Known prefix.
            unsafe { right.set_len(old_len + 1); }
            assert!(right.as_slice()[old_len] == value);
        }
        let left_count = cmp::min(value as usize, left.capacity());
        let middle_count = cmp::min(value as usize, middle.capacity());
        let right_count = cmp::min(value as usize, right.capacity());
        let left_len = left.len();
        let middle_len = middle.len();
        let right_len = right.len();
        unsafe {
            left.advance_unchecked(left_count);
            middle.advance_unchecked(middle_count);
            right.advance_unchecked(right_count);
        }
        assert!(left.len() == left_len.saturating_sub(left_count));
        assert!(middle.len() == middle_len.saturating_sub(middle_count));
        assert!(right.len() == right_len.saturating_sub(right_count));
        left.truncate(first);
        middle.truncate(second);
        right.clear();
        assert!(right.len() == 0);
        let mut context = right.proof_take_coordinator();
        Self::proof_carrier_release_three(left, middle, right, context.borrow_mut(), order);
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    pub(crate) fn proof_carrier_capacity_ops(input: Vec<u8>, first: usize, second: usize, order: u8, value: u8) {
        let mut middle = Self::from_vec(input);
        ghost! { Self::proof_initialization_coordinates(snapshot!(middle)); };
        let mut left = middle.split_to(cmp::min(first, middle.len()));
        let mut right = middle.split_off(cmp::min(second, middle.capacity()));
        left.resize(cmp::min(first, left.capacity()), value);
        middle.resize(cmp::min(value as usize, middle.capacity()), value);
        right.resize(cmp::min(second, right.capacity()), value);
        if right.len() < right.capacity() {
            let old_len = right.len();
            right.extend_from_slice(&[value]);
            assert!(right.as_slice()[old_len] == value);
        }
        let mut context = right.proof_take_coordinator();
        Self::proof_carrier_release_three(left, middle, right, context.borrow_mut(), order);
    }
    // END EXACT CARRIER SPLIT METHODS

    // END EXACT BYTESMUT SEQUENTIAL SPLIT METHODS

    // A restricted predicate for freshly constructed unique handles. It is
    // intentionally not the type invariant for arbitrary BytesMut values.
    #[cfg(creusot)]
    #[logic(prophetic)]
    pub(crate) fn proof_unique_at_zero_owned(self) -> bool {
        pearlite! {
            self.data.addr_logic() & KIND_MASK == KIND_VEC &&
            self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize &&
            self.pending_control.inner_logic() == None &&
            self.shared_registration.inner_logic() == None &&
            self.shared_context.inner_logic() == None &&
            match self.unique_at_zero.inner_logic() {
                None => false,
                Some((recovery, region)) =>
                    self.ptr.invariant() &&
                    self.ptr@ == Some((recovery.namespace(), self.cap@, 0int)) &&
                    recovery.invariant() && region.invariant() &&
                    recovery.capacity() == self.cap@ &&
                    region.capacity() == self.cap@ &&
                    region.namespace() == recovery.namespace() &&
                    region.resource_id() == recovery.namespace() &&
                    region.lo() == 0 && region.hi() == self.cap@ &&
                    self.len@ <= self.cap@
            }
        }
    }

    #[cfg(creusot)]
    #[logic(prophetic)]
    pub(crate) fn proof_unique_at_zero_valid(self) -> bool {
        pearlite! {
            self.proof_unique_at_zero_owned() &&
            forall<index: Int> 0 <= index && index < self.len@ ==>
                crate::ownership_proof::raw_vec::slot_known(self.proof_unique_slot(index))
        }
    }

    #[cfg(creusot)]
    #[logic]
    pub(crate) fn proof_unique_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                None => None,
                Some((_, region)) => region.slot(index),
            }
        }
    }

    // Explicit restricted proof path. Forgetting the handle after B3 prevents
    // its ordinary destructor from freeing the same native allocation again.
    // This does not establish scope-exit Drop or a Shared cleanup protocol.
    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(self.proof_unique_at_zero_owned()))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    pub(crate) fn proof_release_unique_at_zero(mut self) {
        let state = mem::replace(&mut self.unique_at_zero, ghost! { None });
        let capabilities = ghost! { state.into_inner().unwrap() };
        // SAFETY: the predicate provides exact offset-zero full authority.
        unsafe {
            crate::ownership_proof::raw_vec::deallocate_bound_vec(
                self.ptr, self.cap, capabilities,
            );
        }
        #[cfg(bytes_proof_valid_handle)]
        {
            // Authority has already been consumed. Normalize individual fields;
            // assigning a whole handle here would run the old armed destructor.
            proof_assert!(self.unique_at_zero.inner_logic() == None && self.pending_control.inner_logic() == None && self.shared_registration.inner_logic() == None && self.shared_context.inner_logic() == None);
            self.ptr = crate::ownership_proof::raw_vec::BoundPtr::unbound(NonNull::dangling());
            self.len = 0;
            self.cap = 0;
            self.data = invalid_ptr(KIND_VEC);
        }
        mem::forget(self);
    }

    // BEGIN EXACT AS_SLICE
    #[inline]
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result@.len() == self.len@))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        self.proof_view_slot(index) == Some(Some(result@[index]))))]
    fn as_slice(&self) -> &[u8] {
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        { unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) } }
        #[cfg(any(creusot, bytes_proof_probe))]
        {
            #[cfg(bytes_proof_valid_handle)]
            if self.len == 0 {
                return unsafe { crate::ownership_proof::raw_vec::borrow_empty_bound(&self.ptr) };
            }
            if self.kind() == KIND_VEC {
                return unsafe { crate::ownership_proof::raw_vec::borrow_bound(
                    &self.ptr, self.len, ghost! { &self.unique_at_zero.as_ref().unwrap().1 }) };
            }
            let (packet, status, left) = ghost! {
                let registration = self.shared_registration.as_ref().unwrap();
                (&registration.packet, registration.status, registration.left)
            }.split();
            unsafe { crate::ownership_proof::shared_protocol::borrow_packet(&self.ptr, self.len, packet, status, left) }
        }
    }

    // BEGIN EXACT AS_SLICE_MUT
    #[inline]
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result@.len() == self.len@ && (^result)@.len() == self.len@))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        self.proof_view_slot(index) == Some(Some(result@[index]))))]
    #[cfg_attr(creusot, ensures((^self).proof_owned_valid()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < (^self).len@ ==>
        crate::ownership_proof::raw_vec::slot_known((^self).proof_view_slot(index))))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures(((^self).unique_at_zero.inner_logic() == None) == (self.unique_at_zero.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures(self.unique_at_zero.inner_logic() != None ==> (^self).unique_at_zero.inner_logic().unwrap_logic().0 == self.unique_at_zero.inner_logic().unwrap_logic().0))]
    #[cfg_attr(creusot, ensures(((^self).shared_registration.inner_logic() == None) == (self.shared_registration.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures((^self).pending_control == self.pending_control && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().control == self.shared_registration.inner_logic().unwrap_logic().control))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().status == self.shared_registration.inner_logic().unwrap_logic().status))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().left == self.shared_registration.inner_logic().unwrap_logic().left))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        (^self).proof_view_slot(index) == Some(Some((^result)@[index]))))]
    #[cfg_attr(creusot, ensures(forall<index: Int> !(0 <= index && index < self.len@) ==>
        (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        !(self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@) ==>
            (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_registration.inner_logic() != None ==>
        (^self).shared_registration.inner_logic().unwrap_logic().packet.0 == self.shared_registration.inner_logic().unwrap_logic().packet.0 &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.lo() == self.shared_registration.inner_logic().unwrap_logic().packet.1.lo() &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.shared_registration.inner_logic().unwrap_logic().packet.1.hi()))]
    fn as_slice_mut(&mut self) -> &mut [u8] {
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        { unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) } }
        #[cfg(any(creusot, bytes_proof_probe))]
        {
            #[cfg(bytes_proof_valid_handle)]
            if self.len == 0 {
                return unsafe { crate::ownership_proof::raw_vec::borrow_empty_bound_mut(&self.ptr) };
            }
            if self.kind() == KIND_VEC {
                return unsafe { crate::ownership_proof::raw_vec::borrow_bound_mut(
                    &self.ptr, self.len, ghost! { &mut self.unique_at_zero.as_mut().unwrap().1 }) };
            }
            let (packet, status, left) = ghost! {
                let registration = self.shared_registration.as_mut().unwrap();
                (&mut registration.packet, registration.status, registration.left)
            }.split();
            unsafe { crate::ownership_proof::shared_protocol::borrow_packet_mut(&self.ptr, self.len, packet, status, left) }
        }
    }

    /// Advance the buffer without bounds checking.
    ///
    /// # SAFETY
    ///
    /// The caller must ensure that `count` <= `self.cap`.
    // BEGIN EXACT ADVANCE_UNCHECKED
    #[cfg_attr(creusot, requires(self.ptr.invariant() && self.ptr@ != None))]
    #[cfg_attr(creusot, requires(self.ptr@.unwrap_logic().2 + self.cap@ <= self.ptr@.unwrap_logic().1))]
    #[cfg_attr(creusot, requires(count <= self.cap && self.len <= self.cap))]
    #[cfg_attr(creusot, requires((self.data.addr_logic() & KIND_MASK == KIND_ARC && self.proof_shared_allocation_registered()) ||
        (self.proof_unique_owned() && self.ptr@.unwrap_logic().2 + count@ <= crate::capacity_ops::MAX_VEC_POS@)))]
    #[cfg_attr(creusot, ensures((^self).ptr.invariant()))]
    #[cfg_attr(creusot, ensures((^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + count@))))]
    #[cfg_attr(creusot, ensures((^self).len@ == (if count <= self.len { self.len@ - count@ } else { 0int }) && (^self).cap@ == self.cap@ - count@))]
    #[cfg_attr(creusot, ensures(self.data.addr_logic() & KIND_MASK == KIND_ARC ==> (^self).data == self.data))]
    #[cfg_attr(creusot, ensures(self.proof_unique_owned() ==> (^self).proof_unique_owned()))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK == self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures(forall<index: Int> (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    #[cfg_attr(creusot, ensures(self.proof_registered_valid() ==> (^self).proof_registered_valid()))]
    #[cfg_attr(creusot, ensures(self.proof_initialized() ==> (^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        (^self).proof_view_slot(index) == self.proof_view_slot(index + count@)))]
    pub(crate) unsafe fn advance_unchecked(&mut self, count: usize) {
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
            let pos = self.get_vec_pos() + count;

            if pos <= MAX_VEC_POS {
                self.set_vec_pos(pos);
            } else {
                // The repr must be upgraded to ARC. This will never happen
                // on 64 bit systems and will only happen on 32 bit systems
                // when shifting past 134,217,727 bytes. As such, we don't
                // worry too much about performance here.
                #[cfg(not(any(creusot, bytes_proof_probe)))]
                self.promote_to_shared(/*ref_count = */ 1);
                #[cfg(any(creusot, bytes_proof_probe))]
                panic!("unique offset overflow promotion is outside this proof gate");
            }
        }

        // Updating the start of the view is setting `ptr` to point to the
        // new start and updating the `len` field to reflect the new length
        // of the view.
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        { self.ptr = vptr(self.ptr.as_ptr().add(count)); }
        #[cfg(any(creusot, bytes_proof_probe))]
        { self.ptr = self.ptr.advance_within(count); }
        self.len = self.len.saturating_sub(count);
        self.cap -= count;
    }

    fn try_unsplit(&mut self, other: BytesMut) -> Result<(), BytesMut> {
        if other.capacity() == 0 {
            return Ok(());
        }

        let ptr = unsafe { self.ptr.as_ptr().add(self.len) };
        if crate::provenance_specs::pointer_addr_eq(ptr, other.ptr.as_ptr())
            && self.kind() == KIND_ARC
            && other.kind() == KIND_ARC
            && crate::provenance_specs::pointer_addr_eq(self.data, other.data)
        {
            // Contiguous blocks, just combine directly
            self.len += other.len;
            self.cap += other.cap;
            Ok(())
        } else {
            Err(other)
        }
    }

    #[inline]
    #[cfg_attr(creusot, ensures(result == (self.data.addr_logic() & KIND_MASK)))]
    fn kind(&self) -> usize {
        crate::provenance_specs::pointer_addr(self.data) & KIND_MASK
    }

    // BEGIN EXACT PROMOTE_TO_SHARED
    #[cfg_attr(creusot, requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(creusot, requires(ref_cnt == 2usize))]
    #[cfg_attr(creusot, ensures((^self).proof_pending_valid()))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(creusot, ensures((^self).pending_control.inner_logic().unwrap_logic().caps == self.unique_at_zero.inner_logic().unwrap_logic()))]
    unsafe fn promote_to_shared(&mut self, ref_cnt: usize) {
        debug_assert_eq!(self.kind(), KIND_VEC);
        debug_assert!(ref_cnt == 1 || ref_cnt == 2);

        let original_capacity_repr =
            crate::capacity_ops::original_capacity_repr_from_data(
                crate::provenance_specs::pointer_addr(self.data),
            );

        // The vec offset cannot be concurrently mutated, so there
        // should be no danger reading it.
        let off = crate::capacity_ops::vec_pos_from_data(
            crate::provenance_specs::pointer_addr(self.data),
        );

        // First, allocate a new `Shared` instance containing the
        // `Vec` fields. It's important to note that `ptr`, `len`,
        // and `cap` cannot be mutated without having `&mut self`.
        // This means that these fields will not be concurrently
        // updated and since the buffer hasn't been promoted to an
        // `Arc`, those three fields still are the components of the
        // vector.
        #[cfg(any(creusot, bytes_proof_probe))]
        let (ref_count, counter_own) = crate::ownership_proof::sequential_counter::SequentialCounter::new(ref_cnt);
        let shared = Box::new(Shared {
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            buffer: SharedBuffer::from_handle(self.ptr.as_ptr(), self.cap, off),
            #[cfg(any(creusot, bytes_proof_probe))]
            buffer: SharedBuffer { base: self.ptr, capacity: self.cap },
            original_capacity_repr,
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            ref_count: AtomicUsize::new(ref_cnt),
            #[cfg(any(creusot, bytes_proof_probe))]
            ref_count,
        });

        #[cfg(not(any(creusot, bytes_proof_probe)))]
        let shared = Box::into_raw(shared);
        #[cfg(any(creusot, bytes_proof_probe))]
        let (shared, owner) = crate::ownership_proof::boxed_alignment::into_raw_aligned(shared);

        #[cfg(any(creusot, bytes_proof_probe))]
        crate::ownership_proof::boxed_alignment::aligned_address_has_clear_low_bit(
            crate::provenance_specs::pointer_addr(shared), core::mem::align_of::<Shared>(),
        );

        // The pointer should be aligned, so this assert should
        // always succeed.
        debug_assert_eq!(
            crate::provenance_specs::pointer_addr(shared) & KIND_MASK,
            KIND_ARC
        );

        self.data = shared;
        #[cfg(any(creusot, bytes_proof_probe))]
        {
            self.pending_control = ghost! {
                let caps = self.unique_at_zero.take().unwrap();
                Some(sequential_shared_control::PendingControl { counter: counter_own.into_inner(), owner: owner.into_inner(), caps })
            };
        }
    }

    /// Makes an exact shallow clone of `self`.
    ///
    /// The kind of `self` doesn't matter, but this is unsafe
    /// because the clone will have the same offsets. You must
    /// be sure the returned value to the user doesn't allow
    /// two views into the same range.
    // BEGIN EXACT SHALLOW_CLONE
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).proof_pending_valid()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.ptr == (^self).ptr && result.len == (^self).len && result.cap == (^self).cap && result.data == (^self).data))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.unique_at_zero.inner_logic() == None && result.pending_control.inner_logic() == None && result.shared_registration.inner_logic() == None && result.shared_context.inner_logic() == None))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).pending_control.inner_logic().unwrap_logic().caps == self.unique_at_zero.inner_logic().unwrap_logic()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(self.proof_carrier_ready()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).proof_carrier_pending()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(((^self).shared_context.inner_logic() == None) == (self.shared_context.inner_logic() == None)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_context.inner_logic() == None ==> (^self).pending_control.inner_logic() != None))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.ptr == (^self).ptr && result.len == (^self).len && result.cap == (^self).cap && result.data == (^self).data))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.unique_at_zero.inner_logic() == None && result.pending_control.inner_logic() == None && result.shared_registration.inner_logic() == None && result.shared_context.inner_logic() == None))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> (^self).proof_carrier_slot(index) == self.proof_owned_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@ ==>
        crate::ownership_proof::raw_vec::slot_known((^self).proof_carrier_slot(index))))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_context.inner_logic() != None ==> (^self).shared_registration == self.shared_registration &&
        (^self).shared_context.inner_logic().unwrap_logic().status == self.shared_context.inner_logic().unwrap_logic().status))]
    #[inline]
    unsafe fn shallow_clone(&mut self) -> BytesMut {
        if self.kind() == KIND_ARC {
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            increment_shared(self.data);
            #[cfg(all(any(creusot, bytes_proof_probe), not(bytes_proof_repeated_split)))]
            panic!("ARC clone is outside the first sequential split gate");
            #[cfg(bytes_proof_repeated_split)]
            {
                let identity = ghost! { self.shared_registration.as_ref().unwrap().control.identity.into_inner() };
                let control = sequential_shared_control::ControlPtr { pointer: self.data, identity };
                increment_shared(self.data, control, ghost! { self.shared_context.as_mut().unwrap() });
            }
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            { ptr::read(self) }
            #[cfg(any(creusot, bytes_proof_probe))]
            { self.pending_descriptor_copy() }
        } else {
            self.promote_to_shared(/*ref_count = */ 2);
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            { ptr::read(self) }
            #[cfg(any(creusot, bytes_proof_probe))]
            { self.pending_descriptor_copy() }
        }
    }

    // Copies metadata only. The completed split protocol must distribute
    // regions before this pending descriptor can become a verified handle.
    #[cfg_attr(creusot, ensures(result.ptr == self.ptr && result.len == self.len && result.cap == self.cap && result.data == self.data))]
    #[cfg_attr(creusot, ensures(result.unique_at_zero.inner_logic() == None && result.pending_control.inner_logic() == None && result.shared_registration.inner_logic() == None && result.shared_context.inner_logic() == None))]
    #[cfg(any(creusot, bytes_proof_probe))]
    fn pending_descriptor_copy(&self) -> BytesMut {
        BytesMut {
            ptr: self.ptr, len: self.len, cap: self.cap, data: self.data,
            unique_at_zero: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            pending_control: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_registration: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_context: ghost! { None },
        }
    }

    // BEGIN EXACT GET_VEC_POS
    #[inline]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, ensures(result == self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET))]
    unsafe fn get_vec_pos(&self) -> usize {
        debug_assert_eq!(self.kind(), KIND_VEC);

        crate::capacity_ops::vec_pos_from_data(
            crate::provenance_specs::pointer_addr(self.data),
        )
    }

    // BEGIN EXACT SET_VEC_POS
    #[inline]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, requires(pos <= crate::capacity_ops::MAX_VEC_POS))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == pos))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK == self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    unsafe fn set_vec_pos(&mut self, pos: usize) {
        debug_assert_eq!(self.kind(), KIND_VEC);
        debug_assert!(pos <= MAX_VEC_POS);

        self.data = invalid_ptr(crate::capacity_ops::set_vec_pos_in_data(
            crate::provenance_specs::pointer_addr(self.data),
            pos,
        ));
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
    // BEGIN EXACT SPARE_CAPACITY_MUT
    #[inline]
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result@.len() == self.cap@ - self.len@ && (^result)@.len() == self.cap@ - self.len@))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < result@.len() ==>
        self.proof_view_slot(self.len@ + index) == Some(result@[index]@)))]
    // Absolute visible indices let publication and frame clients use the same
    // slot term without reconstructing the spare slice's relative index.
    #[cfg_attr(creusot, ensures(forall<index: Int> self.len@ <= index && index < self.cap@ ==>
        self.proof_view_slot(index) == Some(result@[index - self.len@]@)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> self.len@ <= index && index < self.cap@ ==>
        (^self).proof_view_slot(index) == Some((^result)@[index - self.len@]@)))]
    #[cfg_attr(creusot, ensures((^self).proof_owned_valid()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < (^self).len@ ==>
        crate::ownership_proof::raw_vec::slot_known((^self).proof_view_slot(index))))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures(((^self).unique_at_zero.inner_logic() == None) == (self.unique_at_zero.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures(self.unique_at_zero.inner_logic() != None ==> (^self).unique_at_zero.inner_logic().unwrap_logic().0 == self.unique_at_zero.inner_logic().unwrap_logic().0))]
    #[cfg_attr(creusot, ensures(((^self).shared_registration.inner_logic() == None) == (self.shared_registration.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures((^self).pending_control == self.pending_control && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().control == self.shared_registration.inner_logic().unwrap_logic().control))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().status == self.shared_registration.inner_logic().unwrap_logic().status))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().left == self.shared_registration.inner_logic().unwrap_logic().left))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < (^result)@.len() ==>
        (^self).proof_view_slot(self.len@ + index) == Some((^result)@[index]@)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> !(self.len@ <= index && index < self.cap@) ==>
        (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        !(self.ptr@.unwrap_logic().2 + self.len@ <= index && index < self.ptr@.unwrap_logic().2 + self.cap@) ==>
            (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_registration.inner_logic() != None ==>
        (^self).shared_registration.inner_logic().unwrap_logic().packet.0 == self.shared_registration.inner_logic().unwrap_logic().packet.0 &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.lo() == self.shared_registration.inner_logic().unwrap_logic().packet.1.lo() &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.shared_registration.inner_logic().unwrap_logic().packet.1.hi()))]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        unsafe {
            let ptr = self.ptr.as_ptr().add(self.len);
            let len = self.cap - self.len;
            slice::from_raw_parts_mut(ptr.cast(), len)
        }
        #[cfg(any(creusot, bytes_proof_probe))]
        {
            #[cfg(bytes_proof_valid_handle)]
            if self.cap == 0 {
                return unsafe { crate::ownership_proof::raw_vec::borrow_empty_bound_uninit_mut(&self.ptr) };
            }
            if self.kind() == KIND_VEC {
                return unsafe { crate::ownership_proof::raw_vec::borrow_spare_preserving_prefix(
                    self.ptr, self.len, self.cap,
                    ghost! { &mut self.unique_at_zero.as_mut().unwrap().1 }) };
            }
            let (packet, status, left) = ghost! {
                let registration = self.shared_registration.as_mut().unwrap();
                (&mut registration.packet, registration.status, registration.left)
            }.split();
            unsafe { crate::ownership_proof::shared_protocol::borrow_packet_uninit_mut(self.ptr, self.len, self.cap, packet, status, left) }
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
    fn advance(&mut self, cnt: usize) {
        assert!(
            cnt <= self.len,
            "cannot advance past `remaining`: {:?} <= {:?}",
            cnt,
            self.len,
        );
        unsafe {
            // SAFETY: We've checked that `cnt` <= `self.len` and we know that
            // `self.len` <= `self.cap`.
            self.advance_unchecked(cnt);
        }
    }

    fn copy_to_bytes(&mut self, len: usize) -> Bytes {
        self.split_to(len).freeze()
    }
}

unsafe impl BufMut for BytesMut {
    #[inline]
    fn remaining_mut(&self) -> usize {
        // Max allocation size is isize::MAX.
        isize::MAX as usize - self.len()
    }

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
        self.len = self.len() + cnt;
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
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(result@.len() == self.len@))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        self.proof_view_slot(index) == Some(Some(result@[index]))))]
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Deref for BytesMut {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsMut<[u8]> for BytesMut {
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(result@.len() == self.len@ && (^result)@.len() == self.len@))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        self.proof_view_slot(index) == Some(Some(result@[index]))))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures((^self).proof_owned_valid()))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> 0 <= index && index < (^self).len@ ==>
        crate::ownership_proof::raw_vec::slot_known((^self).proof_view_slot(index))))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(((^self).unique_at_zero.inner_logic() == None) == (self.unique_at_zero.inner_logic() == None)))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.unique_at_zero.inner_logic() != None ==> (^self).unique_at_zero.inner_logic().unwrap_logic().0 == self.unique_at_zero.inner_logic().unwrap_logic().0))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(((^self).shared_registration.inner_logic() == None) == (self.shared_registration.inner_logic() == None)))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures((^self).pending_control == self.pending_control && (^self).shared_context == self.shared_context))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().control == self.shared_registration.inner_logic().unwrap_logic().control))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().status == self.shared_registration.inner_logic().unwrap_logic().status))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().left == self.shared_registration.inner_logic().unwrap_logic().left))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        (^self).proof_view_slot(index) == Some(Some((^result)@[index]))))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> !(0 <= index && index < self.len@) ==>
        (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int>
        !(self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@) ==>
            (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl DerefMut for BytesMut {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl<'a> From<&'a [u8]> for BytesMut {
    #[cfg_attr(creusot, ensures(result.proof_unique_at_zero_valid()))]
    #[cfg_attr(creusot, ensures(result.len@ == src@.len()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < result.len@ ==>
        result.proof_unique_slot(index) == Some(Some(src@[index]))))]
    fn from(src: &'a [u8]) -> BytesMut {
        BytesMut::from_vec(src.to_vec())
    }
}

impl<'a> From<&'a str> for BytesMut {
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
mod runtime_self_comparisons {
use super::*;

impl PartialEq for BytesMut {
    fn eq(&self, other: &BytesMut) -> bool {
        crate::comparison_ops::equal(self.as_slice(), other.as_slice())
    }
}

impl PartialOrd for BytesMut {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BytesMut {
    fn cmp(&self, other: &BytesMut) -> cmp::Ordering {
        crate::comparison_ops::compare(self.as_slice(), other.as_slice())
    }
}

impl Eq for BytesMut {}

}

/// Proof-only adapter bodies over the byte slice exposed by this handle. They
/// delegate to `comparison_ops`, but have no semantic postcondition because
/// `as_slice` is a program operation over raw storage and there is no logical
/// `BytesMut` view available to state one soundly.
#[cfg(creusot)]
impl BytesMut {
    /// Compare this mutable handle's exposed bytes with another `BytesMut` view.
    #[doc(hidden)]
    pub fn __creusot_eq_bytes_mut(&self, other: &BytesMut) -> bool {
        crate::comparison_ops::equal(self.as_slice(), other.as_slice())
    }

    /// Compare this mutable handle's exposed bytes lexicographically with another `BytesMut` view.
    #[doc(hidden)]
    pub fn __creusot_cmp_bytes_mut(&self, other: &BytesMut) -> cmp::Ordering {
        crate::comparison_ops::compare(self.as_slice(), other.as_slice())
    }

    /// Compare this mutable handle's exposed bytes with a borrowed byte slice.
    #[doc(hidden)]
    pub fn __creusot_eq_slice(&self, other: &[u8]) -> bool {
        crate::comparison_ops::equal(self.as_slice(), other)
    }

    /// Compare this mutable handle's exposed bytes lexicographically with a borrowed byte slice.
    #[doc(hidden)]
    pub fn __creusot_cmp_slice(&self, other: &[u8]) -> cmp::Ordering {
        crate::comparison_ops::compare(self.as_slice(), other)
    }

    /// Compare this mutable handle's exposed bytes with a string's UTF-8 bytes.
    #[doc(hidden)]
    pub fn __creusot_eq_str(&self, other: &str) -> bool {
        crate::comparison_ops::equal(self.as_slice(), other.as_bytes())
    }

    /// Compare this mutable handle's exposed bytes lexicographically with a string's UTF-8 bytes.
    #[doc(hidden)]
    pub fn __creusot_cmp_str(&self, other: &str) -> cmp::Ordering {
        crate::comparison_ops::compare(self.as_slice(), other.as_bytes())
    }

    /// Compare this mutable handle's exposed bytes with a frozen `Bytes` view.
    #[doc(hidden)]
    pub fn __creusot_eq_bytes(&self, other: &Bytes) -> bool {
        crate::comparison_ops::equal(self.as_slice(), other.as_ref())
    }
}

impl Default for BytesMut {
    #[inline]
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

impl Borrow<[u8]> for BytesMut {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

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
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a u8>,
    {
        self.extend(iter.into_iter().copied())
    }
}

impl Extend<Bytes> for BytesMut {
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
    fn from_iter<T: IntoIterator<Item = u8>>(into_iter: T) -> Self {
        BytesMut::from_vec(Vec::from_iter(into_iter))
    }
}

impl<'a> FromIterator<&'a u8> for BytesMut {
    fn from_iter<T: IntoIterator<Item = &'a u8>>(into_iter: T) -> Self {
        BytesMut::from_iter(into_iter.into_iter().copied())
    }
}

/*
 *
 * ===== Inner =====
 *
 */

// BEGIN EXACT INCREMENT_SHARED
#[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(ptr == control.pointer))]
#[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(context.inner_logic().valid(control) && context.inner_logic().active()))]
#[cfg_attr(all(creusot, bytes_proof_repeated_split), requires((*context.inner_logic().status.pending).len() < isize::MAX@))]
#[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^context.inner_logic()).valid_count(control, 1) && (^context.inner_logic()).active()))]
#[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^context.inner_logic()).status == context.inner_logic().status))]
unsafe fn increment_shared(ptr: *mut Shared,
    #[cfg(bytes_proof_repeated_split)] control: sequential_shared_control::ControlPtr,
    #[cfg(bytes_proof_repeated_split)] context: Ghost<&mut sequential_shared_control::ControlContext>,
) {
    #[cfg(not(bytes_proof_repeated_split))]
    let old_size = (*ptr).ref_count.fetch_add(1, Ordering::Relaxed);
    #[cfg(bytes_proof_repeated_split)]
    let old_size = sequential_shared_control::increment(control, context);
    if old_size > isize::MAX as usize {
        crate::abort();
    }
}

unsafe fn release_shared(ptr: *mut Shared) {
    // `Shared` storage... follow the drop steps from Arc.
    if (*ptr).ref_count.fetch_sub(1, Ordering::Release) != 1 {
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
    (*ptr).ref_count.load(Ordering::Acquire);

    // Drop the data
    drop(Box::from_raw(ptr));
}

impl Shared {
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
        self.ref_count.load(Ordering::Acquire) == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

// The proof representation carries exclusive sequential counter authority.
// Native thread-safety remains unchanged; no concurrent proof is claimed.
#[cfg(not(any(creusot, bytes_proof_probe)))]
unsafe impl Send for BytesMut {}
#[cfg(not(any(creusot, bytes_proof_probe)))]
unsafe impl Sync for BytesMut {}

/*
 *
 * ===== PartialEq / PartialOrd =====
 *
 */

// Standard comparison contracts require a DeepModel for BytesMut. Until the
// pointer-range ownership invariant supplies that model, these exact upstream
// runtime impls stay outside the proof configuration.
#[cfg(not(creusot))]
mod runtime_cross_comparisons {
use super::*;

impl PartialEq<[u8]> for BytesMut {
    fn eq(&self, other: &[u8]) -> bool {
        &**self == other
    }
}

impl PartialOrd<[u8]> for BytesMut {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other)
    }
}

impl PartialEq<BytesMut> for [u8] {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

impl PartialOrd<BytesMut> for [u8] {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<str> for BytesMut {
    fn eq(&self, other: &str) -> bool {
        &**self == other.as_bytes()
    }
}

impl PartialOrd<str> for BytesMut {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other.as_bytes())
    }
}

impl PartialEq<BytesMut> for str {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

impl PartialOrd<BytesMut> for str {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl PartialEq<Vec<u8>> for BytesMut {
    fn eq(&self, other: &Vec<u8>) -> bool {
        *self == other[..]
    }
}

impl PartialOrd<Vec<u8>> for BytesMut {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
        (**self).partial_cmp(&other[..])
    }
}

impl PartialEq<BytesMut> for Vec<u8> {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

impl PartialOrd<BytesMut> for Vec<u8> {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        other.partial_cmp(self)
    }
}

impl PartialEq<String> for BytesMut {
    fn eq(&self, other: &String) -> bool {
        *self == other[..]
    }
}

impl PartialOrd<String> for BytesMut {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other.as_bytes())
    }
}

impl PartialEq<BytesMut> for String {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

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

impl PartialEq<BytesMut> for &[u8] {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

impl PartialOrd<BytesMut> for &[u8] {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<BytesMut> for &str {
    fn eq(&self, other: &BytesMut) -> bool {
        *other == *self
    }
}

impl PartialOrd<BytesMut> for &str {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        other.partial_cmp(self)
    }
}

impl PartialEq<BytesMut> for Bytes {
    fn eq(&self, other: &BytesMut) -> bool {
        other[..] == self[..]
    }
}

impl PartialEq<Bytes> for BytesMut {
    fn eq(&self, other: &Bytes) -> bool {
        other[..] == self[..]
    }
}

}

impl From<BytesMut> for Vec<u8> {
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
                let vec = unsafe { (*shared).buffer.take_vec() };

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

#[inline]
#[cfg(not(creusot))]
fn vptr(ptr: *mut u8) -> NonNull<u8> {
    if cfg!(debug_assertions) {
        NonNull::new(ptr).expect("Vec pointer should be non-null")
    } else {
        unsafe { NonNull::new_unchecked(ptr) }
    }
}

// Unadapted pointer updates carry no physical binding. In particular,
// replacing a pointer cannot preserve a previous allocation witness.
#[cfg(creusot)]
#[inline]
#[requires(!ptr.is_null_logic())]
fn vptr(ptr: *mut u8) -> crate::ownership_proof::raw_vec::BoundPtr {
    let pointer = if cfg!(debug_assertions) {
        NonNull::new(ptr).expect("Vec pointer should be non-null")
    } else {
        unsafe { NonNull::new_unchecked(ptr) }
    };
    crate::ownership_proof::raw_vec::BoundPtr::unbound(pointer)
}

/// Returns a dangling pointer with the given address. This is used to store
/// integer data in pointer fields.
///
/// It is equivalent to `addr as *mut T`, but this fails on miri when strict
/// provenance checking is enabled.
#[inline]
#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]
fn invalid_ptr<T>(addr: usize) -> *mut T {
    // This null-derived pointer stores integer metadata only. It carries no
    // allocation permission and must not be used as a dereferenceable pointer.
    let ptr = crate::provenance_specs::metadata_pointer(addr);
    debug_assert_eq!(crate::provenance_specs::pointer_addr(ptr), addr);
    ptr.cast::<T>()
}

unsafe fn rebuild_vec(ptr: *mut u8, mut len: usize, mut cap: usize, off: usize) -> Vec<u8> {
    let ptr = ptr.sub(off);
    len += off;
    cap += off;

    Vec::from_raw_parts(ptr, len, cap)
}

// ===== impl SharedVtable =====

static SHARED_VTABLE: Vtable = Vtable {
    clone: shared_v_clone,
    into_vec: shared_v_to_vec,
    into_mut: shared_v_to_mut,
    is_unique: shared_v_is_unique,
    drop: shared_v_drop,
};

unsafe fn shared_v_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = data.load(Ordering::Relaxed) as *mut Shared;
    increment_shared(shared);

    let data = AtomicPtr::new(shared as *mut ());
    Bytes::with_vtable(ptr, len, data, &SHARED_VTABLE)
}

unsafe fn shared_v_to_vec(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Vec<u8> {
    let shared: *mut Shared = data.load(Ordering::Relaxed).cast();

    if (*shared).is_unique() {
        let shared = &mut *shared;

        // Drop shared
        let mut vec = shared.buffer.take_vec();
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

unsafe fn shared_v_to_mut(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> BytesMut {
    let shared: *mut Shared = data.load(Ordering::Relaxed).cast();

    if (*shared).is_unique() {
        let shared = &mut *shared;

        // The capacity is always the original capacity of the buffer
        // minus the offset from the start of the buffer
        let v = &mut shared.buffer;
        let v_capacity = v.capacity();
        let v_ptr = v.as_mut_ptr();
        let offset = ptr.offset_from(v_ptr) as usize;
        let cap = v_capacity - offset;

        let ptr = vptr(ptr as *mut u8);

        BytesMut {
            ptr,
            len,
            cap,
            data: shared,
            #[cfg(creusot)]
            unique_at_zero: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            pending_control: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_registration: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_context: ghost! { None },
        }
    } else {
        let v = slice::from_raw_parts(ptr, len).to_vec();
        release_shared(shared);
        BytesMut::from_vec(v)
    }
}

unsafe fn shared_v_is_unique(data: &AtomicPtr<()>) -> bool {
    let shared = data.load(Ordering::Acquire);
    let ref_count = (*shared.cast::<Shared>()).ref_count.load(Ordering::Relaxed);
    ref_count == 1
}

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

// BEGIN EXACT SEQUENTIAL SHARED CONTROL GATE
// Restricted helper over the actual Shared layout. Automatic Drop and concurrent
// access remain outside this gate; native Release/Acquire calls execute normally.
#[cfg(all(any(creusot, bytes_proof_probe), not(bytes_proof_repeated_split)))]
pub(crate) mod sequential_shared_control {
    use super::*;
    use creusot_std::{ghost::perm::Perm, logic::Id, prelude::*};
    use crate::ownership_proof::{raw_vec::{self, BoundPtr}, sequential_counter::{SequentialCounter, CounterOwn}, shared_protocol::{self, Coordinator, Packet, Status}};

    #[derive(core::clone::Clone, Copy)]
    pub(crate) struct ControlPtr {
        pub(super) pointer: *mut Shared,
        pub(super) identity: Ghost<Id>,
    }
    pub(crate) struct ControlContext {
        pub(super) registry: Option<Coordinator>,
        pub(super) counter: Option<CounterOwn>,
        pub(super) owner: Option<Box<Perm<*const Shared>>>,
        pub(super) status: Snapshot<Status>,
    }
    impl ControlContext {
        #[logic(open(super), prophetic)]
        pub(super) fn valid(self, control: ControlPtr) -> bool {
            pearlite! {
                match (self.registry, self.counter, self.owner) {
                    (Some(registry), Some(counter), Some(owner)) =>
                        registry.public() == *self.status &&
                        counter@.0 == *control.identity &&
                        counter@.1 == (if self.status.left_pending { 1 } else { 0 }) + (if self.status.right_pending { 1 } else { 0 }) &&
                        *owner.ward() == control.pointer &&
                        owner.val().ref_count@ == counter@.0 &&
                        owner.val().buffer.base.invariant() &&
                        owner.val().buffer.base@ == Some((self.status.allocation, self.status.capacity, 0int)) &&
                        owner.val().buffer.capacity@ == self.status.capacity,
                    (None, None, None) => !self.status.left_pending && !self.status.right_pending,
                    _ => false,
                }
            }
        }
        #[logic(open(super))]
        pub(super) fn active(self) -> bool { pearlite! { self.registry != None } }
    }

    // Before split_to assigns two ranges, all allocation authority remains in
    // this private pending owner. It grants neither handle a byte-access API.
    pub(super) struct PendingControl {
        pub(super) counter: CounterOwn,
        pub(super) owner: Box<Perm<*const Shared>>,
        pub(super) caps: (raw_vec::Recovery, raw_vec::PhysicalRegion),
    }
    impl PendingControl {
        #[logic(open(super), prophetic)]
        pub(super) fn valid(self, pointer: *mut Shared, base: BoundPtr, capacity: usize) -> bool {
            pearlite! {
                base.invariant() && base@ == Some((self.caps.0.namespace(), capacity@, 0int)) &&
                self.caps.0.invariant() && self.caps.1.invariant() &&
                self.caps.0.capacity() == capacity@ && self.caps.1.capacity() == capacity@ &&
                self.caps.1.namespace() == self.caps.0.namespace() &&
                self.caps.1.resource_id() == self.caps.0.namespace() &&
                self.caps.1.lo() == 0 && self.caps.1.hi() == capacity@ &&
                *self.owner.ward() == pointer && self.counter@.1 == 2 &&
                self.owner.val().ref_count@ == self.counter@.0 &&
                self.owner.val().buffer.base == base &&
                self.owner.val().buffer.capacity == capacity
            }
        }
    }
    pub(super) struct HandleRegistration {
        pub(super) control: ControlPtr,
        pub(super) status: Snapshot<Status>,
        pub(super) packet: Packet,
        pub(super) left: bool,
    }
    impl HandleRegistration {
        #[logic(open(super))]
        pub(super) fn view_lo(self) -> Int {
            pearlite! { if self.left { 0int } else { self.status.split } }
        }
        #[logic(open(super))]
        pub(super) fn view_hi(self) -> Int {
            pearlite! { if self.left { self.status.split } else { self.status.capacity } }
        }

        #[logic(open(super), prophetic)]
        pub(super) fn valid(self) -> bool {
            shared_protocol::packet_matches(self.packet, self.status.allocation,
                self.status.capacity, self.status.split, self.status.registration, self.left)
        }
        #[logic(open(super))]
        pub(super) fn same_registration(self, other: Self) -> bool {
            pearlite! {
                self.control == other.control &&
                self.status.allocation == other.status.allocation &&
                self.status.capacity == other.status.capacity &&
                self.status.split == other.status.split &&
                self.status.registration == other.status.registration
            }
        }
        #[logic(open(super), prophetic)]
        pub(super) fn matches(self, context: ControlContext) -> bool {
            pearlite! {
                self.valid() && context.valid(self.control) && context.active() &&
                self.status.allocation == context.status.allocation &&
                self.status.capacity == context.status.capacity &&
                self.status.split == context.status.split &&
                self.status.registration == context.status.registration &&
                (if self.left { context.status.left_pending } else { context.status.right_pending })
            }
        }
    }

    #[requires(pending.inner_logic().valid(pointer, base, capacity))]
    #[requires(split <= capacity)]
    #[ensures(result.0.inner_logic().valid(result.1.inner_logic().control))]
    #[ensures(result.1.inner_logic().control.pointer == pointer)]
    #[ensures(result.0.inner_logic().active())]
    #[ensures(result.0.inner_logic().status.left_pending && result.0.inner_logic().status.right_pending)]
    #[ensures(result.0.inner_logic().status.allocation == pending.inner_logic().caps.0.namespace())]
    #[ensures(result.0.inner_logic().status.capacity == capacity@ && result.0.inner_logic().status.split == split@)]
    #[ensures(result.1.inner_logic().valid() && result.2.inner_logic().valid())]
    #[ensures(result.1.inner_logic().left && !result.2.inner_logic().left)]
    #[ensures(result.1.inner_logic().same_registration(result.2.inner_logic()))]
    #[ensures(result.1.inner_logic().status == result.0.inner_logic().status)]
    #[ensures(result.2.inner_logic().status == result.0.inner_logic().status)]
    #[ensures(result.1.inner_logic().matches(result.0.inner_logic()))]
    #[ensures(result.2.inner_logic().matches(result.0.inner_logic()))]
    #[ensures(forall<index: Int> result.1.inner_logic().packet.1.slot(index) ==
        if 0 <= index && index < split@ { pending.inner_logic().caps.1.slot(index) } else { None })]
    #[ensures(forall<index: Int> result.2.inner_logic().packet.1.slot(index) ==
        if split@ <= index && index < capacity@ { pending.inner_logic().caps.1.slot(index) } else { None })]
    pub(super) fn activate(pointer: *mut Shared, base: BoundPtr, capacity: usize, split: usize,
        pending: Ghost<PendingControl>) -> (Ghost<ControlContext>, Ghost<HandleRegistration>, Ghost<HandleRegistration>) {
        let (caps, counter, owner) = ghost! {
            let pending = pending.into_inner();
            (pending.caps, pending.counter, pending.owner)
        }.split();
        let registrations = shared_protocol::initialize(capacity, split, caps);
        let identity = ghost! {
            let id: Snapshot<Id> = snapshot!(counter@.0);
            id.into_ghost().into_inner()
        };
        let control = ControlPtr { pointer, identity };
        ghost! {
            let (registry, left, right) = registrations.into_inner();
            let counter = counter.into_inner();
            let status: Snapshot<Status> = snapshot!(registry.public());
            (ControlContext { registry: Some(registry), counter: Some(counter), owner: Some(owner.into_inner()), status },
             HandleRegistration { control, status, packet: left, left: true },
             HandleRegistration { control, status, packet: right, left: false })
        }.split()
    }

    #[requires(split@ <= input@.len())]
    #[ensures(result.1.inner_logic().valid(result.0))]
    #[ensures(result.1.inner_logic().active())]
    #[ensures(result.1.inner_logic().status.left_pending && result.1.inner_logic().status.right_pending)]
    #[ensures(result.1.inner_logic().status.split == split@)]
    #[ensures(shared_protocol::packet_matches(result.2.inner_logic(), result.1.inner_logic().status.allocation, result.1.inner_logic().status.capacity, result.1.inner_logic().status.split, result.1.inner_logic().status.registration, true))]
    #[ensures(shared_protocol::packet_matches(result.3.inner_logic(), result.1.inner_logic().status.allocation, result.1.inner_logic().status.capacity, result.1.inner_logic().status.split, result.1.inner_logic().status.registration, false))]
    fn new(input: Vec<u8>, split: usize) -> (ControlPtr, Ghost<ControlContext>, Ghost<Packet>, Ghost<Packet>) {
        let (raw, _len, caps) = raw_vec::detach_vec(input);
        let (base, capacity) = raw.into_bound_ptr_at_zero();
        let registrations = shared_protocol::initialize(capacity, split, caps);
        let (ref_count, counter_own) = SequentialCounter::new(2);
        let shared = Box::new(Shared {
            buffer: SharedBuffer { base, capacity },
            original_capacity_repr: crate::capacity_ops::original_capacity_to_repr(capacity),
            ref_count,
        });
        let (pointer, owner) = Perm::from_box(shared);
        let identity = ghost! {
            let identity: Snapshot<Id> = snapshot!(counter_own@.0);
            identity.into_ghost().into_inner()
        };
        let control = ControlPtr { pointer, identity };
        let state = ghost! {
            let (registry, left, right) = registrations.into_inner();
            let status: Snapshot<Status> = snapshot!(registry.public());
            (ControlContext { registry: Some(registry), counter: Some(counter_own.into_inner()), owner: Some(owner.into_inner()), status }, left, right)
        };
        let (context, left, right) = state.split();
        (control, context, left, right)
    }

    #[requires(context.inner_logic().valid(control) && context.inner_logic().active())]
    #[requires(shared_protocol::packet_matches(packet.inner_logic(), context.inner_logic().status.allocation, context.inner_logic().status.capacity, context.inner_logic().status.split, context.inner_logic().status.registration, left))]
    #[requires(if left { context.inner_logic().status.left_pending } else { context.inner_logic().status.right_pending })]
    #[ensures((^context.inner_logic()).valid(control))]
    #[ensures(result == !(^context.inner_logic()).active())]
    #[ensures(result == (!(^context.inner_logic()).status.left_pending && !(^context.inner_logic()).status.right_pending))]
    #[ensures((^context.inner_logic()).status.allocation == context.inner_logic().status.allocation)]
    #[ensures((^context.inner_logic()).status.capacity == context.inner_logic().status.capacity)]
    #[ensures((^context.inner_logic()).status.split == context.inner_logic().status.split)]
    #[ensures((^context.inner_logic()).status.registration == context.inner_logic().status.registration)]
    #[ensures((^context.inner_logic()).status.left_pending == (if left { false } else { context.inner_logic().status.left_pending }))]
    #[ensures((^context.inner_logic()).status.right_pending == (if left { context.inner_logic().status.right_pending } else { false }))]
    pub(super) fn release(control: ControlPtr, mut context: Ghost<&mut ControlContext>, packet: Ghost<Packet>, left: bool) -> bool {
        let old = {
            let (permission, counter) = ghost! {
                let context = &mut **context;
                (&**context.owner.as_ref().unwrap(), context.counter.as_mut().unwrap())
            }.split();
            let shared = unsafe { Perm::as_ref(control.pointer, permission) };
            shared.ref_count.fetch_sub_release(1, counter)
        };
        shared_protocol::retire(ghost! { context.registry.as_mut().unwrap() }, packet, left);
        ghost! {
            context.status = snapshot!(context.registry.unwrap_logic().public());
        };
        if old != 1 { return false; }

        {
            let permission = ghost! { &**context.owner.as_ref().unwrap() };
            let shared = unsafe { Perm::as_ref(control.pointer, permission) };
            let observed = shared.ref_count.load_acquire(ghost! { context.counter.as_ref().unwrap() });
            assert!(observed == 0);
        }
        let full = shared_protocol::finish(ghost! { context.registry.take().unwrap() });
        let (base, capacity) = {
            let permission = ghost! { &mut **context.owner.as_mut().unwrap() };
            let shared = unsafe { Perm::as_mut(control.pointer, permission) };
            let base = shared.buffer.base;
            let capacity = shared.buffer.capacity;
            // Disarm native SharedBuffer::drop before freeing A explicitly.
            shared.buffer.base = BoundPtr::unbound(NonNull::dangling());
            shared.buffer.capacity = 0;
            (base, capacity)
        };
        unsafe { raw_vec::deallocate_bound_vec(base, capacity, full); }
        let owner = ghost! {
            let _counter = context.counter.take().unwrap();
            context.owner.take().unwrap()
        };
        // Existing standard typed ownership operation; no deallocation-event
        // or automatic-Drop proof is inferred from its contract.
        unsafe { Perm::drop(control.pointer, owner); }
        true
    }

    #[requires(split@ <= input@.len())]
    pub(crate) fn release_left_then_right(input: Vec<u8>, split: usize) {
        let (control, mut context, left, right) = new(input, split);
        let first = release(control, context.borrow_mut(), left, true);
        assert!(!first);
        let last = release(control, context.borrow_mut(), right, false);
        assert!(last);
    }

    #[requires(split@ <= input@.len())]
    pub(crate) fn release_right_then_left(input: Vec<u8>, split: usize) {
        let (control, mut context, left, right) = new(input, split);
        let first = release(control, context.borrow_mut(), right, false);
        assert!(!first);
        let last = release(control, context.borrow_mut(), left, true);
        assert!(last);
    }

    #[cfg(feature = "negative_missing_ticket")]
    pub(crate) fn reject_missing_empty_ticket(input: Vec<u8>) {
        let (control, mut context, left, right) = new(input, 0);
        let _ = left;
        let last = release(control, context.borrow_mut(), right, false);
        assert!(!last);
        proof_assert!(context.status.split == 0);
        proof_assert!(context.status.left_pending && !context.status.right_pending);
        // Every byte has retired, but the empty left handle still owns a ticket.
        let _forbidden = shared_protocol::finish(ghost! { context.registry.take().unwrap() });
    }

}
// END EXACT SEQUENTIAL SHARED CONTROL GATE

#[cfg(bytes_proof_repeated_split)]
#[path = "ownership_proof/carrier_protocol.rs"]
pub(crate) mod sequential_shared_control;
