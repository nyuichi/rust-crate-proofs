# Bounded pointer metadata advancement

This probe imports the actual `src/ownership_proof/raw_vec.rs` and the existing
`src/provenance_specs.rs`. `BoundPtr::advance_within` advances the stored
`NonNull<u8>` from its existing pointer with `wrapping_add`, retains the B1
namespace and capacity, and changes only the sealed absolute offset. Its
contracts require the offset to remain within the allocation. It produces no
`Perm`, `PtrLive`, or byte access permission.

The existing B1 invariant is strengthened to record that the allocation base
address plus byte capacity fits in the `usize` address range. `BoundPtr` carries
the corresponding suffix-fit fact at its current offset. This is a stronger
postcondition on the existing trusted B1 allocation interpretation, not a new
trusted function. `STD-PTRWRAP-01` remains the only wrapping-add specification;
it specifies numeric address arithmetic only.

The positive run proves 29 files. The latest native run passes its one test,
covering zero-capacity, spare-capacity, and initialized allocations. Actual
stdout is retained in the canonical positive evidence directory.

The `negative_overshoot` feature calls `advance_within` past capacity. It
leaves exactly one unproved call-precondition VC; all other VCs pass. The
`negative_interior_cleanup` feature advances a valid pointer by one byte and
then deliberately supplies it to B3, which requires the sealed offset-zero
descriptor. It leaves exactly the B3 offset-zero precondition unproved. Both
negative functions are translation/proof probes only and are not executed by
native tests. Their logs and per-feature Coma/proof JSON are retained under
`../../artifacts/evidence/bound-pointer-offset/negative-overshoot` and
`../../artifacts/evidence/bound-pointer-offset/negative-interior-cleanup`.

`../../artifacts/evidence/bound-pointer-offset/positive` contains the passing Coma/proof JSON and exact
source snapshots used by the positive run. `../../artifacts/evidence/bound-pointer-offset/SHA256SUMS`
records hashes for the archived artifacts.

## Derived base recovery

The fresh advance/retreat gate proves 32 files with no unproved leaves.
BoundPtr additionally records `absolute_offset < current_address`; its
constructor derives this from the existing non-null base, and forward/backward
metadata arithmetic preserves it. `retreat_within` uses the original pointer's
wrapping arithmetic and updates only the sealed offset. Its body is proved;
no new physical or numeric trusted contract is added. The caller advances to
one-past, derives offset-zero base, and consumes the original full-capacity
capabilities through B3. Native pointer round trips cover zero, interior,
one-past, initialized and spare capacity. This is a primitive checkpoint;
actual unique BytesMut advancement is a subsequent integration.
