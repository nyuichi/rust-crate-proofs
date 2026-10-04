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

`constructor-body-body-proved-conversion/` is the current source-extracted
gate after changing `RawAllocation::into_bound_ptr_at_zero` from trusted to
body-proved. All 35 files pass. Its archive has 35 Coma files and 35 proof JSON
files, actual extracted Rust and source-range/hash manifest, plus SHA-256
source digests. The native constructor test is also archived in this directory;
it passed allocation identity, capacity, length, contents, and four-word
layout checks.

Both constructor gates verify the extracted `BytesMut::from_vec` body and its
explicit `proof_release_unique_at_zero` cleanup. They do not establish
full-runtime `BytesMut` integration, automatic `Drop`, promotion, split,
reference count, or shared-storage behavior.
