# Raw Vec ownership bridge

This probe checks a narrow `Vec<u8>` allocation round trip with two independently
owned physical intervals. B1 detaches the Vec into sealed raw metadata plus
ghost recovery/region capabilities. The checked `PhysicalRegion::split_at` and
`join` operations preserve the resource ledger. B4 then lends nonempty known
ranges from each fragment, the caller writes one byte through each returned
slice, and B2 resumes the allocation as a Vec.

The proof target `detach_mutate_disjoint_fragments` splits at index 2, writes
index 1 through the left fragment and index 2 through the right fragment, then
proves those values survive resume while every other initialized byte is
unchanged. The right fragment also contains the unborrowed `[4, capacity)` tail,
which the B4 contract frames. Native tests exercise both the unchanged B1/B2
round trip and the disjoint mutation path.

The physical interpretations at `detach_vec` (B1), `resume_vec` (B2), and
`borrow_mut` (B4) are trusted native boundaries. This probe proves their
contracts compose with the resource algebra; it does not prove those native
bodies, allocator behavior, automatic drop effects, Bytes/BytesMut integration,
shared reference counting, or reallocations. Empty regions do not establish
allocation liveness; B4 requires a nonempty interval.

Run the Creusot target from the bytes crate root with:

```sh
./scripts/verify-bytes.sh raw-vec-bridge
```

Run the native tests with:

```sh
source /workspace/bytes-proof-tools/activate.sh
cargo test --manifest-path verification/probes/raw-vec-bridge/Cargo.toml --locked
```
