//! Generic immutable physical projection with a ghost-only choice of authority.
//!
//! This adapter has exactly one native action: `slice::from_raw_parts`. Its
//! caller must establish the same exact pointer, extent and initialized values
//! using either B1/B1-BOX region authority or a standard borrowed slice `Perm`.
//! Selecting a lease never selects a native branch or returns a ghost reference
//! as an ordinary value. There is no Bytes representation or protocol axiom.
//!
//! The Physical arm has the existing B4 interpretation: BoundPtr preserves the
//! provenance of its sealed allocation, and borrowing the initialized region
//! excludes conflicting writes/reclamation for the returned slice lifetime.
//! The Slice arm uses Std `SliceExt::as_ptr_perm` (std/slice.rs:114-121), which
//! obtains its permission through `Perm::from_ref`. Std `Perm::as_ref`
//! (std/ptr.rs:626-633) is the corresponding exact-pointer read rule. In either
//! arm the live borrowed authority supplies alignment, non-nullness and native
//! validity, including a non-null empty slice. Integer address equality never
//! substitutes for the exact provenance-bearing pointer equality below.
//!
//! Adequacy of the native/permission interpretation remains generic physical
//! TCB. Replacement requires a supported from_raw_parts permission/region
//! contract, preserving this interface and its borrow lifetime. A caller proof
//! proves neither this adapter's adequacy nor any destructor effect.

use creusot_std::{ghost::perm::Perm, prelude::*};
use crate::raw_vec::{BoundPtr, PhysicalRegion};

pub(crate) enum ReadLease<'a> {
    Physical {
        bound: &'a BoundPtr,
        region: &'a PhysicalRegion,
    },
    Slice(&'a Perm<*const [u8]>),
}

impl ReadLease<'_> {
    /// The expected sequence is checked against actual borrowed authority;
    /// it is not an assertion supplied by a representation-specific oracle.
    #[logic(open(crate), prophetic)]
    pub(crate) fn authorizes(self, pointer: *const u8, len: usize, contents: Seq<u8>) -> bool {
        pearlite! {
            contents.len() == len@ &&
            match self {
                ReadLease::Physical { bound, region } => {
                    bound.invariant() && bound@ != None &&
                    pointer == bound.raw_pointer() as *const u8 &&
                    region.invariant() &&
                    bound@.unwrap_logic().0 == region.namespace() &&
                    bound@.unwrap_logic().1 == region.capacity() &&
                    region.resource_id() == region.namespace() &&
                    region.lo() <= bound@.unwrap_logic().2 &&
                    bound@.unwrap_logic().2 + len@ <= region.hi() &&
                    (forall<i: Int> 0 <= i && i < len@ ==>
                        region.slot(bound@.unwrap_logic().2 + i) == Some(Some(contents[i])))
                }
                ReadLease::Slice(permission) => {
                    permission.invariant() &&
                    pointer == *permission.ward() as *const u8 &&
                    permission.val()@ == contents
                }
            }
        }
    }
}

/// Both the region/descriptor and Std permission are borrowed for `'a`.
/// The returned native slice cannot outlive that authority.
#[trusted]
#[requires(lease.inner_logic().authorizes(pointer, len, *expected))]
#[ensures(result@ == *expected)]
pub(crate) unsafe fn borrow_any<'a>(
    pointer: *const u8,
    len: usize,
    lease: Ghost<ReadLease<'a>>,
    expected: Snapshot<Seq<u8>>,
) -> &'a [u8] {
    unsafe { core::slice::from_raw_parts(pointer, len) }
}
