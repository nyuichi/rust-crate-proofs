# Heterogeneous `Vec<u8>` / `BytesMut` comparison proof

The archive contains the exact generated probe source and proof tasks for all 103 results from
`cargo creusot --features vec-traits` on the pinned Creusot toolchain. The gate
re-emits these three source implementations under Creusot:

- `PartialEq<Vec<u8>> for BytesMut`
- `PartialEq<BytesMut> for Vec<u8>`
- `PartialOrd<BytesMut> for Vec<u8>`

Each implementation receives only a semantic result contract; no new method
preconditions are added. The caller `compare_vec_to_unique` has the postcondition
`result == left.deep_model().cmp_log(right.deep_model())`. Its right operand is
constructed through the existing typed unique-handle path and explicitly
released after comparison.

The first proof attempt exposed a missing pinned-library contract on
`Vec::as_slice`; its refinement postcondition already passed. The source body
now uses `(&self[..]).partial_cmp(other.as_slice())`, which is native-equivalent
and uses Creusot's stock `Index<RangeFull>` sequence contract. The final body and
caller VCs pass. Native tests pass 2/2.

The readonly extraction builder now detects existing `check(ghost)` attributes
and adds one only when absent. The archived audit confirms the production
annotations were reused without duplicate attributes. This proof still depends
on the handle-comparison gate's existing checked unique-handle invariant and
immutable physical-read boundary; it does not verify the complete bytes
ownership/refcount protocol, and it says nothing about string comparisons.

`artifacts.tar.gz` includes the proof tree, source files, generated source,
logs, and toolchain record. `membermanifest.txt` hashes each input member, and
`archive.sha256` records the archive hash. The preserved earlier 122-proof
baseline is a separate evidence archive and was not modified. After updating
the readonly source extractor to avoid duplicate attributes, the baseline
feature set was replayed and again proved 122 files; that replay log is included
in this archive.
