# Pinned handle-comparison replay

The generated `BytesMut` source, generated probe modules, Why3 tasks, and proof
JSON are packed in `artifacts.tar.gz`. `membermanifest.txt` lists and hashes every
archive member; `archive.sha256` records the archive hash. The proof log, this
README, and `manifest.json` remain outside the archive.

The relevant `BytesMut` source bodies are extracted from `src/bytes_mut.rs`:
`as_slice`, `AsRef`, `Deref`, self/slice comparison methods, and their callers.
The probe adds `check(ghost)` and semantic contracts to generated copies; the
current production `Deref` implementations do not have those annotations.
It also adds ghost purity to the existing immutable B4 read bridges. Those
bridges remain explicitly trusted physical-access boundaries, so this result
does not prove their raw-pointer safety or any bytes ownership/refcount
protocol. It is not integrated whole-crate evidence and includes no `Bytes`
Deref proof.

The iterator portion specializes the actual `IntoIter<T>` body to
`IntoIter<&[u8]>`; it does not establish generic `T: Buf` laws. Heterogeneous
`Vec<u8>`/`BytesMut` orderings run only in native tests, not in this proof.

The captured source is complete and hash-manifested. The later working tree
adds private proof assertions in `reserve` and changes one proof assertion in
`proof_traits_unique`; the target `AsRef`, `Deref`, `as_slice`, and comparison
blocks are byte-identical. See `manifest.json`, `membermanifest.txt`, and the
archived `toolchain.txt` for hashes and the exact boundary.
