# Raw Vec ownership bridge

This probe checks a narrow `Vec<u8>` allocation round trip with two independently
owned physical intervals. B1 detaches the Vec into sealed raw metadata plus
ghost recovery/region capabilities. The checked `PhysicalRegion::split_at` and
`join` operations preserve the resource ledger. B4 lends nonempty known ranges
from each fragment, the caller writes one byte through each returned slice, and
B2 resumes the allocation as a Vec. B3 consumes recovery plus a full region to
explicitly free the allocation by reconstructing a zero-length Vec.

The proof target `detach_mutate_disjoint_fragments` splits at index 2, writes
index 1 through the left fragment and index 2 through the right fragment, then
proves those values survive resume while every other initialized byte is
unchanged. The right fragment also contains the unborrowed `[4, capacity)` tail,
which the B4 contract frames. The B3 callers split and rejoin empty Vecs at 0,
including capacity zero and spare capacity, and nonempty Vecs at 2. They cover
both spatial tuple order and reversed tuple return order; the latter is reordered
before joining and makes no claim about destructor order.

The physical interpretations at `detach_vec` (B1), `resume_vec` (B2),
`deallocate_vec` (B3), and `borrow_mut` (B4) are trusted native boundaries.
B3 treats `Vec::from_raw_parts(base, 0, capacity)` and its allocation-freeing
effect as one trusted operation; this probe does not claim Creusot proves
standard `Drop` effects. The probe proves the contracts compose with the
resource algebra; it does not prove those native bodies, allocator behavior,
Bytes/BytesMut integration, shared reference counting, or reallocations. Empty
regions do not establish allocation liveness; B4 requires a nonempty interval.

Run the Creusot target from the bytes crate root with:

```sh
./scripts/verify-bytes.sh raw-vec-bridge
```

Run the native tests with:

```sh
source /workspace/bytes-proof-tools/activate.sh
cargo test --manifest-path verification/probes/raw-vec-bridge/Cargo.toml --locked
```
