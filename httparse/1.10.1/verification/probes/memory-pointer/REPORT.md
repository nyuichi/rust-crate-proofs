# Raw-pointer runtime probe

This probe imports the published `src/iter.rs` with `#[path]`; the pointer
proofs below are for the actual `Bytes` implementation, not a separate parser.
The completed memory-proof checkpoint used this exact `src/iter.rs` hash:

```text
fcb623725d458ad86a60c3934a2095d1f030ad36d6da3a54076c37502026d87f
```

After that checkpoint, the actual `Iterator` body and its `IteratorSpec` were
restored, and a weak bounds-only contract was added to generic `peek_n`. The
previous complete 141-VC snapshot used `src/iter.rs` hash
`2492453eeb1ec520192e3ff8c93f7bd4941ac7439dab0e7b34fc796a43f9a1a0`.
Pointer-identity postconditions have since been added to `as_ptr`, `start`, and
`end`. The preceding 141-VC snapshot used source hash
`c5c8da21125db4b75107a974af9941aa2ba3908f1714ca74487663b6182f52e8`. The
current source hash is
`369662bbf36c68ba814750ce3a91c4aa78ab40e9453af0f7e07f30d869103625`; the
standard conversion contract hash is
`5a4346a05298dbf71fe16b35718ac57552105426a4e4cc9161176b4bf3ae5348`.

On 2026-10-05, both probes were freshly translated with the active byte
compiler, SHA-256
`1ee46c7a5e05f3dbbdbd03804dff6468338c36134711840372e606bfae0d4b7e`, and no
warnings. The full selected batch was rerun with the checked Why3 profile
(`z3@4.15.3`, one prover, 1000 MiB), through `run-proof.bash` and
`why3find prove --no-cache -s -j 1`. All 31 unique targets passed: 121
recursive proof leaves in memory-pointer and 18 in memory-array-conversion,
for 139 distinct `(target, leaf)` pairs. Fresh CoMa, proof JSON, and Why3
sessions with SHA-256 hashes are archived in
[`evidence/active-byte-refresh-2026-10-05`](evidence/active-byte-refresh-2026-10-05/).

## Previous memory-checkpoint body proofs

At the preceding `c5c8da...` source snapshot, `cargo creusot` translated the
actual imported module with no warnings. The selected CoMa targets were proved
with the pinned Why3 profile (`z3@4.15.3`, one prover, 1000 MiB), through the
shared lock wrapper, using `why3find prove --no-cache -s -j 1`. The 40 VCs in
the seven listed targets were valid for that historical snapshot:

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

## Comprehensive `Bytes` proof manifest

The refreshed selected `Bytes` proof snapshot is identified by these source
hashes:

| Source | SHA-256 |
| --- | --- |
| `src/iter.rs` | `369662bbf36c68ba814750ce3a91c4aa78ab40e9453af0f7e07f30d869103625` |
| `src/verification/model.rs` | `cf3a6f426eb55e28f61a2f5b46277639db9ff86ec0b584bf026735c59aa25cb1` |
| `creusot-libs/creusot-std/src/std/ptr.rs` | `e7aaf642680aaed7229cb185aedd8cea81a2624fd8c48774f3203edc1a134829` |
| `creusot-libs/creusot-std/src/std/convert.rs` | `5a4346a05298dbf71fe16b35718ac57552105426a4e4cc9161176b4bf3ae5348` |

Every listed target passed through `run-proof.bash` with `--no-cache -s -j 1`,
the checked Why3 profile, one prover, a 1000 MiB limit, and `z3@4.15.3`. The
memory-pointer targets below import the actual `src/iter.rs`; array conversion
caller targets run in the separate `memory-array-conversion` probe against the
standard contract hash above. Each CoMa target is counted once. In particular,
`iter/impl_Bytes/bump.coma` was refreshed in the dependency batch and also used
by the Iterator batch; its two VCs are counted once in the total.

| Probe | Exact CoMa target (relative to its `verif` root) | VCs |
| --- | --- | ---: |
| memory-pointer | `iter/bytes_subsequence_head.coma` | 1 |
| memory-pointer | `iter/impl_Bytes/new.coma` | 9 |
| memory-pointer | `iter/impl_Bytes/byte_permission.coma` | 7 |
| memory-pointer | `iter/slice_from_ptr_range.coma` | 12 |
| memory-pointer | `iter/impl_AsRef_for_Bytes/as_ref.coma` | 2 |
| memory-pointer | `iter/impl_Bytes/pos.coma` | 2 |
| memory-pointer | `iter/impl_Bytes/len.coma` | 2 |
| memory-pointer | `iter/impl_Bytes/is_empty.coma` | 2 |
| memory-pointer | `iter/impl_Bytes/advance.coma` | 6 |
| memory-pointer | `iter/impl_Bytes/bump.coma` | 2 |
| memory-pointer | `iter/impl_Bytes/advance_and_commit.coma` | 3 |
| memory-pointer | `iter/impl_Bytes/commit.coma` | 1 |
| memory-pointer | `iter/impl_Bytes/slice.coma` | 3 |
| memory-pointer | `iter/impl_Bytes/slice_skip.coma` | 8 |
| memory-pointer | `iter/impl_Bytes/set_cursor.coma` | 2 |
| memory-pointer | `iter/impl_Bytes/as_ptr.coma` | 1 |
| memory-pointer | `iter/impl_Bytes/start.coma` | 1 |
| memory-pointer | `iter/impl_Bytes/end.coma` | 1 |
| memory-pointer | `iter/impl_Bytes/peek.coma` | 4 |
| memory-pointer | `iter/impl_Bytes/peek_ahead.coma` | 9 |
| memory-pointer | `iter/impl_Bytes/peek_n.coma` | 10 |
| memory-pointer | `iter/impl_Bytes/peek_array8.coma` | 10 |
| memory-pointer | `iter/impl_Bytes/peek_array4.coma` | 10 |
| memory-pointer | `iter/impl_Iterator_for_Bytes/next.coma` | 6 |
| memory-pointer | `iter/impl_Iterator_for_Bytes/next__refines.coma` | 1 |
| memory-pointer | `iter/impl_IteratorSpec_for_Bytes/produces_refl.coma` | 2 |
| memory-pointer | `iter/impl_IteratorSpec_for_Bytes/produces_trans.coma` | 2 |
| memory-pointer | `iter/impl_IteratorSpec_for_Bytes/produces_refl__refines.coma` | 1 |
| memory-pointer | `iter/impl_IteratorSpec_for_Bytes/produces_trans__refines.coma` | 1 |
| memory-array-conversion | `array8_prefix.coma` | 9 |
| memory-array-conversion | `array4_prefix.coma` | 9 |
| **Unique current-snapshot total** | **31 target files; bump counted once** | **139** |

The total is 83 VCs for memory-pointer construction, permission, reads,
lookahead, and array-helper dependencies; 18 VCs for the two actual
standard-conversion callers; 24 VCs for `pos`, `slice`, `slice_skip`, `commit`,
`advance_and_commit`, `set_cursor`, `is_empty`, and the pointer getters; plus
14 Iterator-specific VCs after excluding the shared `bump` target, counted
once in the dependency set: `83 + 18 + 24 + 14 = 139`.

The pointer getters now have functional postconditions: `as_ptr` returns the
origin pointer offset by the model cursor, `start` by the model mark, and `end`
by the model end index. Each getter's one-VC body target passed for this exact
source snapshot. The safe `Bytes` methods have no added preconditions. Unsafe
`peek_ahead`, `slice_skip`, and `set_cursor` retain requirements derived from
their documented same-buffer/bounds safety obligations.

## Iterator proof slice

At source hash
`40cb9e7dae081f7b480ca75ed44d40d209db76de4d2d66e511a414cbd43a9b3b`, a
separate batch proved 95 selected VCs: `new` (9), `byte_permission` (7), the
pointer-range helper (1), `AsRef::as_ref` (2), `len` (2), `advance` (6),
`bump` (2), `peek` (9), `peek_ahead` (9), generic `peek_n` (10), the private
array helpers (10 each), and the standard conversion callers for arrays of 8
and 4 bytes (9 each). Those results predate the current `bump` frame contract
and are not fresh for the current hash.

At source hash
`0c2a77e281438ae4f67bf4efe7bd9a50c8ec0b1c06df6e0ee2d4e97a3bd9b454`, the two
`IteratorSpec` refinement checks and the reflexive/transitive production laws
passed (6 VCs), as did `next__refines` (1 VC). The actual `next` body passed
7/8 split obligations; its remaining `next ensures` obligation timed out
(Z3 unknown, 5.14 s, 0 steps). Inspection showed that `bump` exposed only the
cursor update, while `cursor_produces` also requires unchanged input and end.
The subsequent `2492453e...` snapshot strengthened `bump` with the frame
already established by `advance(1)` and added a body-checked
`bytes_subsequence_head` helper for the remaining sequence decomposition. At
that hash, the helper passed 1/1, `bump` passed 2/2, the actual `next` body
passed 14/14, `next__refines` passed 1/1, both IteratorSpec refinement checks
passed 1/1 each, and the reflexive and transitive IteratorSpec laws passed 2/2
each. The fresh active-byte refresh at `369662...` reran all selected iterator
targets. It proved `next` 6/6 and the production laws 2/2 each. The generated
VC split differs from the preceding checkpoint: `slice_from_ptr_range` has 12
leaves, `peek` has 4, and `next` has 6. The archived proof JSON and Why3
sessions record the exact successful leaves.

## Current proof boundary

At the current source hashes, the 139-VC manifest above covers all selected
actual `Bytes` method bodies, the permission and pointer-range helpers, the
fixed-array methods and standard-array callers, and the `IteratorSpec`/`next`
contract. The returned-byte and state contracts connect reads, slices,
advances, commits, and iteration to the immutable `CursorModel` input. Unsafe
`peek_ahead`, `slice_skip`, and `set_cursor` obligations are proved from their
documented same-buffer and bounds requirements. Safe methods have no added
preconditions.

`peek_n<U>` is proved with its deliberately weak generic postcondition: a
`Some(_)` result implies `n <= remaining_len`; the proof makes no claim about
the value produced by arbitrary downstream `TryFrom<&[u8]>` implementations.
For the parser's fixed widths, `peek_array8` and `peek_array4` state exact
subsequence results and are proved against the audited standard array
conversion contract. The three public pointer getters specify their returned
pointer in the immutable cursor model and their bodies are proved against
those postconditions. Complete parser-result refinement and whole-crate
integration are outside this `Bytes` manifest.

The chunk owner reported both default and `--no-default-features` native
suites passing after the parser call sites switched to `peek_array8` and
`peek_array4`, including doc tests. Later `iter.rs` changes add only
Creusot-only contracts and ghost proof code; the ordinary runtime path is
unchanged from that native-tested snapshot.

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

The standard array conversion extern spec is in
`creusot-libs/creusot-std/src/std/convert.rs`. It states exact length success
and failure and same-index copying for the standard `T: Copy` implementation.
Its source basis is rustc `1.95.0-nightly`
(`6a979b3e32522049d0acb4a47f7ae44b7c8abfd5`),
`library/core/src/array/mod.rs` SHA-256
`67c051d28fd7a68b7ea49088918a5076329483a95c6d27200078963fcb1c374b`, lines
249-260 and 302-310. This trusted std contract does not constrain arbitrary
user-defined `TryFrom` implementations.

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

## Remaining scope

The selected `Bytes` body and iterator goals in the manifest pass at the
recorded hashes, including the caller-facing pointer postconditions for
`as_ptr`, `start`, and `end`. Parser result/state refinement, `Error` formatting,
header initialization, SIMD/runtime dispatch, and the complete crate
configuration matrix remain separate open work; this manifest does not close
the crate-level verification gate.
