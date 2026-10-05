# Unique disjoint in-place reclaim

This focused bytes 1.11.1 probe includes the original
`src/ownership_proof/unique_reclaim.rs` directly. Its `reclaim` body splits the
full physical region at the handle offset, reads the initialized source through
existing B4, writes the disjoint prefix through B4's uninitialized mutable
borrow and `storage_ops::copy_to_uninit_prefix`, then rejoins the regions.
The caller retains `Recovery`, so no ownership protocol or trusted contract is
added. The helper preserves the allocation identity, capacity, recovery resource
identity, and all slots outside the destination prefix.

The helper requires `len <= offset` and a Known source prefix. Overlapping moves
are outside this helper's contract. Empty views, including a zero-capacity
allocation and a one-past view, are supported.

Validation:

- `bash verify.bash`: helper and caller proved (36 files). The replay in
  `positive-known.log` also proves the explicit initialized-result postcondition.
- `cargo test --offline`: two tests pass; the matrix checks every admissible
  offset/length for initialized lengths 0 through 39 with spare capacity, checks
  exact copied bytes and original allocation pointer, and releases through B3.
- `bash verify.bash --features negative_overlap`: translation succeeds and
  `vc_reject_overlap` is rejected at the `len <= offset` precondition.
- `bash verify.bash --features negative_unknown_source`: translation succeeds;
  `vc_reject_unknown_source` proves 14/15 goals and rejects the Known source
  precondition.

Proofs use the shared `/tmp/itoa-creusot-proof.lock` and one prover. Logs record
the actual results. Negative features must never be executed natively: they
deliberately violate the physical-access preconditions being checked.

`actual-method-suggestion.inc` is an integration suggestion, not an extracted
or proved BytesMut method. The root agent owns insertion into `bytes_mut.rs`,
module registration, extraction, and the actual-method integration proof.
