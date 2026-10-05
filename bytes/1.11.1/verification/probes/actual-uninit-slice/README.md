# Exact `UninitSlice` runtime probe

The default extraction captures `UninitSlice`, its view, both mutable-slice
`From` implementations, and the exact `new`, `uninit`, `write_byte`,
`copy_from_slice`, and `len` source methods from `src/buf/uninit_slice.rs`.
The positive proof establishes the typed-slice contracts around those methods
and checks writeback through the initialized conversion and initialization
through the `MaybeUninit` conversion. It proves the extracted methods and local
callers, not an integrated `BytesMut` ownership protocol.

The `remaining_apis` feature adds the exact `uninit_ref`, `as_mut_ptr`, and
`as_uninit_slice_mut` source methods. The mutable slice projection's current and
prophetic values and length are body-proved. Because the projection is unsafe
and its contract exposes the backing `MaybeUninit` slice, it requires callers to
preserve every slot that is currently `Some` in the final wrapper view. The
`bad_deinitialize` control starts from a known byte and attempts to replace it
with `MaybeUninit::uninit()`; its final-state postcondition proves, while the
call is rejected specifically at this preservation requirement. This establishes
the obligation at the unsafe boundary; it does not model provenance through the
bare pointer API.

The `as_mut_ptr` body is proved, but the public bare-pointer API supplies no
pointer provenance or access authority relation in this gate.

The `range_full_index` feature extracts the source `impl_index!` macro and
checks its `RangeFull` expansion. Its proof-only branch has public-model
contracts over `UninitSlice@`; `build.rs` checks that both branch bodies match
the runtime macro bodies after whitespace normalization. The
`range_bounds_index` feature independently checks the half-open
`Range<usize>` implementation with pointwise offset and frame contracts. Each
gate proves only its selected range type; the other four range implementations
remain separate proof targets. The `range_from_index` gate also passes with the
same public `Seq<Option<u8>>` model: its contract maps the suffix and frames the
prefix. Its model functions are body-defined logic and add no trusted range
axiom.

The expanded configuration below proves all 12 generated files, and its native
suite passes all three tests. The captured source, generated extraction, Coma,
Why3 sessions, proof reports, and logs are in the current
`evidence/positive-expanded-rangefull.tar.gz` archive. The earlier eight-file
core run and the intentionally false copy-model control remain separately
archived as `evidence/positive.tar.gz` and `evidence/negative.tar.gz`. The
pre-repair expanded archive is retained as
`evidence/superseded-positive-expanded-rangefull-before-deinitialize-requirement.tar.gz`;
it proves the accessor body but predates the preservation requirement, so it
cannot establish that the unsafe projection boundary is safe.

Run the serialized expanded proof from this directory:

```sh
./verify.bash --features remaining_apis,range_full_index
```

The separate half-open range gate is:

```sh
./verify.bash --features remaining_apis,range_bounds_index
```

Its 12-file proof and 3-test native replay are archived in
`evidence/range-bounds-positive.tar.gz`.

The `RangeFrom<usize>` gate is:

```sh
./verify.bash --features remaining_apis,range_from_index
```

Its 12-file proof and 3-test native replay are archived in
`evidence/range-from-positive.tar.gz`.

The remaining separate range gates are:

```sh
./verify.bash --features remaining_apis,range_to_index
./verify.bash --features remaining_apis,range_to_inclusive_index
./verify.bash --features remaining_apis,range_inclusive_index
```

Each gate proves only its selected `Index` and `IndexMut` form, along with the
12-file `UninitSlice` core/accessor probe and 3 native tests. Their exact source,
generated extraction, proof tasks, and logs are archived respectively in
`evidence/range-to-positive.tar.gz`,
`evidence/range-to-inclusive-positive.tar.gz`, and
`evidence/range-inclusive-positive.tar.gz`. The inclusive-range model uses the
stock in-bounds arithmetic over the public view length and models the empty
range case separately.

Translation only, without starting Why3:

```sh
BYTES_TRANSLATE_ONLY=1 ./verify.bash --features remaining_apis,range_full_index
```

Run the native tests from the crate root:

```sh
cargo test --locked --offline --manifest-path verification/probes/actual-uninit-slice/Cargo.toml --features remaining_apis,range_full_index
```

The unsafe `from_raw_parts_mut` conversion remains outside the probe. A raw
pointer and length alone do not establish the exclusive borrowed slice
authority required to justify that conversion. The raw-pointer accessor gate
does not supply that authority.

The `bad_deinitialize` feature is an intentionally invalid call to the unsafe
mutable-slice projection:

```sh
./verify.bash --features remaining_apis,range_full_index,bad_deinitialize
```

It fails only the `as_uninit_slice_mut` caller requirement. Its exact generated
source, proof task, Why3 session, and log are preserved in
`evidence/deinitialize-negative.tar.gz`.

The unsafe counterexample that proved before this requirement was added is
preserved in `evidence/unsafe-deinitialization-counterexample.tar.gz`. The
repaired scratch audit also includes a ghost-mutation rejection control in
`evidence/prophetic-safety-rejects-deinitialization.tar.gz`.

The `wrong_copy_model` feature adds an intentionally false postcondition:

```sh
./verify.bash --features wrong_copy_model
```

That negative replay fails only `wrong_copy_model ensures`; all eight core
proof files still pass.
