> Historical ledger snapshot. Its applicability/count interpretation is superseded by [strict three-crate reconciliation](../diagnostics/strict-three-crate-reconciliation/README.md). Original rows and proof boundaries are preserved below.

# Closure gaps after the endian and nohash supplements

These are **local evidence statuses, not a declaration that all public APIs are
verified or a live publication ledger**. The original nightly-rustdoc/expanded
inventories remain unchanged. `coverage-reconciliation.json` references their
exact identities, bounds/signatures/spans and hashes; it layers the later
results without relabeling generic partial results as complete APIs. Default
and all-features records are both retained; these crates' checked feature sets
do not establish arbitrary cfg combinations or other targets.

## endian-type 0.2.0

Per default/all inventory on x86_64-unknown-linux-gnu:

| Callable group | Count | Current useful guarantee | Remaining boundary |
|---|---:|---|---|
| Handwritten macro BitAnd/BitOr/BitXor | 6 | Original generic wrapper-body contracts | Underlying T operation uses its official trait dictionary/contract; not arbitrary trait implementation correctness |
| Handwritten integer From | 60 | Original body contracts in Aeneas | Explicit 50 scalar-operation models; reviewed Rust/Lean byte-semantics bridge; little-endian x86_64, pointer width64 premises; not Lean verification of Rust std operation bodies |
| Integer to_bytes/from_bytes | 40 | Original Kani all-input contracts for ten concrete integer types, both wrappers | Fixed scalar/array domains and target; separate tool evidence, not Aeneas claims |
| Generic byte-slice APIs | 6 | Kani specializations for ten integer types (60 API/type uses) | Arbitrary Sized+Copy T and longer-buffer/provenance contexts remain unproved |
| External blanket functions | 16 | No new general guarantee | Separate std-body/model obligations |
| Standard trait impls declared by derive | 46 | Excluded under user instruction | This is the only requested exclusion category |

The handwritten total is **112 = 6 + 60 + 40 + 6**. The first106 have the scoped
contracts stated above; the last6 remain generic gaps. The generic six are
`BigEndian<T>` and `LittleEndian<T>` each with `as_byte_slice`,
`as_byte_slice_mut`, and `unsafe from_byte_slice`. The integer specialization
checks do not close those six at their declared generic type.

The16 external functions remain in the machine-readable table: Into, TryFrom,
TryInto, reflexive From, Borrow, BorrowMut, Any::type_id and CloneToUninit for
the two wrapper types. Their applicability bounds and actual impl source
remain in the original inventory. Any potential specialization composition
through a proved From must be established separately; it is not counted here.
These are not omitted merely because their bodies are outside this crate.

Structural items are counted separately: 2 wrapper structs, 1 marker trait,
30 aliases (including NetworkOrder and its aliases), and 10 associated-type
entries. Aliases and the empty marker trait add no function body, but the
underlying type invariants/layout and external associated Error types are not
established by counting them. The six Bitwise Output equations are tied to
the operation model; four blanket TryFrom Error entries retain external
trait-resolution obligations. The original metadata remains authoritative.

## Concrete official backend obstacle (no runtime tests)

`endian-slice-probe` contains only six generic forwarding functions to the
unchanged path dependency, extracted with official Charon `--include endian_type`.
Charon succeeds and includes all six original transparent bodies. Aeneas fails:

| Original functions (both owners) | Original macro definition span | Pinned backend error |
|---|---|---|
| as_byte_slice | src/lib.rs:59 | Unsupported operation `&raw const (*_6)`; InterpExpressions.ml:1483 |
| as_byte_slice_mut | src/lib.rs:64 | Unsupported operation `&raw mut (*_8)`; InterpExpressions.ml:1483 |
| from_byte_slice | src/lib.rs:54 | Raw pointer dereference unsupported; InterpPaths.ml:112 |

The log also records the actual macro invocation spans at lines121 and183.
This is extraction only: no unsafe harness is executed, and no invalid values,
short buffers, misaligned buffers, or stopped Miri experiment are used.
The archive-identical original files are copied solely as an isolated dependency.

The problem extends beyond these first frontend errors. Official
`Std/RawPtr.lean:11-12` models a raw pointer by a stored value T; it has no heap,
address, allocation or provenance. Its scalar cast at lines31-34 deliberately
returns `.fail .undef` with a TODO about separation logic. `Std/SliceDef.lean:15-18`
models a slice as a bounded list, without underlying allocation/byte permissions.
The LLBC reaches size_of, slice as_ptr, and raw from_raw_parts/from_raw_parts_mut;
a general faithful bridge must relate these operations to an actual typed
memory layout and mutable borrowing. Simply supplying successful opaque
functions or a byte-list conversion would not prove the original memory behavior.
These statements concern the pinned official pipeline, not all future backends.

## Generic slice obligations still required

Let S=size_of(T). A meaningful conditional theorem must specify a layout/validity
relation between T, S bytes, initialization, and provenance, rather than infer
it from `Sized + Copy`. In particular, Copy alone does not promise no padding,
that every byte pattern is a valid T, or that pointer-bearing values can be
reconstructed from arbitrary integers/bytes. This observation does **not**
assert a confirmed crate defect or rely on any stopped investigation.

| Obligation | What must be proved, not silently assumed |
|---|---|
| Layout and defined bytes | Typed memory layout, representation invariants, size/alignment, initialized byte mask including padding, and which observations survive typed copies. `repr(transparent)` relates wrapper and T layout but does not remove T's padding/validity requirements. |
| Shared view | Returned length exactly S; shared bytes correspond to initialized readable storage of the original field; allocation/range and lifetime remain valid; no conflicting writes during the shared borrow. A valid typed T alone is not a proof that all its padding is initialized. |
| Mutable view and reconstruction | Exclusive permissions for exactly that region and lifetime; byte writes update the same typed value; every permitted write sequence preserves validity/provenance wherever Rust requires it; unrelated storage is preserved. Arbitrary replacement bytes need a stronger admissible-T/domain predicate than Copy. No claim is made that temporary invalid typed states are acceptable. |
| Unsafe read preconditions | Input length at least S; pointer aligned for T and non-null where required even for ZST; live readable region in an appropriate allocation with permitted provenance; initialized representation valid for T. A byte slice's validity alone does not establish the validity of a T read through its pointer. |
| Unsafe read result | Result denotes the typed value represented by the first S bytes; later bytes are ignored; typed Copy semantics, initialization and provenance of the resulting value are modeled. A byte-for-byte reserialization theorem needs additional representation assumptions for padding. |
| Arbitrary lengths/ZST | Prove all eligible source lengths n≥S, lifetime/aliasing contexts, offsets with correct alignment, zero-sized types, and size/range bounds. The existing exact-size aligned arrays do not quantify those contexts. |
| Safety from public reachability | Determine which wrapper states are obtainable through original valid public construction/composition, and discharge the premises for them. Do not manufacture unsupported states or mistake arbitrary unsafe construction for a safe-API counterexample. |
| Tool correspondence | Extend/validate official raw borrow, raw dereference, casts, allocation/permissions, layout/validity and pointer-to-slice semantics, with a trusted-boundary audit. A new abstract model would require an explicit Rust correspondence argument; proving only the model is insufficient. |

Two different goals must remain separate: (1) conditional functional correctness
for a justified admissible representation domain; (2) the full source-level
safety/behavior claim under exactly the declared public API contracts. Adding
stronger hidden premises may establish (1), but cannot close (2). Even a future
all-bit-pattern/no-padding theorem would not cover all Copy types. The existing
integer Kani packet supplies a concrete useful domain for (1) only. No new
behavioral testing is performed in this reconciliation.

## nohash-hasher 0.2.0: separate external boundary

| Configuration group | Local manual bodies | External defaults | External blankets | External alias-target APIs |
|---|---:|---:|---:|---:|
| std default/all, debug/release | 15 | 23 | 44 | 117 |
| no-default-features, debug/release | 15 | 11 | 18 | 6 |

The15 local bodies consist of14 existing original Aeneas contracts plus manual
Debug. Debug now has a **u8-only** direct original Kani contract for Default,
zero/one write_u8, default format and a fixed64 writer (accept-all or first-error),
separately checked with debug_assertions true/false. It does not close arbitrary
T/stored u64/other writers/options or no_std Debug. Default/all both resolve to
std in this manifest; this source-feature equivalence is not extra independent
Kani runs. The remaining generic Debug obligation is retained.

Among external defaults, Clone::clone_from has supplementary official-model
composition evidence; it is not a separate original std-body contract. Other
Hasher defaults (including write_u128/write_i128, length prefixes and strings),
BuildHasher behavior/defaults, blanket conversions/borrows/Any/CloneToUninit,
and BuildHasherDefault/HashMap/HashSet alias-target methods retain separate
unproved obligations. A full specification must decide and prove any applicable
panic behavior, preconditions, trait laws and std-version boundaries. Existing
14 local proofs, the modeled Clone composition and the bounded Debug supplement
must not be described as verification of these APIs. `external-api-gaps.json`
retains every entry rather than collapsing them into an excluded std category.

## Reproduction and frozen packets

`python3 reconcile.py` reproduces the gap JSON and checks the endian group counts.
The extraction-only `endian-slice-probe/replay.bash` recreates the official blocker.
All existing completed proof packets are read-only inputs to this work. The
new reconciliation intentionally does not overwrite the earlier inventory
snapshots/current-summary/delta or imply a successful publication transaction.
