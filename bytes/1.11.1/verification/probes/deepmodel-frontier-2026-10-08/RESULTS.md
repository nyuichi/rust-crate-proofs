# Results: DeepModel comparison frontier

This is a translation-only diagnostic for bytes 1.11.1 on the pinned
`nightly-2026-06-22` toolchain. It changes files only in this probe directory.
All three translation commands use the shared `/tmp/itoa-creusot-proof.lock`;
no Why3 prover phase runs.

The negative fixture in `src/missing.rs` is a two-field raw-pointer/length
handle with concrete `PartialEq` and `PartialOrd` impls plus the same generic
reference-forwarding impl shape and bodies as upstream `Bytes`. It has no
`DeepModel`. `./translate.sh negative` reproduces the ICE. Rustc fails to
normalize `<RawBytes as DeepModel>::DeepModelTy` while the Creusot terminates
validator is processing the generic trait-call environment:

```text
<RawBytes as PartialOrd<T>>
<RawBytes as PartialEq<T>>
T: MetaSized
```

The pinned compiler reports `Failed to normalize Alias(... DeepModelTy ...
args: [missing::RawBytes])`; its query stack names
`creusot::validate::terminates::BuildFunctionsGraph::function_dep`. The full
stderr and rustc ICE report are retained in `logs/negative.log` and
`rustc-ice-2026-10-07T15_28_01-818192.txt`.

This matches the stock comparison model contracts in
`/workspace/bytes-proof-tools/bytes-proof-std/src/std/cmp.rs:13` and `:23`:
`PartialEq` and `PartialOrd` require the receiver to implement `DeepModel`, and
the RHS model type must equal the receiver model type. `PartialOrd` also
requires the receiver's model type to implement `PartialOrdLogic`. Their
postconditions compare the logical models (`PartialEq` line 14 and
`PartialOrd` line 26), rather than comparing runtime pointer fields. The
generic forwarding impl's `RawBytes: PartialEq<T>` / `RawBytes: PartialOrd<T>`
bounds lead Creusot to normalize `RawBytes::DeepModelTy`; because there is no
such associated type, the pinned rustc ICEs instead of reporting an ordinary
missing-bound error.

The intermediate `src/modeled_gap.rs` adds `DeepModelTy = Seq<u8>` and a ghost
`Seq<u8>` snapshot to the receiver, but leaves the generic impl bounds
unchanged. `./translate.sh model-gap` no longer ICEs; it returns ordinary
`E0277` because the forwarded comparison call needs a `DeepModel` on generic
`T`. This isolates the receiver projection failure from the separate RHS model
bound needed by this source-level impl.

The positive `src/modeled.rs` also requires
`T: DeepModel<DeepModelTy = Seq<u8>>`. `./translate.sh positive` passes
Creusot translation and emits eight Coma files under `verif/`; it does not run
proofs. The generic model bound supplies the same-model condition required by
the standard comparison specs. Its concrete `PartialEq` / `PartialOrd` bodies
are still placeholders, so the translated bodies carry no evidence of
comparison correctness.

Two model directions remain possible for the original raw-pointer type:

- A logical byte snapshot can be maintained as ghost state and selected as
  `DeepModelTy = Seq<u8>`. A useful contract must link each snapshot to the
  actual initialized bytes visible through the handle at construction and after
  every operation. Storing the snapshot alone leaves it arbitrary.
- `deep_model` can instead be defined from a read view of `ptr` and `len`. That
  needs an explicit pointer-validity, initialization, provenance, and memory
  permission relation that justifies reading exactly those bytes. An address
  plus length model alone cannot express byte-content equality.

This probe does not choose between or establish either memory link. It shows
that comparison translation needs a concrete receiver model projection and a
same-model RHS bound; an implementation path still must define and prove (or
temporarily trust under the authorized strong-contract/removal policy) the
connection between that model and runtime bytes. No production source,
standard-library contract, or tool source was changed.
