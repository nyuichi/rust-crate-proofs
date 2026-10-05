# Raw-pointer runtime probe

This probe imports the published `src/iter.rs` with `#[path]`; the pointer
proofs below are for the actual `Bytes` implementation, not a separate parser.
All counts below come from fresh Why3 sessions and `proof.json` files generated
for the same `src/iter.rs` source hash:

```text
fcb623725d458ad86a60c3934a2095d1f030ad36d6da3a54076c37502026d87f
```

## Fresh body proofs

On 2026-10-05, from this probe directory, `cargo creusot` translated the
actual imported module with no warnings. The selected CoMa targets were then
proved with the pinned Why3 profile (`z3@4.15.3`, one prover, 1000 MiB), through
the shared lock wrapper, using `why3find prove --no-cache -s -j 1`. All 40 VCs
in the seven listed targets are valid:

| CoMa target | VCs | Result |
| --- | ---: | --- |
| `iter/impl_Bytes/byte_permission.coma` | 7 | 7 valid |
| `iter/impl_Bytes/peek.coma` | 9 | 9 valid |
| `iter/impl_Bytes/peek_ahead.coma` | 9 | 9 valid |
| `iter/impl_Bytes/new.coma` | 9 | 9 valid |
| `iter/impl_Bytes/pos.coma` | 2 | 2 valid |
| `iter/impl_Bytes/slice.coma` | 3 | 3 valid |
| `iter/slice_from_ptr_range.coma` | 1 | 1 valid |

For reproduction, the working directory is
`verification/probes/memory-pointer` and the proof command is run through
`../../../run-proof.bash` so it acquires the shared proof lock and exports the
checked Why3 profile. For example, the batch targets are the seven paths in the
table above. `proof.json` and `why3session.xml` are refreshed alongside each
target under
`verif/httparse_memory_pointer_probe_rlib/iter/`; the individual artifact
paths are:

```text
iter/impl_Bytes/byte_permission/{proof.json,why3session.xml}
iter/impl_Bytes/peek/{proof.json,why3session.xml}
iter/impl_Bytes/peek_ahead/{proof.json,why3session.xml}
iter/impl_Bytes/new/{proof.json,why3session.xml}
iter/impl_Bytes/pos/{proof.json,why3session.xml}
iter/impl_Bytes/slice/{proof.json,why3session.xml}
iter/slice_from_ptr_range/{proof.json,why3session.xml}
```

`byte_permission` is body-checked and establishes both exact permission-ward
identity (`ward == ptr`) and the input byte at that cursor index. `peek` and
`peek_ahead` use that derived permission to prove safe reads and exact byte
results, with no extra caller restriction beyond the cursor invariant and
`peek_ahead`'s documented `n <= len` requirement. The `slice` caller proves the
pointer-range helper's range obligations from the constructor-established
invariant and proves the exact returned logical byte subsequence. This is
evidence that the permission bridge is usable by representative runtime
callers; it does not discharge every `Bytes` method.

## Current proof boundary

The fresh body proofs cover exactly `byte_permission`, `peek`, `peek_ahead`,
`new`, `pos`, `slice`, and `slice_from_ptr_range`. `new` establishes the model
state for an input slice; `pos` returns the model-relative offset; and the
read/slice proofs connect returned bytes to the immutable input snapshot.

Other actual method bodies remain open pending their own proof targets,
including `advance`, `bump`, `len`, `is_empty`, `slice_skip`, `commit`,
`advance_and_commit`, `set_cursor`, and `AsRef`. The generic `peek_n` has no
model contract and its `TryFrom<&[u8]>` conversion semantics are not specified
for this generic target. The actual `Iterator for Bytes` implementation is
temporarily excluded only under `cfg(creusot)`; it remains in ordinary Rust
builds. Thus the parser callers requiring `Iterator::next` are not included in
this probe, and neither the `Bytes` API nor the crate is fully verified.

The native httparse tests passed before the final ghost-only
`byte_permission` contract edit: default 100/100 and `--no-default-features`
96/96, including doc tests. The final edit changes only proof annotations and
the ghost-returning helper contract; it does not alter ordinary runtime code.
Treat the native tests as evidence for the runtime version immediately before
that annotation-only edit, not as a rerun of this exact source hash.

## Model and trusted boundary

`Bytes` carries a ghost snapshot of the original `Seq<u8>`, the original
pointer, and a retained `Perm<*const [u8]>`. Its `View` maps `start`, `cursor`,
and `end` to absolute indices relative to the original allocation, and its
invariant binds those indices and the snapshot to the retained permission.
`byte_permission` obtains element permission through the existing
`Perm::index` contract; pointer addition and slice reconstruction use the
existing `PtrAddExt::add_live`, `Perm::split_at`, and `Perm::as_ref` contracts.
No custom permission is conjured in the Creusot body.

The numeric pointer observation for `pos` and `len` uses an extern spec for
`expose_provenance` with `result == self.addr_logic()` and a termination check.
That specification is trusted for the returned numeric address only. The
model does not track Rust's exposed-provenance set, and the contract does not
create `Perm`/`PtrLive` or justify integer-to-pointer conversion. Raw thin
pointer ordering is observed through `.addr()`; this preserves address
ordering without exposing provenance or producing memory permission.

## Historical diagnostics

Earlier, weaker versions are not part of the 40 fresh VCs. A run with weaker
contracts proved only the then-current `new` body and failed on missing
invariant facts; the pointer-range helper had four obligations blocked by
missing range preconditions. A later `peek` attempt timed out on permission
ward equality and the exact byte postcondition. These gaps were addressed by
the current invariant and the body-checked `byte_permission` contract, and all
selected targets were refreshed and passed afterward.

Initial Why3 attempts also failed before opening a CoMa target: one used a
default profile without Z3 4.15.3, and one ran from the crate root without
loading this probe's `why3find.json` package list. Both are environment/setup
failures, not proof results. The matching Creusot theory package is now
resolvable from this probe's working directory, and `run-proof.bash` exports
the checked profile and holds the shared proof lock.

Earlier translation diagnostics identified two backend limitations. Casting a
pointer with `as usize` produced `Unsupported pointer cast:
PointerExposeProvenance`; using the numeric `expose_provenance` spelling and its
numeric-only extern spec makes the actual code translate. Raw pointer
relational comparison caused a Creusot backend ICE at `ty_to_prelude`; the
actual code compares thin pointer `.addr()` values instead. Neither adjustment
supplies memory permissions. Logical pointer offsets are reified in ghost code
through `snapshot!(ptr.sub_logic(origin)).into_ghost().into_inner()` using the
existing `Snapshot<Int>`/`Plain` conversion.

## Remaining work

Restore and verify the actual `Iterator` body with an `IteratorSpec` based on
`CursorModel::cursor_produces` and `cursor_completed`; prove the public next
postcondition and trait laws before removing the temporary `cfg(creusot)`
exclusion. Separately specify the generic `peek_n` conversion, or a precise
supported array fast path, using a body-checked standard-library conversion
contract. Prove the remaining mutation, slicing, and view methods and run the
native suites again after any runtime-code change.
