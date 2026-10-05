# Byte-class translation checkpoint

This harness path-includes the current `src/byteclass.rs`, the original
`src/macros.rs` for `byte_map!`, and `src/verification/model.rs`. The isolated
Creusot translation passed with:

```sh
CARGO_NET_OFFLINE=true ./verify.sh translate
```

Its `classify_byte` caller invokes all four source helpers and specifies their
four independent model results, so the proof run will also check contract
composition at a concrete caller boundary.

The first translation attempt kept the original table declarations as
`static` and failed at `TOKEN_MAP[b as usize]` with:

```text
Unsupported constant value: Scalar(alloc5) of type &'?3 [bool; 256_usize]
```

Declaring the same immutable `byte_map!` expansions as `const` allowed the
translation to pass. The crate's current uses only index the tables or iterate
over their values; no table address is returned, compared, persisted, or used
for unsafe loads. The original table match patterns and predicate bodies are
retained.

The first proof run used the shared queue wrapper and found that each helper's
final table-to-model postcondition goal was unproved: three of four split goals
for URI, header-name, and header-value helpers passed; four of five passed for
the method helper. The `classify_byte` caller (5 VCs), all independent model
obligations, and each map-construction invariant (1 VC per table) passed. The
run ended with four unproved helper files. The map constructor's proof exposed
only array well-formedness, not the per-byte contents, because its constant
initializer was opaque to the generated logic.

In response, `byte_map!` now has a predicate-aware arm. It keeps the runtime
`matches!` loop unchanged while specifying every output byte and maintaining
the already-written prefix as a loop invariant. The three byteclass tables use
that arm with their independent logical predicates. The fresh translation
passed, and the shared-queue proof passed all 29 obligations:

- The three table constructors discharged their complete per-byte result
  contracts (1 VC each).
- The four source helper bodies discharged their contracts (3 VCs each).
- `classify_byte` discharged its four helper-contract compositions and result
  bounds (5 VCs).
- The included independent model discharged 9 VCs: accepted span, cursor skip,
  five Clone bodies, maximal prefix, and token model.

The command sequence for this clean target-local run was:

```sh
source /workspace/proof-tools/activate.sh
cargo clean -p httparse-byteclass-harness
touch src/lib.rs
CARGO_NET_OFFLINE=true ./verify.sh translate
./verify.sh prove
```

`verify.sh prove` retranslates the explicit harness manifest, checks that its
expected Coma target is nonempty, queries the selected `creusot` Why3 package,
then calls `why3find prove --no-cache -s -j 1` through
`../../../run-proof.bash`. The runner checks the selected one-prover, 1000 MiB
Why3 profile before starting proof. Before translation the script cleans only
this harness package in its explicit local `target/` directory and removes
prior Coma outputs recursively from this harness's verification target;
afterward it requires every caller, imported runtime helper, table constructor,
and model-body target in its mandatory manifest to be nonempty. Additional
Coma outputs are allowed. The recorded run had 29/29
successful VCs and no failures. The generated Why3 session is under
`verif/httparse_byteclass_harness_rlib`; translation and proof configuration
is in this target directory.

The 29/29 result above is from the previously recorded proof run. This revised
split-phase runner has only passed shell syntax and diff checks so far; it has
not yet been run to create fresh proof evidence.

The generated `.coma` files show the concrete table-content bridge. For each
table, `const_*_spec` contributes only the array invariant; there is no assumed
per-byte constant axiom. The generated `set_*` initializer calls the verified
`make_map`, then binds that returned array to the constant. The helper proof
uses this initializer path before indexing the array. Its three VCs prove the
constructor body/postcondition, the constant initializer setter, and the
actual helper's postcondition. All three pass for each table.

This remains an isolated shared-source harness checkpoint. The parent crate's
`lib.rs` still owns the original inline declarations until chunk's separate
module-wiring checkpoint, so this report does not claim an integrated parser
runtime bridge yet.
