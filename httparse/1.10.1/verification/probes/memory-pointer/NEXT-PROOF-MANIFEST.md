# Completed `Bytes` proof manifest

This manifest records the completed selected `Bytes` proof batch. Its unique
current-snapshot total is 141 VCs across 31 CoMa targets; see `REPORT.md` for
the per-target count table, source boundary, and trusted boundary.
The current `src/iter.rs` is SHA-256
`2492453eeb1ec520192e3ff8c93f7bd4941ac7439dab0e7b34fc796a43f9a1a0` and
`src/verification/model.rs` is
`cf3a6f426eb55e28f61a2f5b46277639db9ff86ec0b584bf026735c59aa25cb1`;
`creusot-libs/creusot-std/src/std/ptr.rs` is
`e7aaf642680aaed7229cb185aedd8cea81a2624fd8c48774f3203edc1a134829` and
`creusot-libs/creusot-std/src/std/convert.rs` is
`5a4346a05298dbf71fe16b35718ac57552105426a4e4cc9161176b4bf3ae5348`.
Both probes translated with no warnings. All commands below passed with the
checked Why3 profile, one prover, a 1000 MiB limit, and `z3@4.15.3`, using
`--no-cache -s -j 1`. `iter/impl_Bytes/bump.coma` appears in more than one
phase; its two VCs are counted once in the unique total.

All commands should use the existing `run-proof.bash`, checked Why3 profile,
one prover and 1000 MiB limit. The probe directory is the working directory so
its `why3find.json` loads the pinned Creusot package. The target paths below
are generated CoMa files, not Rust item-pattern guesses.

## Phase A: constructor and pointer/slice dependencies

Run from `verification/probes/memory-pointer`:

```sh
../../../run-proof.bash why3find prove --no-cache -s -j 1 \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/new.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/byte_permission.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/slice_from_ptr_range.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_AsRef_for_Bytes/as_ref.coma
```

The constructor establishes the `Bytes` invariant. The helper derives pointer
range permissions; `AsRef::as_ref` is a real body dependency of both typed
array helpers. These current-hash targets passed 19/19 VCs.

## Phase B: advancement and length dependencies

Run from the same directory:

```sh
../../../run-proof.bash why3find prove --no-cache -s -j 1 \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/len.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/advance.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/bump.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/byte_permission.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/peek.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/peek_ahead.coma
```

`next` calls `peek` and then `bump`; `bump` calls `advance`. This phase proves
the actual dependency chain and exact byte read before proving `next`. The
selected current-hash targets passed 35/35 VCs, including the duplicate
`byte_permission` target (count it once in the manifest total).

## Phase C: generic and fixed-array lookahead

Run from `verification/probes/memory-pointer`:

```sh
../../../run-proof.bash why3find prove --no-cache -s -j 1 \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/peek_n.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/peek_array8.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/peek_array4.coma
```

`peek_n` has only the generic bounds fact (`Some(_) => n <= remaining_len`),
so it does not make claims about arbitrary user-defined `TryFrom` values. The
two typed methods use the same `.get(..N)?.try_into().ok()` runtime steps but
have exact byte-sequence postconditions backed by the audited standard array
contract.
The current-hash pointer targets in this phase passed 30/30 VCs; the two
separate caller targets passed 18/18.

Run from `verification/probes/memory-array-conversion`:

```sh
../../../run-proof.bash why3find prove --no-cache -s -j 1 \
  verif/httparse_memory_array_conversion_probe_rlib/array8_prefix.coma \
  verif/httparse_memory_array_conversion_probe_rlib/array4_prefix.coma
```

These are actual standard-conversion caller bodies for N=8 and N=4. The
`TryFrom` contract is an explicit standard-library TCB assumption, source
audited against the pinned Rust implementation; the caller VCs must still
prove prefix and length facts.

## Phase D: current Iterator evidence

Run from `verification/probes/memory-pointer`:

```sh
../../../run-proof.bash why3find prove --no-cache -s -j 1 \
  verif/httparse_memory_pointer_probe_rlib/iter/bytes_subsequence_head.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/bump.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_IteratorSpec_for_Bytes/produces_refl__refines.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_IteratorSpec_for_Bytes/produces_trans__refines.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_IteratorSpec_for_Bytes/produces_refl.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_IteratorSpec_for_Bytes/produces_trans.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Iterator_for_Bytes/next__refines.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Iterator_for_Bytes/next.coma
```

These current-hash targets passed under the wrapper (see `REPORT.md` for exact
counts and generated proof artifacts). The head lemma proves sequence
decomposition by length/index extensionality; the `bump` frame preserves the
cursor model's input and end fields. The `next` contract explicitly states
`None => completed` and `Some(byte) => produces(singleton(byte), final_state)`.
At prior source hash `0c2a77e281438ae4f67bf4efe7bd9a50c8ec0b1c06df6e0ee2d4e97a3bd9b454`,
the body timed out on one postcondition because `bump` did not expose the full
frame; the current frame and lemma resolved it.

## Phase E: remaining state methods and getters

Run from `verification/probes/memory-pointer`:

```sh
../../../run-proof.bash why3find prove --no-cache -s -j 1 \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/pos.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/slice.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/slice_skip.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/commit.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/advance_and_commit.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/set_cursor.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/is_empty.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/as_ptr.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/start.coma \
  verif/httparse_memory_pointer_probe_rlib/iter/impl_Bytes/end.coma
```

All current-hash targets passed (24/24 VCs). The three pointer getter bodies
have no caller-facing functional postconditions; see `REPORT.md` for this
remaining specification gap.

## Native evidence

After the parser's two fixed-array call sites were switched to
`peek_array8`/`peek_array4`, the chunk owner reported both default and
`--no-default-features` native suites passed, including doc tests. Subsequent
`iter.rs` updates in this milestone are Creusot contracts/ghost proofs; the
ordinary runtime path is unchanged from that tested snapshot.
