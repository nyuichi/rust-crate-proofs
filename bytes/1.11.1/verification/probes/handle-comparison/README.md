# Exact handle comparisons and concrete slice iteration

This gate reuses the `valid-handle-traits` source extractor, including its
actual `BytesMut` declaration, constructor, physical read path, explicit
cleanup, and checked invariant. That invariant covers initialized unique
storage at offset zero and canonical empty handles. It excludes shared
registrations and incomplete ownership transitions.

`build.rs` adds a total `Seq<Int>` model using `Seq::create` over the handle's
logical `proof_view_slot`. Absent/Unknown slots map to zero only to make the
model total; this supplies no ownership or initialization. The actual
initialized predicate and type invariant establish Known bytes before any
physical read. `as_slice` and `from_vec` receive proposed stronger contracts
connecting their existing exact bodies to this model.

The four self/slice `__creusot_eq*` and `__creusot_cmp*` adapter bodies are
extracted unchanged. Their added contracts require initialized handles and
specify exact sequence equality or lexicographic order. `actual-traits` also
extracts the unchanged native self `PartialEq`, `PartialOrd`, `Ord`, and `Eq`
implementations to request the real standard-trait refinement obligations.
These trait implementations get no stronger preconditions. Generic reference
blanket implementations are not included.

`readonly-deref` additionally extracts actual immutable `Deref` and four
cross-slice comparison implementations. Generated copies add ghost purity
only to `borrow_bound`, `borrow_empty_bound`, `borrow_packet`, `pointer_addr`,
`kind`, `as_slice`, and `AsRef::as_ref`; immutable slice results and original
lifetimes/contracts/bodies are retained. This is an explicit extension of the
two existing trusted immutable physical interfaces. No mutable physical
bridge is ghost-callable. Mutable B4's rejected purity extension is documented
separately in `../vtable-leaf-integration/`.

`str-adapters` extracts the two string adapter bodies. Its one additional
standard-library boundary, `src/str_contract.rs`, specifies that `str::as_bytes`
returns the string's modeled UTF-8 bytes. This is not a buffer ownership or
comparison axiom. The trait `PartialEq<str>` is not inferred from these adapter
proofs: stock Creusot gives strings `Seq<char>` models, whereas arbitrary
byte handles use `Seq<Int>`.

`concrete-iterator` specializes the original `IntoIter<T>` representation to
`T = &[u8]`. `build_iter.rs` retains the original constructor/accessor bodies
and rebinds `next`/`size_hint` calls to extracted concrete slice Buf bodies;
every rebinding is inverse-checked. Actual core `Iterator` and
`ExactSizeIterator` implementations receive body-proved specification laws.
No generic Buf law is trusted. The independent `iterator/` gate checks this
small component without importing physical ownership. This is a concrete
slice specialization, not verification of arbitrary `T: Buf`.

Native tests cover empty/prefix/mismatch/high-byte comparisons, UTF-8 strings,
and iteration through empty and nonempty slices. Proof results and snapshots
are recorded alongside the corresponding logs; no whole-crate or automatic
Drop proof is implied by this gate.

Commands from the bytes crate directory:

```sh
bash verification/probes/handle-comparison/run.sh
bash verification/probes/handle-comparison/run.sh --features actual-traits
bash verification/probes/handle-comparison/run.sh --features readonly-deref,concrete-iterator,str-adapters
bash verification/probes/handle-comparison/run.sh --features negative_wrong_equality
bash verification/probes/handle-comparison/iterator/run.sh
```

Proof commands use the shared lock and one prover. Request elevated execution
for Why3 sockets. The ordinary implementation files remain unchanged; generated
model/contract/purity annotations are proposals for reviewed integration.
