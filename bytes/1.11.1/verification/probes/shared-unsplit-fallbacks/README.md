# Same-control sequential public unsplit fallbacks

**115 proof files pass**, with no unproved leaves. The native matrix also passes for lengths 0, 1, 4, and 9, spare capacities 0 and 8, and all tested split positions.

The gate extracts the actual `BytesMut::unsplit` method under `bytes_proof_unsplit_fallbacks`, together with the existing repeated-split and shared-reserve configurations. Its contract requires two distinct registered handles for the same control block, with an explicit exclusive `ControlContext`, initialized visible bytes, and an allocation-size bound. The supported branches are:

- Empty self: adopt the other handle and explicitly release the previous self. This branch has priority.
- Otherwise, zero-capacity other: explicitly release other and preserve self.
- Otherwise, a genuinely nonadjacent pair (`self.offset + self.len != other.offset`): copy into fresh unique storage, initialize the appended bytes, and explicitly release both old registrations.

The proof establishes concatenated bytes, resulting ownership, the precise registration-count change, and the remaining siblings' frame. Callers exercise the actual public method and explicitly clean up every owner. A small body-proved lemma relates allocation and visible slot coordinates at offset zero; no new trusted code is added. Native/default dispatch and the existing adjacent gate retain their separate bodies.

This is a restricted sequential public-method proof. It does not establish automatic `Drop`, different-control concatenation, arbitrary native `BytesMut`, or concurrent ownership.

Run `bash run-proof.sh` with the pinned tool activation; it serializes the proof through `/tmp/itoa-creusot-proof.lock`. `cargo test --locked` runs the native matrix. `evidence/public-positive.tar.gz` and its manifest contain the exact source, extracted Rust, Coma, proof sessions, and logs.

Development evidence preserves the first 28/29 copy-kernel failure and the successful 112-file kernel gate. The initial attempt to preserve the unique-at-zero predicate through stronger public contracts was replaced by the small coordinate lemma; the existing `set_len` and `spare_capacity_mut` interfaces are unchanged.
