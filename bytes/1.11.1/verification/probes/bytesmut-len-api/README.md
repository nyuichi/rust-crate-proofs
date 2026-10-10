# BytesMut length methods and spatial helper: conditional proof

This probe body-proves the production `BytesMut::len`, `truncate`, and unsafe
`set_len` methods extracted exactly from `src/bytes_mut.rs`. It also proves the
spatial view split helper and the actual pointer/region split dependency
bodies. Creusot cannot
translate the complete crate in this environment, so the probe includes the
actual type records, unique proof model, and method bodies directly from the
crate source. Its source map and generated Rust/COMA are saved in `evidence/`.

Run `./verify.sh` from this directory with the pinned toolchain installed. The
script proves all selected targets with one Why3 worker and refreshes the saved
source, COMA, per-target proof records, and `proof.json` only after success.

The contracts require the existing `unique_valid` predicate. That predicate
currently covers only a Vec-backed handle at allocation offset zero with the
whole allocation region. The producer that establishes it is not body-proved
here. Shared-backed, split BytesMut, nonzero-offset, and immutable-peer states
are not covered by the length-method result. The spatial helper proves interval
partitioning only; it does not establish Shared lifetime, refcount, or
immutable-overlap authority. This is a validated conditional result, not
completion of the public BytesMut API proof.

The crate-level retained snapshot in `verification/results/current.json` and
`current.zip` predates these source changes. Its reuse check is expected to
reject the modified inputs until a new proof and source correspondence are
produced; this probe does not update or certify that snapshot.
