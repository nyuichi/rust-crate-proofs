# Astra consultation: runtime verifier support

This records the support investigation during the httparse 1.10.1 effort.
It does not establish completion of a runtime proof. Root orchestrates execution;
Luna xhigh workers implement the selected routes. See the crate status ledger
and BACKEND-PROBES.md for current reproduced results.

## Pointer-to-integer exposure

The actual `Bytes::pos` expression `self.cursor as usize - self.start as usize`
fails Creusot translation with `PointerExposeProvenance`; `Bytes::len` has the
same operation. `creusot/src/translation/function/statement.rs` explicitly rejects
both PointerExposeProvenance and PointerWithExposedProvenance.

Selected route, approved by root: add a standard-library specification for
`*const T::expose_provenance` (and mutable counterpart as needed), with exact
numeric postcondition `result == self.addr_logic()` and `check(terminates)`.
Use `.expose_provenance()` in the actual runtime expressions. Rust defines that
method as the address-exposing cast, so this preserves the numerical result and
the provenance-exposure effect. It requires Rust 1.84 or later and this increased
minimum for the annotated runtime must be documented.

Do not label this method `check(ghost)`: exposing provenance is not interchangeable
with an erased pure observation. Creusot currently has no exposed-provenance set
model and still rejects integer-to-pointer reconstruction. The numeric contract
must not grant ownership or permission to dereference an address. All reads,
additions and slice construction still require independent Perm/PtrLive proofs.
The exposure effect belongs to the explicitly documented Rust standard-library
TCB; it must not silently become a capability-producing logical rule.

`addr()` has an existing exact numeric contract, but omits the exposure effect
and is therefore not the selected equivalence. `offset_from` also has no existing
program contract here and introduces same-allocation/distance safety obligations;
it is not the smallest repair.

## Endian conversion and SIMD

Current byte-to-word and word-to-byte calls lack standard-library contracts.
The initial SWAR probe translated, but its generated Coma contained `false any`
from impossible external-call preconditions. This is not a successful proof.
The exact arithmetic expression on a word input translates, but that alone does
not connect the actual byte input to its result.

Route: add exact, endian-sensitive standard specifications for the required
`usize::from_ne_bytes` and `to_ne_bytes`. Connect each byte to the corresponding
word bits; handle target pointer width explicitly. The inverse/packing lemmas
used by httparse must then be proved from those standard semantics.

For SSE/AVX, use an exact 16/32-byte lane model of `__m128i`/`__m256i` and
instruction-level specifications for the seven operations actually used:

- set1: replicate the signed byte's bit pattern;
- max_epu8: unsigned maximum in each lane;
- cmpeq_epi8: each output lane is all ones iff the input lanes are equal;
- or and andnot: lane-wise bit operations, with the actual operand order;
- lddqu: exact loaded bytes, with valid live readable range and alignment rules;
- movemask: extract each byte's high bit into the corresponding result bit.

For SSE, movemask is between 0 and 65535. AVX's bit 31 can set the sign bit of
its i32 result; the subsequent cast to u32 must have the correct bit-preserving
meaning. Do not use a nonnegative i32 postcondition for arbitrary AVX masks.
Load wrappers need ghost Perm/PtrLive evidence and mechanical erasure to the
actual intrinsic; pointer numeric bounds alone do not authorize a memory read.
CPU-feature preconditions and the feature-detection result also need models.

These are standard instruction semantics in the TCB, not an assumed httparse
scanner contract. The actual scanner's accepted prefix, first invalid byte,
advance bound, block composition and tail behavior remain body-proof obligations.
In particular SWAR header-value blocks may conservatively stop at TAB, and the
scalar correction consumes it. Avoid claiming a false maximal-prefix block
contract, or exact per-lane SWAR mask semantics without handling subtraction
borrows.

A new `sse-exact` probe has translated a fixed-length signed-lane model,
set1 contract, and movemask sum-of-bits contract. Its caller splats -1 and checks
65535. The generated Coma retains both calls without contractless-call warnings
or `false any`. This demonstrates translation feasibility only; proof execution
and the connection of complete runtime scanners remain separate milestones.
See `verification/probes/backend-translation` and its current status report.
NEON, AVX, foreign pointer widths, and big-endian targets remain separate gaps.

## Raw atomics and global static initialization

The dispatch probe fails at the `static RUNTIME_FEATURE: AtomicU8` declaration.
`creusot/src/ctx.rs::item_type` does not handle DefKind::Static, and
`creusot/src/backend.rs` reports the unsupported definition kind.

A parameterized version avoids that first error, but raw standard AtomicU8
load/store lack contracts and become `false any`. Existing
`creusot-std/src/std/sync/atomic.rs` supports Relaxed operations with histories,
SyncView, Committer and AtomicInvariant. Its AtomicU8 is a distinct wrapper type,
whose constructor is non-const and whose methods take ghost callbacks. Merely
changing the import does not produce an initialized global proof resource.

Recommended smallest raw-atomic bridge to investigate:

1. Give raw `std::sync::atomic::AtomicU8` the same Container/HasTimestamp history
   model as the existing wrapper, within the standard-library support code.
2. Introduce permission-aware extension methods such as
   `load_with(order, Ghost<F>)` and `store_with(value, order, Ghost<F>)`. Their
   runtime bodies call the original std operation with its original ordering.
   Use mechanical erasure declarations to the actual std methods and the same
   Committer contracts as the existing wrapper, with raw AtomicU8 as the ward.
3. Prove the parameterized actual cache body against that history model, keeping
   the original static wrapper/runtime dispatch. Every ghost callback must
   establish its required resource invariant. No unconstrained load result and
   no assumed dispatch-level postcondition is acceptable.
4. Separately connect the actual static declaration and its one-time initializer
   to a stable shared reference plus a persistent invariant resource. This needs
   generic verifier support; the extension methods alone do not establish it.

This route avoids an unsafe layout cast between std and the different wrapper,
which would require an additional representation/layout proof. It is a proposal,
not an implemented or demonstrated bridge. The standard atomic/history primitive
specifications still belong to the TCB, as they do for the existing wrapper;
this does not claim a proof of Rust's standard atomic implementation from zero
axioms. No httparse-specific algorithm theorem may be added to that TCB.

The dispatch invariant can be small:

```
cache == 0
|| (cache == AVX2 && cpu_supports_avx2)
|| (cache == SSE42 && cpu_supports_sse42)
|| cache == NOP
```

Initialization satisfies it. Detection returns an allowed supported value; each
store preserves it. A Relaxed load reads an initial or historical value, so
racing first callers remain safe. Use the existing history model rather than
assuming sequential consistency. Feature availability must be linked to actual
standard feature detection; opaque supported-feature Booleans alone are not a
runtime bridge.

Do not simply categorize Static as Constant: `expand_constant` in
`backend/clone_map/elaborator.rs` generates initializer setters for dependent
modules. Reusing that mechanism could recreate the initial value/resource on
every access or proof module. Static state requires a distinct treatment:

- one stable identity for each static, distinct identities for different statics;
- initializer safety/initial resource established once;
- a persistent shared invariant holding the resource after initialization;
- every access uses the same invariant, without creating duplicate ownership;
- load/store callbacks preserve the invariant across arbitrary interleavings.

Minimum negative checks for a static-support patch: an invalid store must fail;
a post-store load must not be forced to the initializer value; a second access
must not mint another exclusive initial resource; and recursively opening the
same invariant must be rejected. Passing a sequential cache model or a static
wrapper with a trusted postcondition does not meet these requirements.

No scalar/SIMD backend or runtime cache has been authorized for removal by this
consultation. Root will decide further infrastructure work after the currently
tractable pointer and instruction routes are checked. Full verification remains
open until all runtime bridges and configuration obligations are discharged.

## Returned mutable builder references: contract correction

Inspection of the actual ParserConfig setter contracts and generated Coma found
an invalid prophecy contract, rather than a difficult valid solver goal. A setter
returning `&mut Self` originally claimed both:

```
(^self)@ == updated(self@, value)
result@ == (^self)@
```

Here `self@` is the incoming value, `result@` is the returned reference's current
value, and `^self`/`^result` describe values when their mutable borrows end. The
caller can mutate the returned reference, so the receiver's eventual value is
not fixed to the state immediately after the setter. Coma explicitly required
`view(self.final) == updated(entry)` and
`view(result.current) == view(self.final)`, which are false for legal callers.

The sound minimal replacement is:

```
result@ == updated(self@, value)
(^result)@ == (^self)@
```

The first clause describes the setter's immediate update and frames all six
other flags; the second connects eventual changes through the returned reference
to the eventual receiver value. Full value equality `^result == ^self` is also
possible. No condition should freeze that eventual value while the returned
mutable reference remains usable. The existing standard Vec DerefMut specification
uses the same separation of current and eventual values.

A small proof caller should set a flag, change it again through the returned
reference, release the borrow, and verify the final flag and unrelated flags.
Chaining a setter for a different flag checks composition as well. This directly
tests the missing prophecy relationship. Increasing solver timeout cannot prove
the original false contract. This consultation did not launch a solver; execution
remains coordinated by root after the worker's contract changes.

## Passing mathematical indices to ghost permission operations

`Perm::index` and `Perm::split_at` take `Int`, while runtime cursor positions use
`usize`. Direct Pearlite `@` syntax is not valid inside the Rust body of `ghost!`,
and calling a logic-only method there is also invalid. This does not require a
new trusted conversion or a verifier patch.

Existing `Snapshot<T>::into_ghost` accepts `T: Plain`; `Int` already implements
Plain. Obtain the mathematical value in Pearlite, then extract it in ghost code:

```rust
let index = snapshot!(position@);
let element = ghost!(permission.index(index.into_ghost().into_inner()));
```

A pointer offset can likewise be computed inside the snapshot, for example
`snapshot!(ptr.sub_logic(*self.origin))`, and passed through the same existing
adapter. This transfers only a plain integer, not ownership or permission.
The permission is still the original live allocation's witness, and all range
and provenance obligations for indexing/splitting remain to be proved.

Relevant existing definitions are `creusot-std/src/snapshot.rs`,
`ghost.rs::Plain`, and `logic/int.rs::impl Plain for Int`. The recommendation adds
no crate axiom or unconstrained Ghost::conjure. Translation/proof of the complete
cursor remains a separate evidence milestone for the memory worker.

## Follow-up: pointer-ordering ICE and fixed byte decomposition

After applying the existing ghost integer adapter, the cursor translation reached
an ICE at `backend/ty.rs::ty_to_prelude`: non-primitive type `*const u8`. The stack
passes through executable `RValue::into_why`. Inspection of
`backend/program.rs` shows that its BinOp path handles Eq/Ne specially and sends
other operand types to a primitive-number prelude. Actual raw-pointer comparisons
such as `cursor < end` are therefore the leading concrete cause; changing the
logical pointer-distance expression is not the first repair to try.

Recommended minimal diagnostic is a tiny raw-pointer `<` translation probe.
For thin `*const u8`, ordinary address comparisons can be spelled as
`left.addr() < right.addr()` (and corresponding <=, >, >=) without changing their
address-order meaning or exposing provenance. Existing `addr()` contracts then
allow integer translation. This is distinct from the earlier `as usize` casts,
which expose provenance and must keep `expose_provenance()`. All memory access
permissions and allocation relationships remain separate proof obligations.
The proposal was sent to root for adoption; no solver was run by this consult.

For the byte-conversion round-trip proof, avoid opaque `pow(index)` mixed with a
symbolic array index. Use a fixed eight-byte little-endian Horner packing:

```
pack8(b) = b[0] + 256 * (b[1] + 256 * (... + 256 * b[7]))
```

Prove the base-256 step for `0 <= lo < 256` and `hi >= 0`:
`(lo + 256*hi) % 256 == lo` and `(lo + 256*hi) / 256 == hi`.
Eight fixed applications yield the lane identities and pack injectivity.
An exact caller-friendly standard specification can use
`from_ne_bytes(b)@ == pack8(b)` and `pack8(to_ne_bytes(n)) == n@`;
byte ranges plus proved pack injectivity make this unique, not a weaker arbitrary
encoding. The round-trip caller then consumes pack equality and the proved
injectivity lemma. Never assume the round-trip law as a shortcut.

Keep the first lemma and representative caller in the probe while diagnosing;
promote only the needed general arithmetic lemmas and standard conversion
specifications when connecting the actual runtime SWAR body. Big-endian ordering
and 32-bit width require their corresponding packing functions/configurations.

## SSE prefix proof: split after repeated failures

After two attempts at the combined vector-to-prefix proof, keep three separately
proved boundaries: (A) actual intrinsic sequence produces a u16 whose bit i is
exactly the URI predicate on lane i; (B) complement/trailing-zeros maps that mask
to the maximal initial run of one bits; (C) a thin caller composes A and B. The
inspected split contracts in `verification/probes/backend-sse-prefix` are sound.
Do not run a third monolithic attempt with additional assertions.

Use small scalar lemmas before sequence/vector integration:

1. Signed lane to byte: byte_value(x) is in 0..255; byte_value(x)==127 iff
   x==127i8; unsigned max(x,33)==x iff byte_value(x)>=33.
2. Compare masks: for a,b each in {-1i8,0i8}, the sign bit of `!a & b` is one iff
   a==0 and b==-1. Prove the four cases in a small bitwise context.
3. Narrowing a movemask: i32 to u16 preserves bits 0..15; the known 0..65535 range
   additionally establishes numeric equality.
4. u16 complement: for 0<=i<16, bit i of !mask is the negation of bit i of mask.

Prove each body and a representative consumer. If vector quantifier application
is unstable, give a pointwise lemma one arbitrary lane index and the explicit
intrinsic facts at that index, then assemble the quantified mask result once.
Keep lane arithmetic, mask narrowing, and first-set-bit reasoning out of the same
large VC. The load-to-lanes and feature-support bridges remain separate gaps.

The existing `trailing_zeros_logic` standard contract is stronger than its probe
comment initially suggested: for r != BITS,

```
x << (BITS-r-1) == 1 << (BITS-1)
```

already entails that bits below r are zero and bit r is one. What is absent is a
direct postcondition phrased in nth_bit. Thus the probe-local trusted
`exact_u16_trailing_zeros` wrapper should ultimately be body-proved from the
existing shift contract via a shift-to-bit lemma. If the symbolic shift is hard,
use independent fixed-shift cases for r=0..16. Do not describe the existing
standard contract as failing to specify the lower bits. No solver was launched
by this consultation.

## Raw-atomic erasure follow-up

The static-bridge worker reported that scratch support adding raw AtomicU8
Container/HasTimestamp and permission-aware load_with/store_with can coexist with
an erasure audit using `--erasure-check=error -- -Z build-std=std`. Small untrusted
helpers around actual std AtomicU8 new/load/store passed that audit. Contractless
standard-call warnings remained, and trusted extension bodies were skipped by
the erasure checker. Consequently this establishes a useful call-erasure route,
not a proof of the extension history contracts or a complete static bridge.

The generic static feature requires its own item kind (not Constant), stable
identity/reference lowering, one-time initializer/resource obligations, persistent
invariant access, and an actual primitive-call erasure link. Standard atomic
history and static allocation semantics are the declared TCB; the cache invariant
and every stored feature value remain crate proof obligations. The feasibility
of that whole compiler/resource extension is still unestablished.
