# Bound Vec constructor evidence

## Helper positive and negative controls

`positive/` contains the current 17-file proof of `detach_bound_vec` followed
by explicit B3 deallocation. `negative-unbound-cleanup/` contains the control
that preserves the same native non-null pointer but changes its sealed binding
to `None`. Its only unproved goal is the B3 precondition requiring
`Some((namespace, capacity, 0))`; the exact generated goal and context are in
`negative-unbound-cleanup/negative_vc_detail.log`.

The positive tree has 17 Coma files and 17 proof JSON files. The negative tree
has 18 of each because it additionally translates the negative function. Both
have stdout and source snapshots.

## Source-extracted constructor gate

`constructor-body-initial-pass-rawvec-trusted/` records the first gate over the
actual `BytesMut::from_vec` source body, after the explicit
`proof_release_unique_at_zero` cleanup. It proved 34 files, including the
constructor and release method bodies. It predates the conversion helper's
body proof and is retained as a checkpoint.

`constructor-body-body-proved-conversion/` records the source-extracted gate
after changing `RawAllocation::into_bound_ptr_at_zero` from trusted to
body-proved. All 35 files passed. Its archive has 35 Coma files and 35 proof
JSON files, actual extracted Rust and source-range/hash manifest, plus SHA-256
source digests. Its native constructor tests passed allocation identity,
capacity, length, contents, and four-word layout checks.

`cap-class-contract-2592fae-kernel/` adds the packed capacity-class
postcondition tying `BytesMut::data` to the capacity field returned by
`from_vec`. It proves 35 files and passes 3 native tests, including a nonzero
capacity class. To avoid racing pool work, this checkpoint uses the committed
2592fae `raw_vec.rs` and `owned_region.rs` snapshot; its exact mixed-source
scope and hashes are recorded in that archive.

These constructor gates verify the extracted `BytesMut::from_vec` body and its
explicit `proof_release_unique_at_zero` cleanup. They do not establish
full-runtime `BytesMut` integration, automatic `Drop`, promotion, split,
reference count, or shared-storage behavior.
