# Public unsplit with independent sequential controls

**114 proof files pass**, with no unproved leaves. The native matrix covers 32 combinations of two input lengths (0, 1, 4, 9) and spare capacity (0 or 8).

This gate extracts the actual `BytesMut::unsplit` method under `bytes_proof_unsplit_independent`, with the existing repeated-split/shared-reserve configurations. The method accepts two explicit exclusive `ControlContext` borrows, requires distinct runtime Shared pointers and initialized registered handles, and proves:

- Empty self adopts the other owner while explicitly releasing its previous registration.
- Otherwise, zero-capacity other is explicitly released and self is preserved.
- Otherwise, fresh unique storage contains the exact concatenation, and both old registrations are explicitly retired through their respective coordinators.

Both contexts retain valid state with the precise registration-count changes and sibling frames. The complete caller uses two independently constructed controls, retires the temporary empty siblings, calls the actual public method, and explicitly cleans up the result. It guards distinct runtime control pointers instead of assuming a global allocator-freshness theorem.

`shared_copy::reserve_copy` now permits a singleton source. It copies bytes before consuming that source, so final source reclamation is safe. Its active-context postcondition is conditional on an original count of at least two. A small body-proved coordinate lemma exposes initialization of the new owner. No trusted code is added. The existing Shared reserve gate is replayed separately against this generalized helper.

This is a restricted sequential public-method proof. It does not verify default native dispatch, automatic `Drop`, mixed unique/shared ownership, or native concurrency. The same-control fallback and adjacent cases have separate gates.

Run `bash run-proof.sh` with pinned tools; the wrapper serializes through `/tmp/itoa-creusot-proof.lock`. `cargo test --locked` runs the native matrix. `evidence/public-positive.tar.gz` and its JSON manifest contain source, exact generated Rust, Coma, complete proof sessions, and logs. Earlier kernel evidence records the initial 33/34 initialization-interface failure and the successful 121-file kernel replay.
