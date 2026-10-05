# `skip_spaces` isolated translation checkpoint

**Status: source and model translated; body remains unproved. No solver was run.**

This harness path-includes the actual `src/iter.rs`, `src/macros.rs`,
`src/status.rs`, `src/error.rs`, and extracted helper `src/skip_spaces.rs`.
The extracted source keeps the runtime `peek`/match/`bump`/`slice` branches
from the current `skip_spaces` in `src/lib.rs`. The new source adds only
Creusot contracts, loop annotations, and `cfg(creusot)` ghost snapshots and a
ghost lemma call. Erasing those proof-only items leaves the native runtime
body unchanged. `lib.rs` was not edited for this probe. The harness also
includes the independent cursor model and `src/verification/spaces.rs`.

The exact model consumes the maximal run of literal `0x20` bytes. It returns
complete at the first following non-space byte and commits the mark to that
cursor without consuming the terminator. It returns partial at EOF, preserving
the entry mark while leaving the cursor at `end`. The loop has a single
progress measure, `end - cursor`, and invariants for the input, mark, end,
valid cursor state, consumed SP prefix, and endpoint bound. It snapshots the
entry input and maintains `bytes@.input == *entry_input` across each iteration. The independent
`space_prefix_at_cursor` lemma relates the opaque endpoint model to the loop's
current cursor: a non-space/EOF cursor equals the endpoint, while an SP byte
places the endpoint strictly later. The loop calls this checked lemma before
each peek.

The isolated translation completed successfully with:

```sh
./verify-string.sh translate
```

It generated the `skip_spaces`, `space_prefix_end`, `space_prefix_at_cursor`,
and `skip_spaces_model` Coma targets under
`string/verif/httparse_spaces_string_harness_rlib/`. The translation also
emitted the actual `Bytes` targets used by the later closure. The exact new
and direct dependency target paths are listed in
[`evidence/proof-targets.tsv`](evidence/proof-targets.tsv), with prior
same-source Bytes evidence referenced in
[`evidence/dependencies.tsv`](evidence/dependencies.tsv).
It exited successfully with warnings only for unused harness imports/macros.
The harness uses the isolated string-model compiler because the current
compiler ICEs on the existing string-literal contracts in `src/error.rs`.
Translation is configured for the default `std` feature, offline cargo, and
the same toolchain used by the `code-harness`. `verify-string.sh` only runs
Creusot translation and has no proof/solver mode.

The parser helper body remains unproved: no solver was run. The helper has no
parser trust attribute or trusted parser substitute. Its direct `Bytes`
dependencies are `peek`, `bump`, and `slice`. The actual `peek` body depends on
`byte_permission`; `bump` delegates to `advance`; and `slice` depends on
`commit` and `slice_from_ptr_range`. The latter range helper uses the existing
permission and pointer-model contracts. The parser model dependencies are
`space_prefix_end`, `space_prefix_at_cursor`, and `skip_spaces_model`. The
selected proof manifest includes the direct Bytes bodies and the transitive
`byte_permission`, `advance`, `commit`, and `slice_from_ptr_range` bodies. The
existing exact-source Bytes proof artifacts are cross-referenced in the
dependency manifest. The selected harness path-includes the actual Error and
Status sources because `Result<()>` uses those types, though this helper cannot
produce an error. Count shared `Bytes` leaves once rather than adding their
prior proof totals again.

[`verify-proof.sh`](verify-proof.sh) is prepared for a later authorized run.
It reads only the exact target paths in `evidence/proof-targets.tsv`, validates
that those generated targets exist, invokes direct `why3find prove` in the
isolated string-model environment, checks the one-prover/1000 MiB profile and
the isolated Creusot package, and runs through the shared proof lock. The
environment is confined to a child shell with a working-directory restore
trap. This runner was not invoked.

The source review confirms the only new executable syntax is guarded by
`cfg(creusot)`; the native parser path retains the original loop's `peek`,
space match, unsafe `bump`, `slice`, and `Status` returns. A balanced-source
comparison of the runtime `let b = bytes.peek(); match b { ... }` block in both
files returned byte-identical. No native tests, Why3 proof, or full-crate
verification were run as part of this translation-only task.
