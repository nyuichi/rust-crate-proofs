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
