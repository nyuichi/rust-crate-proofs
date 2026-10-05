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

## Chunk accumulator: bound the recurrence before the parser caller

The inspected `hex_capacity(digits)` has only a 0..16 precondition and recursively
computes 16^digits. Its caller receives no positivity or maximum bound. The
runtime `hex_nibble` contract states agreement with the model but omits the
immediately useful nibble<=15 fact. Thus the one-step parser mixes recursive
power unfolding, three digit-class cases, overflow checks, state transitions
and sequence reasoning in a single proof context.

Recommended proof order, without another large-caller attempt:

1. Give hex_capacity the proved postcondition result>=1.
2. Prove cap_monotone(a,b): 0<=a<=b<=16 implies cap(a)<=cap(b), recursively on
   b-a. For a<b, consume the result for (a,b-1), positivity, and the one-step
   recurrence cap(b)=16*cap(b-1).
3. Prove the fixed endpoint facts cap(15)=1152921504606846976 and
   cap(16)=18446744073709551616 separately. A small bounded constant-unfolding
   proof is preferable to requiring every parser branch to discover 16 levels.
4. Expose cap_digit_bounds(d): cap(d)<=2^64 for d<=16, and cap(d)<=2^60 for d<16.
5. Add the direct `Some(nibble) => nibble@ <= 15` postcondition to hex_nibble,
   preserving its exact model agreement.
6. Prove a small mathematical append lemma. Given 0<=d<16,
   0<=s<cap(d), 0<=n<=15, return s<=u64::MAX/16,
   16*s+n<=u64::MAX, and 0<=16*s+n<cap(d+1).
7. Prove a representative runtime append helper performing only the actual
   multiplication/addition against that lemma before integrating it into the
   parser step.

The safety calculation is linear: s<=2^60-1, hence 16*s<=u64::MAX-15, and n<=15.
The capacity calculation is 16*s+n<16*cap(d)=cap(d+1). Do not introduce variable
products such as cap(d)*cap(16-d); they add nonlinear arithmetic unnecessarily.
The debug overflow-rejection branch is unreachable by the same explicit bound.

Another missing interface is the logical `step` result: its Continue branch
currently has no explicit progress/state-preservation postcondition. After the
arithmetic helper, prove direct facts for Continue: next_pos=pos+1<=input.len(),
0<=next.digits<=16, 0<=next.size<cap(next.digits), and flag exclusion. Complete
should directly bound consumed and size. Mirror those structural postconditions
on the runtime step while retaining exact steps_equal. The recursive scan and
public parser can then compose contracts without reopening every lexical branch.
This consultation inspected the actual source and Coma but did not launch a
solver; root retained the proof queue.

## 2026-10-05: smallest global-dispatch closure path

### Decision: a stable value invariant, not a general global history resource

The current tracked atomic report establishes 15 VCs for the parameterized
history caller, not persistent concurrent access or initialization. The raw
extension experiment establishes some primitive-call erasure, not its trusted
history contract. Neither artifact closes the actual global static.

For httparse's cache, a smaller sound abstraction than general global Perm/history
support is sufficient. Introduce a generic verifier rule for a private immutable
`static AtomicU8` and a pure stable predicate `I: u8 -> bool`:

- prove `I(initial_value)` in a separately emitted initializer obligation;
- every load returns an arbitrary byte satisfying I;
- every store must prove I of its supplied byte;
- do not manufacture any exclusive permission or assume a load equals the
  initial value. Loads are nondeterministic independently of initial-state use.

This is an overapproximation of atomic memory behavior, not a full history or
linearizability model. Its justification is induction on finite execution
histories: initialization satisfies I, every permitted write preserves I, and
an atomic read observes an initialization/write value. All historical values
satisfy I, including stale Relaxed reads. The rule is adequate for value safety
under arbitrary interleavings. A small generic Why3 trace lemma should document
that implication. It must not assert that a load returns the latest store, or
claim a precision level that this model cannot establish.

The crate-specific choice is:

```
choose = if usable_avx2() { AVX2 }
         else if usable_sse42() { SSE42 }
         else { NOP }
I(v) = (v == 0 || v == choose)
```

The actual detector is proved to return choose. A cache load yields 0 or choose;
the zero branch detects/stores choose, so the actual get_runtime_feature body
returns choose in either case. I(0) is unconditional: the static initializer
neither invokes feature detection nor needs to know the CPU's capabilities.
All original backend choices, standard atomic operations and relaxed ordering
remain in the executable source. No backend or cache is removed.

### Compiler-checked attachment and alias confinement

For the first implementation, avoid a new procedural macro. A target-local
registration manifest can map a complete Rust static def-path to a complete
logical predicate def-path. The compiler must resolve each uniquely to actual
DefIds, validate the following conditions, and fail closed on missing/ambiguous
entries. The manifest itself is not proof evidence or a grant of trust.

- The static is local and non-mut, with normalized type exactly the sysroot
  core/std AtomicU8 (resolved type identity, not suffix/name matching).
- It is private, has no exported symbol/linkage attributes, and its initializer
  is a recognized standard AtomicU8::new with a compile-time u8 value. Initially
  reject arbitrary initializer helpers or arbitrary UnsafeCell expressions.
- The predicate is a checked, pure, nonprophetic logical function `(u8)->bool`,
  with no mutable-resource, view-time, arbitrary ghost-permission or static-cell
  dependencies. It may use arithmetic, immutable constants, checked pure
  helpers and explicitly identified stable CPU observations. Do not accept a
  crate-local trusted predicate law in place of proving preservation.
- A function-local static CACHE is permitted: Rust still initializes it once and
  it cannot capture the function's local values. Bind it by the complete def-path
  including its owning function/disambiguator, not by the short name CACHE.
- The initializer produces its own named VC and every supported access records
  the same StaticId. Different declarations have different IDs. No initializer
  setter is inserted in each calling module.

Run a flow-sensitive MIR audit over every local body, including bodies excluded
from normal proof translation. A reference created from a registered static is
labelled with its StaticId. Track simple moves, copies, shared reborrows and local
aliases to a fixed point. A join of the same ID is allowed; initially reject a
join of different IDs or an unknown source. The following uses are allowed:

- the direct standard `AtomicU8::load(Ordering::Relaxed)` receiver;
- the direct standard `AtomicU8::store(value, Ordering::Relaxed)` receiver;
- reference propagation within the audited body and ordinary end of borrow.

Recognize actual resolved sysroot method/variant DefIds. Reject all other uses,
including as_ptr, pointer casts, expose/address operations on that reference,
get_mut/into_inner, swap/fetch/update/CAS operations, return values, storage in an
aggregate or another static, closure/async capture, passing to any unregistered
helper, trait object, function pointer, generic utility, FFI or unknown callee.
A benign helper is still rejected in the first version: it needs an explicit
later interprocedural extension, not an inference that the alias is harmless.
Opaque macros and anonymous closures do not evade this because auditing uses
resolved MIR rather than textual source patterns. Raw address-of forms and
constant/promoted aliases must be found by the same static-reference discovery;
if not understood, reject them. Do not silently ignore a failed data-flow case.

Only Relaxed load/store need be accepted initially. Other valid atomic orderings
can be future generic extensions; invalid orderings must never be treated as
panic-free. Unsupported operations are diagnostics, not nondeterministic writes.
The current actual httparse cache requires only the supported pair.

### The normal-configuration audit is a required engineering gate

The registered cell must be confined in the actual runtime build too. Auditing
only cfg(creusot) would miss a writer under cfg(not(creusot)). Build a proof-side
manifest that includes the registered static identities, resolved access
operations, initializer identity/value, target/feature configuration and the
runtime-erased body representation of every function touching the static.

Add an explicit normal-cfg audit mode to the compiler driver. This mode must run
before the existing setup_plugin branches: both current ToWhy and WithoutContracts
paths add cfg(creusot), so reusing WithoutContracts does not audit normal Rust.
The audit-only callback must not inject that cfg. Audit all actual local bodies,
then require their static-access bodies to match the checked proof-side erased
ANF/THIR or another mechanically justified equivalent representation. A normal-only
writer, a different cfg branch containing a store, or a new alias use must fail.
Matching source spans, def-paths, manifests or operation counts alone is inadequate.

Reusing existing ANF/THIR erasure utilities is a plausible implementation route,
but the cross-build comparison and handling of supported std erasure adapters are
OPEN engineering requirements until implemented and tested. A JSON manifest does
not make this equivalence true. The manifest must be bound to current source,
compiler, target/features and proof output; stale results must not be accepted.

### Concrete compiler patch boundary

Use a dedicated patch and build directory on pinned Creusot commit
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`, nightly-2026-02-27. Suggested code slices:

1. New `creusot/src/static_atomic.rs`: registration resolution, eligibility
   checks, StaticId/reference-use dataflow audit, manifest I/O and the generic
   rule's metadata. Keep predicate/type validation here.
2. `creusot/src/ctx.rs`: distinct StaticAtomic item/registry, not Constant;
   `backend.rs` and `translated_item.rs`: emit the initializer VC as a real
   required proof module.
3. `translation/constant.rs`, `translation/function.rs` and
   `translation/function/terminator.rs`: recognize the registered reference
   origin and lower only the audited load/store operations. Keep the original
   runtime bodies and orderings available to the erasure/audit comparison.
4. `backend/clone_map/elaborator.rs` plus a small backend helper as needed: one
   shared static symbol and per-operation declarations with I(result)/I(value).
   Do not route through expand_constant or emit repeatable initialization code.
5. `creusot_args` options and `creusot-rustc/src/main.rs`: explicit manifest/audit
   mode, selected before adding verification cfg; `callbacks.rs`: normal-cfg
   audit-only callback using the resolved standard Rust body. Reuse erasure
   normalization utilities where demonstrated compatible.
6. A target-local reproduction script runs proof-side compile, required VCs and
   the normal-cfg audit for the exact same feature/target matrix. Initially keep
   the script separate from the general verifier until all tests pass.

This is a bounded specialization, not support for arbitrary global mutable
resources. No new generic Perm allocator, invariant namespace API or general
atomic history constructor is required for this value-only rule. It still
requires a real compiler patch and review of the confinement and erasure checks.
Do not describe it as already implemented or as a one-line static fix.

Build from an isolated archive/worktree of the pinned source and apply the
existing required backend patch plus the new static-atomic patch explicitly.
Do not modify the active compiler installation or silently omit existing patches.
Activate `/workspace/proof-tools/activate.sh`, use an isolated target directory
such as `/workspace/proof-tools/targets/httparse-static-atomic`, and build only
`cargo build --offline --manifest-path <isolated-source>/Cargo.toml -p creusot-rustc`.
Use the resulting binary through CREUSOT_RUSTC for proof-side runs. A normal-cfg
invocation must route through the new audit-only entry point; a proposed RUSTC
wrapper/script must preserve Cargo's rustc arguments and pass through dependency
crates that are not the manifest's target. Pin target rustc sysroot and record the
binary/patch hashes. Compiler tests and httparse-specific harnesses are the
appropriate narrow checks; do not reverify unrelated repository crates.

### CPU feature observations: precise standard boundary

The pinned runtime macro expands to a target-feature cfg branch or
`std_detect::detect::__is_feature_detected::{avx2,sse4_2}()`. Source inspection
confirms these generated functions call the standard check_for cache.
`std_detect` is a sysroot crate gated by `stdarch_internal`; on the pinned nightly,
an explicitly std-enabled contracts module can investigate direct extern specs
using that crate and exact resolved definitions. If source visibility prevents
that attachment, register those standard DefIds in the verifier rather than
trusting the crate-owned detect_runtime_feature body.

The standard contract connects each Boolean result to a stable process execution
capability predicate. For AVX2, this means usable execution capability including
OS XSAVE/XCR0 state, not just the raw CPUID AVX2 bit. The inspected standard
x86 detector enables AVX/AVX2 only under the relevant OS support checks. Stability
across calls/threads is an explicit runtime/platform premise already required
when caching feature detection and invoking target_feature code. It is not a
proved fact about arbitrary heterogeneous/hot-changing execution environments.

For compiled-in target features, the proof environment must record the same
execution-capability premise as Rust's target-feature build contract. The macro's
constant true branch then has a justified capability fact. Do not infer hardware
support merely because a proof was run on an x86-64 host. Model appropriate
instruction prerequisites on the actual SSE/AVX callers; ISA implication facts
must be correct (for example the needed SSE generations), and AVX2 calls must
consume the AVX2 capability. The detector's priority branches are ordinary
crate body proofs against these standard observations. The static I(0) proof is
independent of all CPU facts.

These standard detector/atomic semantics are the declared TCB, comparable to the
existing standard atomic contracts. I, initialization, all actual stores and the
whole detector/dispatch body are proved obligations, not new trusted algorithms.
NEON and other compile-time backend paths retain their separate target assumptions
and proofs; this x86 dispatch work does not discharge them.

### Acceptance sequence and tests

Gate A, before adding trusted load results: implement and exercise registration,
normal-cfg auditing, escape rejection and initializer/access inventory. Cases:

- direct accesses and local alias/reborrow to one static accepted;
- function-local CACHE accepted, two different CACHE declarations distinguished;
- raw address/as_ptr, returned/captured/stored ref, generic/indirect/unknown helper
  escape, cross-static alias join and unsupported atomic operations rejected;
- a cfg(not(creusot)) store or changed branch rejected by actual-body comparison;
- wrong type, duplicate/unresolved registration, mutable/exported static and
  unsupported initializer rejected;
- stale source/target/feature manifest rejected.

Gate B: emit the generic initializer/read/write VCs and prove the abstract
finite-history preservation lemma. Use two- or three-value predicates unrelated
to httparse. Correct initialization and stores must prove; bad initialization
and bad stores must fail the proof gate. An assertion that every read equals the
initializer must remain unproved when I permits another value: the abstraction
must not reset on read. No API may produce an exclusive permission, so duplicate
ownership is structurally absent rather than justified by a special axiom.
Unknown solver status is recorded as unknown, never a counterexample. An expected
negative test is successful only insofar as the verification pipeline refuses to
certify it; it does not by itself establish that the solver found a model.

Gate C: attach/test the two standard CPU observations under matching baseline
and forced-feature build configurations. Prove a tiny detector-priority caller
for all Boolean capability combinations, and an invalid unguarded SIMD caller
must fail its capability requirement.

Gate D: apply I(v)=0-or-choose to the unchanged actual runtime static and its
load/detect/store body. Prove initialization, detector, stores and exact returned
choice, then integrate actual guarded SIMD callers with common scanner contracts.
Require the normal-cfg audit and all required initializer/function VCs on the same
committed tree before declaring the static boundary closed.

The existing parameterized 15-VC result supplies a useful baseline; the new rule
and cross-cfg audit must pass their own acceptance gates. Implementation should
start with Gate A and small compile smoke tests, then Gate B/C, and only then the
httparse dispatch proof. Stop for another Astra review if alias flow, erasure
comparison or CPU attachment needs a broader scope than this specialization.

Driver compatibility note: the installed cargo-creusot frontend does not know
new command-line options. The first isolated compiler prototype may read explicit
driver-only registration/audit manifest environment variables before setup_plugin,
matching the intended crate name/target and passing dependency compilations
through normal callbacks. This avoids rebuilding the frontend just to pass a
manifest. If CreusotArgs/options are extended instead, add compatible defaults
for serialized options and rebuild/test the frontend when its parser must accept
new flags; do not assume building only creusot-rustc updates cargo-creusot too.

## 2026-10-05: `nth_bit` failures and the installed Z3 driver

The SSE helper split has isolated the remaining failures to bit observations:
scalar signed-byte/max/andnot helpers pass, whereas the low-bit cast clause times
out and complement clauses return `unknown` almost immediately. This is not yet
evidence that sixteen manual cases are necessary. The installed driver drops
standard bit-observation axioms while mapping the underlying operations to native
SMT bitvectors. A task export must establish which bridges actually survive
before changing the Rust contracts again.

Read-only source findings (no solver run by this consultation):

- Installed Why3 `stdlib/bv.mlw`, `BV_Gen`, defines `nth` with mathematical integer
  indices and least-significant bit numbered zero. `nth_out_of_bound` (line 190)
  specifies **False** for both negative indices and indices at least the width.
  This is not a Rust shift with a masked or wrapping shift count.
- `Nth_bw_not` (line 232) relates complement to Boolean negation only under
  `0 <= i < size`. `nth_bv_def` (line 598) defines bitvector-index observation by
  logical right shift, bitwise AND with one and comparison with zero.
  `Nth_bv_is_nth` / `Nth_bv_is_nth2` (lines 603/609) connect that observation to
  mathematical-index `nth`; the latter requires `0 <= i < 2^size`.
- The active driver family is under
  `/workspace/proof-tools/creusot-data/_opam/share/why3/drivers/`.
  `smt-libv2-bv.gen` removes `nth_out_of_bound`, the `Nth_bw_*` axioms and the
  shift-observation axioms while mapping bitwise operations to native SMT.
  `z3_bv.gen` additionally removes both `Nth_bv_is_nth` bridges for BV8 through
  BV256. `nth_bv_def` is not removed there. No direct `nth` syntax rule was found
  in the active driver imports. The actual SMT task, rather than source inspection
  alone, must confirm whether a remaining import/transformation supplies a bridge.

Root authorized a reversible **target-local** driver/config experiment after the
active proof session. Leave the installed Why3 drivers unchanged. First copy the
selected driver and its changed `.gen` dependencies, point imports unambiguously
at the local copies, and retain the necessary original true standard axioms.
Appending a later theory block cannot undo an earlier `remove prop`. Start with
BV16/BV32 observation bridges and the required generic complement/bounds axioms;
extend widths only if the emitted BV256 cast/conversion task requires them.
Do not add an assumed httparse helper contract.

Exporting tasks requires no prover: `why3 prove -D <driver> -o <existing-dir>`
prints SMT files. Supply the Creusot package load path, `verif` load path, and the
same `split_vc` transformation as the proof configuration. Compare original and
local-driver exports for the complement and narrowing leaves: the relevant `nth`
observations must have a retained defining bridge, rather than occur solely as
uninterpreted functions in the goal. Record exact commands, local driver hashes,
and task excerpts. A source-level driver hypothesis is not a successful proof.

If retaining the standard axioms still performs poorly, a second, explicitly
reviewed option is a width-specific native `nth` syntax rule. Its exact meaning
for width W is the Boolean expression

```
(and (<= 0 i) (< i W)
     (= (bvand (bvlshr x ((_ int2bv W) i)) (_ bv1 W)) (_ bv1 W)))
```

The integer bounds are essential: conversion to W bits wraps a large or negative
index, which would otherwise disagree with Why3's False result outside the valid
range. This mapping is a standard-theory/driver correctness boundary, not a
trusted parser algorithm. It must be checked against the installed theory and
actual emitted SMT, with W specialized to each used width.

Acceptance checks for either experiment include true and deliberately false
constant bit assertions at bit 0 and bit W-1, negative indices, i=W, and a large
index such as 2^W (which catches accidental wrapping). Include positive complement
and low-bit narrowing lemmas, plus incorrect complement/cast claims that must
not be certified. An `unknown` result only establishes lack of certification;
it is not a counterexample and must not be reported as one. The corrected path
must prove the intended true obligations as well as reject bad assertions.

Only after restoring/checking the standard observation semantics should further
body-checked lemma splitting be evaluated. A useful small interface takes one
index with `0 <= i < 16` and proves one bit equality; bounded universal statements
then compose it. The i32-to-u16 low-bit preservation fact does not need the
numeric `0 <= value <= 65535` premise, so separate it from the numeric equality
clause if mixed integer/BV arithmetic remains expensive. A finite sixteen-case
index split is a last small proof tactic, not a replacement for missing semantics.
The backend worker owns driver copies, task exports, and root-scheduled proof
runs; this consultation edits only this document.

## 2026-10-05: string literals and exact UTF-8 models

Two separate defects explain the current literal/EMPTY_HEADER failures:

1. `creusot/src/translation/pearlite/from_thir.rs` translates Boolean, integer and
   character THIR literals, but has no `LitKind::Str` arm; its fallback raises
   `Unsupported literal`. In contrast, runtime
   `creusot/src/translation/constant.rs:48` already decodes a Rust string constant's
   allocation bytes as UTF-8 and produces `Literal::String`.
2. `backend/ty.rs` represents `str` as Why3's primitive `string`, and
   `backend/term.rs` emits a primitive string literal. The repository standard
   contract `View for str` in `creusot-libs/creusot-std/src/std/string.rs` is an
   unrelated opaque function returning `Seq<char>`. Consequently, even a runtime
   empty literal gives no fact that its View is empty. Fixing only the THIR arm
   does not fix `.len()` or `.as_bytes()` proofs.

### Smallest coherent generic representation change

Represent an immutable Rust string's logical content directly as the existing
`Seq<char>` model, rather than maintaining primitive Why3 strings plus an opaque
conversion. This is a compiler abstraction change and must be tested as such,
not justified by adding a literal-specific trusted lemma.

The proposed isolated patch owns these files:

- `creusot/src/translation/pearlite/from_thir.rs`: add
  `LitKind::Str(symbol, _) => Literal::String(symbol.as_str().to_owned())`.
  Rustc has already decoded escapes/raw literal syntax. Byte strings and C strings
  are different types and must not silently use this arm.
- `creusot/src/backend/ty.rs`: lower `Str` to `seq.Seq.seq Char.t`.
  Shared references already lower to their referent's logical content; mutable
  references keep the existing borrow representation.
- `creusot/src/backend/term.rs`: lower every `Literal::String` through host Rust
  `.chars()`, emitting `Char.of_int` for each Unicode scalar and the same
  `Seq.create`/`FunLiteral` construction already used for `TermKind::SeqLiteral`.
  The sequence length is the number of Unicode scalars, not the UTF-8 byte length.
  Ensure the empty sequence has/inherits the `Char.t` element type.
- `creusot/src/backend/clone_map.rs`: a small `PreMod::Seq` entry mapped to
  `seq.Seq` permits explicit, dependency-tracked imports for both type and literal
  lowering, including files whose only string is empty.
- `creusot-libs/creusot-std/src/std/string.rs`: make `str::view` the existing
  `#[builtin("identity")]` operation. `Seq<&T>::to_owned_seq` demonstrates this
  builtin already. `DeepModel for str` remains `self.view()`. Do not change owned
  `String` to this representation; its existing View and Deref contracts compose
  with the new `str` representation.

No change to `translation/constant.rs` is required by this route: MIR constants
and Pearlite literals converge at the existing `Literal::String` lowering.
No new standard axiom or native SMT string operation is needed. Immutable string
content equality becomes sequence equality, as required for Rust `str` equality;
allocation identity/provenance remains the responsibility of the existing
reference/pointer model, not the character sequence.

The static compiler worker confirmed its current Gate A changes own only the new
static audit module, library registration, callbacks and driver branch. They do
not overlap these literal/backend files. Implementation must nevertheless use
root-assigned ownership and an isolated pinned compiler build, carrying forward
other required patches. This consultation makes no compiler edits.

### Existing byte bridge and discovered Unicode boundary defect

The repository standard contracts already express the correct byte relation:

- `str::as_bytes`: returned slice View equals `self@.to_bytes()`;
- `str::len`: returned integer equals the UTF-8 byte sequence length;
- `from_utf8` success: returned string View encodes exactly to input bytes;
  its error observer implies input is not `valid_utf8`;
- `from_utf8_unchecked`: requires an existing `Seq<char>` whose encoding equals
  the input bytes, and returns a string encoding to exactly those bytes;
- `valid_utf8(bytes)` is precisely the existence of such a character sequence.

`Seq<char>::to_bytes` is an open flat-map over `CharExt::to_utf8`; the latter has
an explicit standard one-/two-/three-/four-byte UTF-8 encoding body. Existing
`injective_to_bytes` is a declared trusted standard-library lemma, not a parser
lemma. No new trusted decoder or httparse literal fact should be introduced.
If exact character equality is needed after a byte roundtrip, use the existing
injectivity boundary explicitly, or prove a generic UTF-8 injectivity lemma
separately. Runtime decoder implementation remains in the standard-library TCB.
The parsed slice's range, lifetime/permission and valid-UTF-8 premise remain
ordinary obligations at the actual unsafe call.

Read-only inspection found an independent standard-prelude defect:
`prelude-generator/prelude.coma`, module `Char`, uses `< 0x10FFFF` in four places
(`to_int`, `to_of_int`, program `of_int`, and `of_BV256`). Rust permits U+10FFFF;
all four upper bounds must instead be `< 0x110000`, while retaining exclusion
`0xD800 <= code < 0xE000`. Installed
`share/why3find/packages/creusot/creusot/prelude.coma` has the same four bounds.
Update the source and use a matching isolated generated/package prelude for tests;
rebuilding only `creusot-rustc` does not update the installed Why3 package.
Do not certify full Unicode using the stale prelude. No replacement characters,
normalization, surrogate encodings or code-point truncation belong in literal
lowering. Rust `.chars()` supplies precisely Unicode scalar values.

The existing `size_of_val_logic_str` and `metadata_matches_str` contracts already
use `value@.to_bytes().len()`, preserving byte-oriented layout despite the
character-oriented content model. Executable MIR `PtrMetadata` currently supports
slice referents only; this proposal does not silently claim to add raw-str
metadata instruction support. If an actual caller needs it, use a separately
reviewed byte-length implementation, never `Seq<char>` length.

### Staged acceptance before parser integration

1. Build the isolated compiler and translate small literal-return functions and
   exact View/equality postconditions. Cover empty, ASCII, embedded NUL, escaped
   and raw literals, two-/three-/four-byte characters, U+D7FF, U+E000 and U+10FFFF.
   Check that runtime MIR literals and specification THIR literals lower to the
   same scalar sequence. Inspect Coma: no opaque `view_str` remains, the empty
   literal is a zero-length character sequence, and U+10FFFF has its correct
   numeric value. Rebuild contract metadata with the same compiler; do not mix
   cached old primitive-string interfaces with new sequence interfaces.
2. Prove those exact View/equality contracts, then byte lengths and exact
   `as_bytes()` results separately. Examples: empty -> `[]`, NUL -> `[0]`,
   U+00E9 -> `[0xC3,0xA9]`, U+20AC -> `[0xE2,0x82,0xAC]`, and U+10FFFF ->
   `[0xF4,0x8F,0xBF,0xBF]`. Distinguish a one-character four-byte string from
   byte length one. Deliberately wrong literal/byte claims must not certify.
3. Prove a generic valid-input caller of `from_utf8_unchecked` followed by
   `as_bytes`, preserving exactly the supplied bytes, and a safe `from_utf8`
   roundtrip for representative valid sequences. Test that unchecked conversion
   without the validity premise fails. Invalid concrete sequences should not
   satisfy `valid_utf8`: isolated continuation `[0x80]`, overlong `[0xC0,0x80]`,
   surrogate `[0xED,0xA0,0x80]`, truncated `[0xE2,0x82]`, and above-max
   `[0xF4,0x90,0x80,0x80]`. Encoding-range/concatenation helper lemmas may be
   needed to discharge these negatives; derive them from the generic encoding
   body instead of assuming invalidity for selected bytes.
4. Only then re-run actual EMPTY_HEADER and error-description contracts and
   connect actual parser-produced byte slices to returned strings. This repairs
   the string-content boundary; it does not by itself prove formatting traits,
   buffer ownership, every parser output path or the whole crate.

Root schedules proof runs. Record compiler/contract/prelude hashes together with
results. Expected-negative cases succeed as tests only when the pipeline refuses
to certify them; distinguish solver unknown from a concrete counterexample.

### Isolation and reproducible rebuild details

Do **not** change the active shared `creusot-std` View to identity while other
workers still use the old primitive-string compiler. Keep the standard-library
change in a copied `creusot-libs` tree and point only the isolated probe manifest
at that copy. Copy the whole local-library tree so its sibling path dependencies
remain consistent. Use a fresh Cargo target directory; compiled rlibs and Creusot
metadata from the two representations are incompatible. Actual httparse source
may be included by the isolated harness without changing its canonical manifest.
No blind global path replacements, shared-package symlink replacement, or
canonical compiler/contract activation is part of the experiment.

Start the compiler source worktree/archive from pinned commit
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`, preserving the required existing
narrow-cast patches. Store the string patch independently of the static patch.
`tools/creusot-toolpatch/scripts/build-creusot-rustc.sh` documents the pinned
archive/apply/offline `cargo build -p creusot-rustc` pattern; use a dedicated target
such as `/workspace/proof-tools/targets/httparse-string-model` and the existing
nightly `2026-02-27`. The installed frontend supports `CREUSOT_RUSTC` to select
the isolated executable, so replacing the globally installed executable is
unnecessary. A future combined static+string compiler must apply both patches
to the same pinned baseline and re-run each narrow acceptance harness before
canonical integration; a successful string-only build is not evidence about
the combined compiler.

Prelude isolation also matters. `prelude-generator/src/main.rs` writes to
`<source-worktree>/target/creusot/packages/creusot/creusot`, independently of the
Cargo compilation target directory. Its `prelude.coma` copy is timestamp-based.
Generate in a fresh isolated tree (or explicitly verify the resulting file hash),
and configure the test to load a copied Creusot Why3 package containing that
updated prelude. Merely setting `CARGO_TARGET_DIR`, editing the generator source,
or rebuilding rustc does not select that package. Record which actual package
path the solver/Why3 command loaded; do not assume environment search order.

Implementation/proof checkpoints should remain small: first syntax+sequence
lowering+identity in the isolated compiler/library with empty and ASCII View
proofs; then the four Unicode-bound repairs and non-ASCII byte/boundary proofs
using the isolated package; then actual EMPTY_HEADER/Error description body
contracts. Invalid-input UTF-8 cases and generic conversion callers follow
without adding parser-specific trusted facts. Root grants all solver runs and
owns activation, commits and push.

## 2026-10-05: chunk loop blocked by a missing model interface

The current outer parser problem is a missing semantic premise, not evidence
that its timeout should be increased. In the actual generated
`contracts-harness/verif/httparse_contracts_harness_rlib/chunk/parse_chunk_size.coma`:

- `scan` is an uninterpreted function with only `scan_spec`, which constrains
  result ranges but states neither EOF behavior nor the recursive step equation;
- `parse_chunk_size_model` is a bare uninterpreted function with no postcondition;
- neither `scan_def` nor `parse_chunk_size_model_def` is present;
- the complete `step` classification body and its useful invariant-preservation
  postcondition are present.

Thus the loop's initial residual equality, preservation of that equality, and
EOF result cannot follow from the current caller interface. In particular,
`parse_chunk_size_model` could denote an unrelated outcome under those premises.
The successful helper proofs do not automatically reveal private logical
function definitions to a sibling module.

### Three body-checked, finitely invoked ghost lemmas

Keep recursive `scan` and `parse_chunk_size_model` private-definition `#[logic]`
functions. Add the following ordinary `#[check(ghost)]` lemma functions in the
same `verification_chunk` module, where their bodies can use the definitions.
Use `Snapshot<Seq<u8>>`, `Snapshot<Int>` and `Snapshot<ChunkState>` parameters;
this avoids translating program computation on logic-only `Int` values and
avoids manufacturing ghost permissions. Empty bodies are appropriate if the
postconditions discharge from one local unfolding. They are **proof obligations**,
not trusted functions, extern contracts, or free axioms.

1. `scan_initial(input: Snapshot<Seq<u8>>)` has no extra caller requirement and
   ensures
   `outcomes_equal(scan(*input, 0, initial_state), parse_chunk_size_model(*input))`,
   where `initial_state` has size/digits zero, `in_digits=true`,
   `in_extension=false`. Its body only needs the model's initial definition;
   `hex_capacity(0) >= 1` already establishes the scan domain.
2. `scan_eof(input, pos, state)` requires the existing scan-domain facts and
   `*pos == input.len()`, and ensures
   `outcomes_equal(scan(*input,*pos,*state), ChunkOutcome::Partial)`.
3. `scan_unfold(input, pos, state)` requires exactly the existing scan-domain
   facts with `0 <= *pos && *pos < input.len()`, and ensures:

```
match step(*input, *pos, *state) {
    ChunkStep::Continue { next_pos, next } =>
        outcomes_equal(scan(*input, *pos, *state),
                       scan(*input, next_pos, next)),
    ChunkStep::Complete { consumed, size } =>
        outcomes_equal(scan(*input, *pos, *state),
                       ChunkOutcome::Complete { consumed, size }),
    ChunkStep::Partial =>
        outcomes_equal(scan(*input, *pos, *state), ChunkOutcome::Partial),
    ChunkStep::Invalid =>
        outcomes_equal(scan(*input, *pos, *state), ChunkOutcome::Invalid),
}
```

The common state-domain facts are `0 <= digits <= 16`,
`0 <= size < hex_capacity(digits)`, and `!(in_digits && in_extension)`.
The caller already maintains them; do not strengthen the public parser's
preconditions. Recursive-call well-definedness in `scan_unfold` follows from the
already proved `step` postcondition. The lemma reasons about one transition,
not every suffix of the input.

Use these as finite program ghost calls. Schematically, inside cfg(creusot):

```
ghost!(verification_chunk::scan_unfold(
    snapshot!(buf@), snapshot!(cursor@), snapshot!(state.deep_model())
));
```

Call `scan_initial` before establishing the loop, `scan_unfold` at the top of
each loop iteration **before moving** the current runtime state into
`step_chunk_size`, and `scan_eof` after the loop. Snapshots preserve the old state
without cloning the runtime state or changing runtime instructions. Adapt macro
imports in the actual shared file and verify erasure/normal compilation.

The actual step's existing `steps_equal` contract identifies the model step with
the matched runtime result. In Continue, that relation and `step_spec` provide
`next_pos=cursor+1`, bounds, the next state's validity and its arithmetic bound;
`scan_unfold` provides the residual equality. Transitivity with the existing
loop invariant gives the next loop invariant. In each terminal branch the same
one-step postcondition gives the exact outcome, including both consumed count
and size. At EOF, cursor bounds plus the loop exit imply equality with the input
length, so `scan_eof` completes the existing Partial postcondition.

Keep `outcomes_equal`/`states_equal` transparent as currently defined. If their
match-based encoding alone remains a composition obstacle, add a tiny
body-checked equality/transitivity lemma; do not weaken Complete to mere success
or erase the consumed offset. Avoid placing an unconditional recursive equation
on scan's exported global contract: that can trigger repeated unfolding on every
new suffix term. An ordinary ghost lemma call exports its postcondition only at
that call site.

### Staging and remaining performance boundary

Translate the three lemmas first. Inspect their Coma to confirm their own bodies
can see the local model definitions and that the outer caller gains finite calls
with these postconditions while `scan`/`parse_chunk_size_model` remain opaque.
Prove the three small targets individually under root's queue, then a one-step
representative composition (a symbolic model-valid input/state passed through
the actual runtime step and the new ghost interface), then the outer loop.
Do not retry the old premise-deficient outer goal unchanged.

The existing `step` function is globally `#[logic(open)]`, so its full body may
still occur in the caller Coma; this consultation does not claim otherwise.
The first remedy is the missing finite recurrence interface. If that interface
passes but outer VCs still repeatedly expand the already-proved byte classifier,
consult again about relocating/encapsulating the transition interface. Changing
step to opaque immediately would invalidate the existing runtime step proof's
access to its specification body, so avoid that unrelated change in this first
small patch. Root schedules solver runs; the chunk worker owns implementation.

## 2026-10-05: actual `Bytes::next` suffix split and missing frame

Read-only inspection found a prerequisite missing before the sequence lemma:
`src/iter.rs`'s current `bump` contract promises only
`(^self)@.cursor == self@.cursor + 1`. The input/mark/end frame belongs to
`advance`, not to `bump`; callers do not inherit a callee-of-callee's contract.
The generated actual `next.coma` confirms this: the imported `bump` definition
at lines 226–234 has only the cursor postcondition. `cursor_produces` separately
requires input and end preservation, so these facts cannot be supplied by any
sequence decomposition proof.

First add a body-checked full frame to `bump`: unchanged input, mark and end,
plus cursor advanced by one. Its existing body delegates to `advance(1)`, whose
contract already entails exactly these facts. Re-prove this strengthened actual
body; do not assume framing simply because the method's source currently changes
only a pointer.

There is no analogous opacity defect in `cursor_remaining`/subsequence.
`cursor_remaining` is open and the generated caller contains its definition as
`Seq.([..]) input cursor end`. `Seq::subsequence` is a builtin for Why3's
`seq.Seq.([..])`, whose contract supplies length `end-start` and element
`result[k]=input[start+k]` for every in-range index. `singleton` and concatenation
likewise supply their length/index contracts. `Seq::ext_eq`, already present in
`creusot-std`, maps to `seq.Seq.(==)`, which equates length and all in-range
indices and implies ordinary sequence equality.

### Generic body-checked head decomposition

A small ghost function can expose the exact algebra required by Iterator::next:

```
#[check(ghost)]
#[requires(0 <= *lo && *lo < *hi && *hi <= s.len())]
#[ensures(s.subsequence(*lo, *hi) ==
    Seq::singleton(s[*lo]).concat(s.subsequence(*lo + 1, *hi)))]
fn subsequence_head<T>(s: Snapshot<Seq<T>>,
                      lo: Snapshot<Int>, hi: Snapshot<Int>) {
    // Proof assertions only; no trusted marker and no recursion.
}
```

Use snapshots `left = s[lo..hi]` and
`right = singleton(s[lo]) ++ s[lo+1..hi]` in its body. Four compact assertions
are enough as a proposed proof structure (each is a checked goal):

1. left and right have the same length;
2. their elements at zero agree;
3. for every `k` with `0 < k < hi-lo`, their elements at k agree;
4. `left.ext_eq(right)`.

For the positive-index case, right[k] is the tail's element at k-1, hence
`s[(lo+1)+(k-1)] = s[lo+k]`. All arithmetic is linear; no induction or unfolding
of the iterator implementation is needed. The final `ext_eq` fact supplies the
ordinary equality postcondition. Adapt explicit snapshot dereferences to Rust
syntax and inspect the emitted extensional-equality predicate, rather than
assuming a proof assertion is accepted.

Why3's separate `seq.FreeMonoid` module contains related proved lemmas
`cons_dec`, `cons_def`, `double_sub_sequence` and `cat_dec`. The current caller
imports `seq.Seq`, not that lemma module, so those facts are not automatically
available. The direct extensional proof above uses the already imported standard
semantics and avoids adding a new builtin/trusted lemma merely to import them.

In actual `next`'s Some branch, instantiate `subsequence_head` on a snapshot of
the state before bump. Peek establishes cursor < end and the returned byte's
exact input value; the type invariant supplies the other bounds. The strengthened
bump postcondition then identifies the final input/end and cursor+1 with the
lemma's suffix. This establishes the full existing `produces` postcondition;
keep its exact sequence relation and input/end frame unchanged.

Stage the tiny algebra helper and strengthened bump body before retrying next.
If a remaining failure persists, inspect that next's Coma actually exports the
new frame and contains the finite lemma call before adjusting solver limits.
The memory worker owns implementation; this consultation runs no prover and
adds no trusted algebra.

## 2026-10-05: empty string lowering and proof-runner isolation

The first isolated string proof attempt exposed a syntax issue before any VC:
Why3 rejects `Seq.create 0 [||]`. `why3/src/printer.rs` prints `Exp::FunLiteral`
by enclosing its list in `[|` and `|]`, even when the list is empty. The generic
empty-character-sequence representation should instead be the existing typed
empty constant:

```
(Seq.empty: Seq.seq Char.t)
```

In the empty `Literal::String` case, the existing AST APIs can express this as:

```
Exp::qvar(self.names.in_pre(PreMod::Seq, "empty")).ascribe(
    Type::qconstructor(self.names.in_pre(PreMod::Seq, "seq")).tapp([
        Type::qconstructor(self.names.in_pre(PreMod::Char, "t"))
    ])
)
```

Keep the existing `Seq.create`/nonempty `FunLiteral` lowering for nonempty
strings. This uses standard sequence semantics, preserves type information in
unconstrained contexts, and needs no empty-string axiom, synthetic default
character or function body. Register both imports through `in_pre`, including
when every string in a file is empty. Rebuild/update patch hashes and retranslate;
then use Why3 parse/type-only preflight before another proof slot. This small
fix is for the string-literal lowering path; it does not claim that every
pre-existing generic `SeqLiteral([])` path was separately checked.

The observed shared `why3.conf` refresh has a separate, confirmed cause in the
pinned frontend:

- `CreusotPaths::new` in `creusot-setup/src/lib.rs` applies `CREUSOT_DATA_HOME`
  only to the data directory. The config directory still comes from
  `directories::ProjectDirs`, hence `XDG_CONFIG_HOME` on Linux.
- `cargo-creusot/src/why3_launcher.rs::check_why3_conf_exists` regenerates that
  config whenever it is missing or older than the data directory.
- `why3find_wrapper.rs::raw_prove` overrides the subprocess `WHY3CONFIG` with
  `paths.why3_conf()`. Supplying an isolated `WHY3CONFIG` to cargo-creusot does
  not bypass this logic.
- `activate.sh` unconditionally sets `XDG_CONFIG_HOME` to the global proof-tools
  config directory; `run-proof.bash` sources it again. The current isolated env
  array provided data/config-file values but no post-activation XDG config value.

Consequently, a new isolated data directory combined with a global XDG config
can refresh the global config even though the caller supplied an isolated
`WHY3CONFIG`. This is a tooling side effect, not proof evidence.

Prefer two stages: use cargo-creusot only for translation, and invoke the
absolute `why3find` executable directly for the selected generated Coma files.
After the shared queue wrapper has sourced activation, set the isolated
`WHY3CONFIG`, `DUNE_DIR_LOCATIONS=why3find:lib:<isolated>/share/why3find`, and
isolated XDG config/cache paths on the executed process. The direct executable
has no cargo-creusot setup call. Ensure the queue wrapper validates the same
config that the actual prover will read (one prover, 1000 MiB); it is insufficient
to validate a global file and then silently select a different isolated one.

A direct `why3 prove --type-only -C <isolated-conf>
-L <isolated>/share/why3find/packages/creusot -L verif <file.coma>` performs
parse/type preflight without running a solver or triggering cargo setup.
For the subsequent root-scheduled direct why3find run, record the selected
package path and corrected Char prelude hash. Compare the shared config hash
before and after to establish the intended isolation; do not modify the shared
configuration as part of a string-model experiment. The message worker owns
these implementation/runner changes. This consultation used read-only inspection
and edited only this document.

## 2026-10-05: executable Gate B for the static AtomicU8 invariant

Gate A's compiled audit is useful infrastructure, but its manifest and self-
reported source digest cannot authorize load assumptions on their own. Gate B
should implement the generic invariant rule directly at audited MIR operations,
using existing FMIR/Coma assertion and nondeterminism machinery. It does not
need to construct an AtomicU8 history object or a persistent exclusive ghost
permission. The registered static identity stays compiler metadata.

### 1. Carry an enforced audit result into translation

Extend `static_atomic.rs` to return a typed resolved record, rather than only
write text: unique static DefId, predicate DefId, initial u8, checked alias roots,
and operation sites. Store it in `TranslationCtx` (or pass it explicitly to the
body translator) only after the normal/proof audit correspondence succeeds.
Resolve/check every operation against the **actual MIR body being translated**;
locations in the pre-analysis built MIR may not be identical after compiler
passes. Recompute the alias/site map at that stage and check coverage, rather
than blindly reusing a built-MIR block number.

Initially use a fail-closed correspondence check: accept only access bodies whose
normal and proof executable MIR match under explicitly implemented erasure and
stable-name normalization. A strict comparison may accept simple no-ghost
fixtures first and reject richer source. That is a legitimate small acceptance
slice; ignoring differing bodies is not. Compute source/input fingerprints from
compiler-observed file contents/source hashes, not a digest copied from the
manifest environment. Bind both audits to compiler build, crate/package, target,
features/cfg and dependency snapshot. Include macro/include/environment inputs
where supported; otherwise reject unaccounted inputs in this first specialization.
Do not weaken the correspondence check merely to admit the dispatch macro.

Require that every registered-static accessor is actually translated and checked:
reject trusted, excluded, missing-body and otherwise skipped accessor bodies.
An unproved writer invalidates every load's invariant assumption. Logic/spec-only
reads of the raw static are outside this initial rule. Require each predicate to
be a total, checked, nonprophetic `fn(u8)->bool` with no generic arguments or
nontrivial preconditions; reject builtin/trusted or opaque-dead replacements.
Its transitive logical dependencies must obey the existing checked logic rules
and the declared stable standard-observation boundary. No mutable-state/history
observer may make I change between initialization, store and load.

### 2. Erase only audited receiver construction

The static object is never exposed to the logical program under Gate A's accepted
shape: only its identity selects a load/store invariant. In
`translation/function/statement.rs`, skip precisely certified assignments that
seed or copy/reborrow a registered static's shared-reference handle. Keep their
StaticId in the compiler alias map; do not lower those operands through ordinary
constant translation. Ensure no surviving FMIR expression or borrowed-place
resolution references an erased alias. Unused alias locals may be removed by the
normal dead-local pass, or explicitly omitted after checking all their uses.

This is not blanket erasure of AtomicU8 statements. Value computations, branch
conditions, stored values and load destinations remain ordinary runtime MIR.
The alias escape audit is what makes receiver-handle erasure semantics-preserving.
An unknown use must be rejected before it can disappear. No raw pointer,
reference equality, identity inspection or alias escape is admitted.

### 3. Lower actual atomic calls using existing proof primitives

In `translation/function/terminator.rs`, intercept a certified load/store before
translating its receiver operand and before the generic contractless-external
call branch. Recheck exact resolved AtomicU8 method, receiver StaticId, Relaxed
ordering, operand types, and normal return edge. Initially reject unsupported
projected destinations or unusual unwind behavior rather than guess semantics.

A dedicated FMIR `StaticAtomicLoad`/`StaticAtomicStore` statement carrying
StaticId, predicate DefId and the real destination/value is a clear first
implementation. Update both FMIR visitors and exhaustive optimizer matches:
`translation/fmir.rs`, `backend/optimization/simplify_temps.rs`,
`remove_dead_locals.rs`, and `invariants.rs`. A load writes its destination;
store operands count as reads; the store check is an effect that dead-code
optimization must retain. An alternative narrowly scoped RValue for the load
plus an existing Assertion for stores is also possible, but must preserve the
same write/liveness/check behavior.

Concrete backend mechanisms already exist in `backend/program.rs`:

- Load: allocate a fresh local name and emit
  `IntermediateStmt::Any(fresh, lower.ty(u8))`, then
  `IntermediateStmt::Assume(I_static(fresh))`, then normal
  `lower.assignment(destination, Exp::var(fresh), ...)`.
  `IntermediateStmt::Any` lowers to Coma's nondeterministic continuation binder.
  **Do not use `Any.any_l(())`: that is a pure logical application and cannot
  stand for a fresh independent atomic observation on each execution.**
- Store: evaluate the actual u8 operand once; emit
  `IntermediateStmt::Assert(I_static(value))` with an explanation naming the
  static and operation site, then assign unit to the actual call destination.
  Equivalently use FMIR `StatementKind::Assertion { check: true, assume: true }`
  after materializing the value as a term. Never use assume-only for stores.
- Build the predicate application with the resolved DefId using existing
  `Term::call` and `lower_pure`, or the corresponding dependency-aware name
  application. Use the registered function's actual definition/contracts;
  never synthesize an assumed httparse detector/cached-value predicate.

The load forgets which valid value is current; stores have no additional modeled
observable state beyond their required preservation check. Thus two loads are
not constrained equal, and even after a store a later load may return another
I-satisfying value. This overapproximation supports a relaxed cache while
avoiding false latest-read guarantees. It gives no Perm, history token or
permission to dereference a raw pointer.

### 4. Emit a separate initializer VC, never a constant setter

Every registration must generate one named, independently checked goal
`I_static(initial_u8)` with no static-load assumption. The initializer byte comes
from Gate A's checked direct `AtomicU8::new(const_u8)` and compiler constant
interpretation, not the manifest. Preserve unique complete DefPaths for different
function-local statics even when their short names are both CACHE.

Add a narrowly recognized `ItemType::StaticAtomic` (or an equivalent explicit
registered-static branch before the unsupported-kind dispatch) in `ctx.rs` and
`backend.rs`. It emits only this initializer proof module. Unregistered statics
retain the existing unsupported error. It must never reuse `ItemType::Constant`,
`expand_constant`, or a call-setter path for the static value.

Reuse the dependency/goal pattern in `backend/logic.rs`:
`Dependencies::new`, lowering the predicate application, `provide_deps`, and
`setters.mk_goal(unique_vc_name, goal)` produce a normal Why3 goal/file module.
Ordinary immutable constants used by I may have their existing dependency
setters; the registered atomic itself must not be one of those dependencies.
If source-item naming requires a new item-kind case in `clone_map`, give it only
an identity/name for the initializer module and reject any attempt to elaborate
it as a program value. This avoids silently constructing a fresh mutable object.
Use the existing FileModule/TranslatedItem output machinery so initializer goals
are visible to the normal proof runner and cannot disappear in a side report.

Do not place `I(initial)` as an axiom next to that goal. Load assumptions are
conditional on the complete program's initializer/store obligations being
proved; the verification result must retain that obligation dependency.

### 5. Mandatory coverage and proof closure

Emit a compiler-generated obligation index containing each registration's
initializer module and every translated accessor module/site. Verify at end of
translation that every audited operation was lowered exactly once and every
initializer VC was emitted. Fail if a source body is filtered out or its mapped
operation is missing/extra. The certification wrapper must require successful
fresh proofs of the whole listed set, plus predicates' required definition VCs,
on the matching compiler/source/audit snapshot. A selected function proof that
uses load invariants is conditional evidence until this set closes; it cannot
be reported as a complete static proof while the bad-store or initializer goal
is omitted. Stale/missing proof files and unknowns fail the closure gate.

Soundness is the standard induction over writes to one initialized static:
I(initial) holds; each actual store preserves the same stable I; a relaxed load
observes a value supplied by initialization or a store, hence satisfies I.
A small checked generic finite-history lemma can record this argument independently
of httparse. Rust atomic initialization/read-from semantics and process-stable
standard CPU observations are explicit standard/compiler TCB assumptions. The
registered I, detector priority, initialization and every actual write remain
proved obligations, not new trusted application algorithms.

### Small acceptance sequence for the implementation worker

1. Translate a private static with constant invariant `v==0 || v==1`, init0,
   direct load/store only. Inspect generated Coma for a separate initializer
   goal, fresh nondeterministic load binder, invariant assumption and actual
   store assertion. Parse/type-check before requesting a solver slot.
2. Positive proof batch: init0, store1, assertion that a load is 0 or1. Negative
   batch: init2, store2, an assertion that load must equal initializer0, and an
   assertion that two loads must be equal. The last two must remain uncertified
   because the rule deliberately forgets read order/current state.
3. Two statics with disjoint predicates and distinct local CACHE names; verify
   they select different invariants. Include deliberately wrong cross-static
   conclusions and repeated/loop loads to expose accidental pure-value reuse.
4. Reject audit drift, forged/self-reported-only hashes, skipped/trusted writer,
   missing initialization/site coverage, unresolved aliases, unsupported ops,
   and a predicate with `requires(false)` before producing a certifiable result.
5. Only after this generic slice closes, add the standard stable CPU observation
   contracts and actual dispatch body. The original exact dispatch/full-backend
   goal remains unchanged; passing Gate B fixtures is not its proof.

The static worker owns compiler/code changes. This consultation inspected the
packaged Gate A implementation and compiler APIs without changing them or
running a solver.

## 2026-10-05: derived Error equality/Debug contracts and Unicode proof status

The isolated string model reveals two pre-existing derived-method proof gaps;
these are not evidence of an incorrect string or Error DeepModel representation.
The generated Error model maps its seven variants to seven distinct model
constructors, as intended.

`impl_PartialEq_for_Error/eq__refines.coma` currently asks
`forall self,rhs,result. result = (deep_model(self)=deep_model(rhs))` with no
implementation-postcondition antecedent. `impl_Debug_for_Error/fmt__refines.coma`
similarly asks that an arbitrary final formatter extend the initial one, without
an implementation postcondition. These goals are too strong because the local
built-in derived implementations have no attached contracts. Inspection of
`translation/specification.rs::inherited_extern_spec` explains this: it returns
None immediately for a local definition, even though the standard trait has an
extern contract. `translation/traits.rs` then checks impl-post implies trait-post;
an empty impl-post gives exactly the impossible goals above.

The Debug body itself is present and branches over all seven variant names,
then calls `Formatter::write_str`. The existing standard write_str contract
already says the formatter appends a prefix of the supplied UTF-8 bytes and
appends all of them on Ok. It entails the existing Debug trait's
`formatter_extends` postcondition. No weaker formatter contract or new trusted
Error formatting fact is required to prove that refinement.

### Preferred fix preserves the original derived bodies

Root prefers verifying the original built-in derived Eq body in both normal and
proof configurations. A narrowly generic compiler change should allow local
`automatically_derived` implementations, when their method has no explicit
contract, to inherit the existing standard trait extern contract. Inspect the
containing impl's rustc `automatically_derived` attribute. Initially scope the
change to supported PartialEq::eq and Debug::fmt if needed. Preserve argument
renaming, generic substitution and trait predicates through the existing
`inherited_extern_spec` machinery. The resulting local implementation must still
be translated/body-checked with that postcondition, and its refinement VC must
have the postcondition antecedent. Never mark it trusted or replace its body by
an assumed declaration.

This change does not make the implementation contract true automatically: it
moves the obligation onto the real method body, where it belongs. Require both
the body and refinement files in the proof index. A deliberately wrong local
implementation carrying the same automatically-derived metadata must fail its
body postcondition, rather than inherit a free proof.

The current derived `eq.coma` has the separate body blocker `bb0 = false any`.
The translation warning/actual callee must confirm the suspected uncontracted
`core::intrinsics::discriminant_value`; do not label that call confirmed from
`false any` alone. If confirmed, add a generic exact intrinsic lowering in
`translation/function/terminator.rs` before contractless-external handling.
Recognize the actual rustc intrinsic DefId/metadata, not a user function with the
same short name.

The pinned core source specifies: return the semantic discriminant of the active
variant, or zero when T has no discriminant; this is a safe intrinsic with no
additional user safety precondition. Existing `make_switch` in the same compiler
file already obtains `AdtDef::discriminants(tcx)`. Lower a concrete supported enum
argument to a match on its Why3 constructors, returning those compiler-evaluated
discriminants in the exact intrinsic result integer type. Do not substitute the
variant's ordinal when explicit discriminants exist. Preserve signed
representation/target width; do not treat negative values as unsigned literals.
Do not interpret physical enum/niche memory tags: they need not equal semantic
discriminants.

A first small implementation can support concrete fieldless enums, including
Error, and reject other/unresolved types explicitly. Add positive and negative
fixtures for implicit tags, explicit nonconsecutive tags, signed negative tags,
and equal/different variants. Expand data-carrying and non-enum cases only with
correct typed semantics. Check the original equality Coma contains both actual
intrinsic-result computations and Boolean comparison, no `false any`, and the
exact inherited DeepModel postcondition. The normal Rust structural derive and
its constant-pattern capability remain intact.

The existing `creusot_std::std::cmp::PartialEq` derive (also exported by prelude)
is a useful diagnostic alternative: it emits a match body and exact DeepModel
postcondition. But selecting it only under cfg(creusot) verifies a different
body. Root explicitly requires treating that as temporary refinement evidence
with an equivalence bridge still open, not closure of the original actual-body
proof. Replacing PartialEq by a handwritten implementation in all configurations
can also lose the built-in structural-equality marker used by constant patterns,
so it is not the preferred repair.

A local extern_spec attachment is another annotation experiment: source inspection
shows local targets can receive such specs and ordinary local body translation
is not automatically skipped. Nevertheless it counts only if generated output
retains and proves the actual target's body postcondition and the proof index
requires it; a caller-only assumed declaration would not close this gap. Generic
derived-contract inheritance is the preferred reusable implementation path.

The string patch and static Gate B may both modify the compiler Call branch.
Their scratch trees are separate, but the eventual combined compiler must merge
these intercepts explicitly and rerun both acceptance suites on the same pinned
baseline.

### Unicode facts versus solver outcome classification

Read-only inspection confirms the current literal lowering is exact for the
requested examples: é=233, €=8364, U+D7FF=55295, U+E000=57344, and
U+10FFFF=1114111. Expected byte arrays in the generated goals are correct.
The isolated Char prelude uses `<0x110000` and excludes the surrogate interval.
The explicit `to_utf8_char` formula agrees with the standard UTF-8 encoding.

The saved `scalar_boundaries/proof.json` and `three_byte_scalar/proof.json` contain
null VCs. That means unproved; it does not distinguish timeout, Unknown or an
Invalid/counterexample result. The probe's `proof-positive.log` encountered during
inspection was an older offline-dependency failure, not the live run's solver
log. Obtain the current prover-results/session artifact before classifying the
red marks further. No counterexample was established by this consultation.

The generated task exports `utf8_byte` only as an opaque function with an exact
numeric postcondition, so its recursive implementation is not directly unrolling
in these callers. The recursive `flat_map_char` defining axiom is present.
For a small next proof split, prove each character's `to_utf8` equality first;
then use the existing body-checked `Seq::flat_map_singleton` and
`flat_map_push_back` lemmas to compose string bytes. This isolates encoding
arithmetic from sequence extensionality/concatenation and preserves exact expected
bytes. It is a proposed proof decomposition, not a new assumed encoding fact.
This consultation edits documentation only and runs no solver.

## 2026-10-05: concrete checked UTF-8 lemma staging

The current positive Unicode goals combine three independent steps: Unicode
scalar encoding, flattening a character sequence, and equality with a finite
byte sequence. Keep the expected byte equations unchanged and separate these
steps before another solver attempt. The copied `CharExt::to_utf8` has its exact
branch formula open, `utf8_byte` exposes its exact numeric postcondition, and
`Seq<char>::to_bytes` is the open `flat_map` definition. No additional encoding
axiom is needed.

### First prove the five scalar encodings without string flattening

A representative ordinary ghost helper is:

```rust
#[check(ghost)]
#[ensures('\u{20AC}'.to_utf8() == seq![0xE2u8, 0x82u8, 0xACu8])]
fn euro_encoding() {
    let encoded = snapshot!('\u{20AC}'.to_utf8());
    proof_assert!('\u{20AC}'@ == 8364);
    proof_assert!(encoded.len() == 3);
    proof_assert!(encoded[0]@ == 226);
    proof_assert!(encoded[1]@ == 130);
    proof_assert!(encoded[2]@ == 172);
    proof_assert!(encoded.ext_eq(seq![0xE2u8, 0x82u8, 0xACu8]));
}
```

These are checked assertions, not assumptions. The three byte facts follow from
the open encoding formula and `utf8_byte`'s postcondition; if a trigger is absent,
explicitly assert the needed `utf8_byte(226)@ == 226` (and the other actual byte
values) as a separate checked goal. Do not reveal the recursive byte-construction
implementation or perform hundreds of successor unfoldings.

Use the same short proof structure for:

| Scalar | Integer value | Exact bytes |
|---|---:|---|
| U+00E9 | 233 | 195, 169 |
| U+20AC | 8364 | 226, 130, 172 |
| U+D7FF | 55295 | 237, 159, 191 |
| U+E000 | 57344 | 238, 128, 128 |
| U+10FFFF | 1114111 | 244, 143, 191, 191 |

The helpers may initially live in the isolated proof harness: they establish
specific positive boundary tests from the generic standard encoding body and
are not trusted parser facts. Prove one helper first, then the others. A failure
of the final ext_eq clause after all length/index clauses pass is a sequence
extensionality interface issue; a failure of an index clause instead isolates
character arithmetic/byte-construction facts.

### Then expose two existing generic flat-map laws

The copied `Seq` implementation already has body-checked logical methods
`flat_map_singleton` and `flat_map_push_back`. Use thin logical wrappers:

```rust
#[logic]
#[ensures(Seq::singleton(c).to_bytes() == c.to_utf8())]
fn utf8_singleton(c: char) {
    Seq::flat_map_singleton(c, |x: char| x.to_utf8());
}

#[logic]
#[ensures(s.push_back(c).to_bytes() == s.to_bytes().concat(c.to_utf8()))]
fn utf8_push_back(s: Seq<char>, c: char) {
    s.flat_map_push_back(c, |x: char| x.to_utf8());
}
```

The logic macro performs the usual mapping conversion for the closures, as in
`to_bytes` itself. Check translation/type inference before a proof run. Require
body-proof evidence for the underlying helper instantiations as well as the
wrapper VCs; importing their postconditions alone is conditional evidence.
The existing `flat_map_push_back` proof decreases `s.len()`, recursively calls
itself on `s.tail()`, then proves
`tail(s).push_back(c) == tail(s.push_back(c))`. This is a generic sequence
induction, independent of Unicode.

If the stock push-back lemma needs more guidance, add checked extensional facts
inside its isolated proof: the tail equality above, equality of both heads for
nonempty s, empty-sequence/concatenation identities in the base case, and
concatenation associativity for the induction step. Each extensional fact can
be proved from equal lengths and pointwise indices. Do not add a trusted
associativity or tail axiom.

A generic concat wrapper is optional; it is not necessary for the present
three-character boundary string. If needed, its precise contract is
`left.concat(right).to_bytes() == left.to_bytes().concat(right.to_bytes())`,
with induction on `left.len()`. Base: left is empty. Step: recurse on
`left.tail()`, prove `head(left++right)=head(left)` and
`tail(left++right)=left.tail()++right` by ext_eq, then use one-step flat_map
recurrence and concatenation associativity. Keep this separate from individual
scalar arithmetic.

### Compose the existing literal goals

For é/€, prove the literal's `Seq.create 1` character model extensionally equal
to `Seq::singleton(c)`, then combine utf8_singleton with the corresponding
scalar encoding helper. This leaves no recursive suffix flattening in the
literal caller's final obligation.

For the boundary string, first prove its model extensionally equal to
`Seq::singleton('\u{D7FF}').push_back('\u{E000}').push_back('\u{10FFFF}')`.
Use utf8_singleton once and utf8_push_back twice, followed by the three proven
scalar byte equations. Finally prove the concatenation of the resulting
3-, 3- and 4-byte sequences extensionally equals the ten-byte expected sequence:
length 10; indices 0..2 in the first segment, 3..5 in the second, 6..9 in the
third. This is finite linear index reasoning, not Unicode decoding induction.

Ordinary ghost scalar helpers can be invoked directly with ghost!. For logical
wrappers, use a snapshot/logic invocation as supported by the harness and inspect
that the generated caller actually imports their proved postconditions. Their
source existence alone does not make them available to a caller. Keep all helper
VCs in the proof target/coverage index, with no trusted markers. Root schedules
all solver runs; this consultation changes only this document.

Correction to the preceding discriminant implementation note: the message worker
has now inspected the pinned MIR and reports that `discriminant_value` has
already become `Rvalue::Discriminant` before Creusot translation. The concrete
lowering therefore belongs in `translation/function/statement.rs`, whose current
handler discards that rvalue, while preserving `discriminator_for_switch`'s
existing enum-switch optimization. The generic semantic discriminant rules and
actual-derived-body requirements stated above are unchanged.

## Actual parse_code: export next's concrete state transition

Read-only inspection of `code-harness/string/verif/.../parse_code.coma`
finds a missing premise, not merely a difficult quantified sequence goal.
`next_Bytes` exports only `None => completed(self)` and
`Some(byte) => produces(entry, singleton(byte), final)`. The latter expands to
equal input, equal end, and suffix concatenation; it deliberately omits mark.
The actual parser requires mark preservation. Its Some branches cannot establish
that property from the imported interface, even if suffix arithmetic succeeds.
The None branch does have full resolution through `completed`, but that does
not repair any preceding Some call. The model's exact decision ladder is already
visible in this generated caller; no opaque-model unfolding repair is needed.

Keep the existing Iterator contract and the generic produces relation unchanged.
Add and prove the following concrete postconditions on the actual `Bytes::next`
body, using its already proved peek and bump contracts:

```rust
#[ensures((^self)@.input == self@.input
    && (^self)@.mark == self@.mark
    && (^self)@.end == self@.end)]
#[ensures(match result {
    Some(byte) => self@.cursor < self@.end
        && byte@ == self@.input[self@.cursor]@
        && (^self)@.cursor == self@.cursor + 1,
    None => self@.cursor == self@.end
        && (^self)@.cursor == self@.cursor,
})]
```

These are concrete method guarantees, not stronger caller preconditions or a
trusted lemma. Existing type invariants bound the indexed byte. Preserve and
recheck the actual next body and Iterator refinement goals after this contract
change. Then regenerate the parser caller with the matching isolated compiler.
The explicit byte/cursor facts also remove its need to infer an index and a
one-byte advance through quantified subsequence/concatenation semantics.

Only if the repaired interface remains expensive, add erased checked cutpoints
after the existing successful `expect!` expressions. Snapshot the entry model;
after the first successful expression record cursor = entry.cursor + 1,
hundreds = entry.input[entry.cursor], its ASCII bounds, and the unchanged frame.
After the second and third, record the corresponding offsets and byte facts.
The arithmetic then has three values in 0..9 and a result in 0..999. These
cutpoints cannot by themselves fix early returns, which must continue to use
the exact next transition and the model's already visible decision ladder.
Do not refactor the runtime macro, weaken the exact result/cursor postcondition,
or add an axiom for this parser. Root owns the running proof and solver queue;
this consultation starts no solver and changes no implementation.

## MaybeUninit slices: initialization and loan restoration are separate gates

The memory-initialization proposal correctly identifies the missing wide-pointer
permission conversion. Inspection adds a necessary condition: for a slice that
started as `[T]`, preserve initialization of the **entire original backing
slice**, including elements outside the subsequently returned prefix. Its
owner can later drop the original array or Vec. In contrast, a slice that
started as `[MaybeUninit<T>]` needs only its returned prefix initialized.
The current parser writes only `MaybeUninit::new(Header { ... })` and does not
deinitialize other slots, so it can prove the stronger conditional frame.

### First gate: standard assume_init_mut

The pinned core method `[MaybeUninit<T>]::assume_init_mut` is stable since
1.93.0 and its implementation is exactly the wide-pointer cast used by the
crate helper. Add its generic extern specification to the isolated, matching
`creusot-std/src/std/mem.rs`, adjacent to scalar MaybeUninit specifications:

```text
requires forall i in [0, self.len): self[i].view != None
ensures result.len == entry(self).len
ensures forall i in that range: entry(self)[i].view == Some(result[i])
ensures forall i in that range: final(self)[i].view == Some(final(result)[i])
```

These are equations on T values, not copies or runtime moves; no T: Copy bound
is needed. The precondition requires a valid T, not merely nonzero bytes.
The final equation is sound because the returned borrow has type `[T]` and
cannot safely leave invalid T values behind. Audit the standard method's source
and record same-allocation, metadata, alignment, exclusivity and lifetime facts
as the standard memory primitive's semantic justification. A sequence equation
alone does not prove those pointer facts. If later callers need a logical
pointer identity, expose it through a permission-aware adapter with the same
allocation/provenance and length; do not equate bare integer addresses with
permission ownership.

The actual `assume_init_slice` can then call `s.assume_init_mut()` and have its
own checked body/contract. This changes the helper's spelling, preserves its
signature and layout, and requires no parser trust. Confirm the project's
supported minimum Rust version before adopting the newly stable method in
normal builds; the pinned verification compiler supports it. An older-compiler
compatibility adapter must preserve the same audited cast and semantic contract,
not silently select a different unverified branch.

### Second gate: do not trust an unrestricted deinitializing loan

`Perm::from_mut` borrows a typed permission whose final value restores the
original reference; `Perm::as_mut` requires that same typed permission.
`split_at_mut` preserves element type and rejoins final values. There is no
existing cross-type loan conversion in those APIs. Layout equality and
`Perm::cast`-style pointer spelling do not supply one.

In particular, a trusted operation returning an unrestricted
`&mut [MaybeUninit<T>]` from `&mut [T]` cannot merely promise initial Some
values and final restoration in ensures. Its caller could write uninit, end
the loan, and use/drop the original T. Placing final Some in a trusted ensures
would assume precisely the obligation the caller should prove. The same problem
exists for the crate's outer `&mut &mut` binding cast. A generic permission
conversion must not duplicate a live T permission and an independently mutable
MaybeUninit permission to the same storage.

The smallest scoped alternative is a generic support operation of this shape:

```text
with_uninit_view<T, R, F>(s: &mut [T], f: F) -> R
where F: for<'loan> FnOnce(&'loan mut [MaybeUninit<T>]) -> R
```

It retains the full original capacity for the whole call and returns no loan.
Its requirements, checked at the caller, are:

1. For every borrowed slice b whose initial elements are Some(entry(s)[i]),
   the callback precondition holds.
2. For every such b and callback result r, the callback's postcondition implies
   that every element of final(b), over the full original length, is Some.
3. The callback cannot unwind while the original T loan is suspended. Require
   checked panic freedom of the concrete callback/callees, or supply an explicit
   restoration guard valid on unwind. A normal-return postcondition alone is
   insufficient. Do not accept an arbitrary external callback on that basis.

The bridge post relates its result to the callback's postcondition and maps
final(b)[i] = Some(final(s)[i]). Existing `FnOnceExt::precondition` and
`postcondition_once` in `std/ops.rs` provide the logical vocabulary. First make
a translation-only HRTB/closure probe to establish concrete supported syntax.
Its body is the generic scoped reborrow/cast followed by exactly one callback;
the only new trusted support boundary is that representation-and-loan operation,
with its caller obligations. No parser result, prefix predicate, callback
contract, or callback body belongs in the TCB. This new support primitive needs
its own audit; it is not an existing std method and must not be labelled as one.

Inside the actual initialized parser wrapper, the callback can reborrow the
full slice into a local mutable slice binding, run the existing uninitialized
parser, and return `(result, final_prefix_length)`. Its proof retains the full
slice's final contents after that local reborrow ends. Only after the scoped
bridge returns should the original initialized binding be shortened to the
returned count. Account for the original ShrinkOnDrop behavior on every result;
prove no panic through this path or preserve the binding-shortening guard on
unwind as well. This is an actual runtime refactor and needs root approval of
the concrete diff, normal-build tests, and an erasure/behavior correspondence
check. Do not pretend it verifies the old unrestricted converter body.

If a scoped cross-type TCB is undesirable, a larger, safe runtime alternative
is one parser over two storage adapters: initialized `[T]` and uninitialized
`[MaybeUninit<T>]`, exposing capacity/write/shrink. This avoids the binding cast.
For arbitrary non-Copy T, the initialized write can use `mem::replace` followed
by `mem::forget(old)` to match the original MaybeUninit overwrite's no-drop
behavior; ordinary assignment would drop old T and is not generally equivalent.
For actual Header, old values have no Drop, but that does not justify claiming
a generic arbitrary-T equivalence. This alternative is larger than the scoped
bridge and should be a deliberate reviewed choice.

### Required staged evidence

Start with standard assume_init_mut and generic one-slot/slice probes: empty,
mixed initialized/uninitialized input, initialized non-Copy T, and zero-sized T.
An uninitialized element within the converted prefix must fail its precondition;
an uninitialized tail outside that prefix must remain permitted for the uninit
API. Next probe the scoped bridge with a callback that replaces values by Some
and one that writes None: only the first may establish restoration. Also reject
a callback that initializes the returned prefix but deinitializes the original
tail, a callback with no proved restoration post, an escaping borrowed slice,
and an unproved panic path. These are semantic proof tests, not trusted facts.

Then prove the actual parser's storage invariant: n is in bounds, [0,n) is
initialized, unvisited slots retain their entry states, and writes occur only
at the next slot with Some(header). For initialized input this yields all-Some
over the entire original region; for uninitialized input the tail stays
unconstrained. ShrinkOnDrop changes only the exposed length and never proves
initialization by itself. Only after these checked boundaries pass should the
request/response header paths consume the conversion contract. No solver or
implementation was changed by this consultation.

## Preferred header storage implementation: safe typed adapter

Following root's review, prefer a private typed adapter over the new scoped
cross-type TCB described above. It is a somewhat larger runtime refactor but a
smaller proof/support change: two ordinary safe branches per storage operation,
no higher-order restoration contract, and no new representation/loan primitive.
Keep only the standard uninitialized-to-initialized slice contract as TCB.

Use an adapter borrowing the caller's slice **binding**, so the current
ShrinkOnDrop mechanism can remain responsible for early-return shortening:

```rust
enum HeaderStorage<'s, 'h, 'b> {
    Initialized(&'s mut &'h mut [Header<'b>]),
    Uninitialized(&'s mut &'h mut [MaybeUninit<Header<'b>>]),
}
```

The three lifetimes distinguish the short binding borrow, the storage borrow,
and the input bytes referenced by Header. Keep them independent except for
outlives bounds actually required by Rust; do not tie the storage borrow to the
whole Request/Response mutable borrow. Private helper shape:

```text
len(&self) -> usize
write(&mut self, index: usize, header: Header<'b>) -> ()
shrink(&mut self, n: usize) -> ()
```

`write` requires index < len, preserves the variant and capacity, sets exactly
that logical slot to Some(header), and frames all other slots. Its initialized
branch performs ordinary Header assignment; its uninitialized branch assigns
MaybeUninit::new(header). Actual Header consists of borrowed name/value slices
and has no Drop implementation, so dropping its replaced value has no runtime
effect. This argument is Header-specific. Delete the unused private generic
deinit converter; do not claim a verified arbitrary-T initialized-to-uninitialized
conversion. The standard assume_init_slice helper can remain generic over T.

`shrink` requires n <= len and uses, in each typed branch,
`let full = mem::take(binding); *binding = full.split_at_mut(n).0;`.
It preserves the prefix values, variant, and allocation while changing exposed
length to n. The initialized branch cannot deinitialize the underlying tail;
the uninitialized branch makes no tail-initialization claim. Only a standard
safe slice operation is needed. There is no pointer cast or unchecked index.

Lift the existing guard to hold HeaderStorage plus num_headers, with invariant
num_headers <= storage.len and all logical slots below num_headers initialized.
Its Drop calls shrink(num_headers). Verify the actual destructor and its
invariant on each early-return/drop edge; source-level existence of a guard is
not proof that the verification pipeline checks its cleanup. Preserve the
existing guard's effect on all checked normal return paths and any supported
unwind semantics. Its operations have explicit in-bounds conditions, so the
guard itself must not panic.

The shared header parser accepts HeaderStorage by value and creates that guard.
The initialized/uninitialized entry helpers only construct the corresponding
variant and call this single parser body. Replace iter_mut and iter.next with
the guard's count and capacity: n is both the next storage index and the number
of fully written headers. At the old iter.next location, check n == capacity
and break with the existing TooManyHeaders result; otherwise perform the
existing trim, write(n, header), then increment n. **Do not move the capacity
check earlier.** The old parser consumes the complete name and value before
discovering capacity exhaustion; checking at loop entry would change consumed
bytes, partial/error precedence, and zero-capacity blank-line behavior.

For the proof model, map initialized slots to Some(Header) and uninitialized
slots through their existing MaybeUninit View. The parser needs only capacity,
exact indexed update, unchanged other slots, and initialized prefix. Rust's
typed initialized branch supplies validity of the full original capacity.
This also removes the IterMut consumed-reference/remaining-sequence algebra
from the storage proof. Keep syntax/byte-cursor/result refinement independent
from these three small storage methods.

### Complete the Request/Response wiring too

Removing deinit_slice_mut alone does not remove every initialized-to-uninitialized
cast. Both Request::parse_with_config and Response::parse_with_config currently
perform their own raw cast and restore the original full Header slice after a
non-complete result. Refactor their existing shared parsing work into a private
method accepting typed storage and updating method/path/version/code/reason at
the same points as today. It must not assign self.headers internally. The
public-facing wrappers decide which typed storage to supply and when to publish
the resulting header slice.

For an initialized wrapper, move out self.headers with the existing mem::take,
retain that full initialized slice, and pass a shorter-lived reborrow through
a local slice binding and Initialized storage. Once that reborrow and guard
end, read the local prefix length. On Complete, publish the corresponding
prefix of the retained full slice; on Partial/Error, restore the full original
slice, preserving any prefix writes already performed. This recovers capacity
without raw pointers because shortening affects only the local reborrow.
On a panic path the original code leaves self.headers empty after mem::take;
do not accidentally introduce a different restoration guarantee without review.

For an uninitialized wrapper, borrow its existing input binding through the
Uninitialized variant. On Complete only, convert its shortened initialized
prefix with the standard assume_init_mut contract and assign self.headers.
On Partial/Error, leave the prior self.headers untouched, as the current code
does. Keep all other partial field updates in their original order. This is
one shared parser/state algorithm, not duplicated initialized/uninitialized
implementations.

Public type layouts, method signatures and lifetimes remain unchanged; the new
enum is private and stack-local. A runtime enum branch on writes may have a
performance cost, so avoid claiming zero overhead without measurement. A later
generic storage trait could permit static dispatch, but adds trait-refinement
proof work and is unnecessary for the first safe implementation.

Validate the adapter methods first, then the storage loop invariant and guard,
then both wrapper families against the exact model. Regression cases must
include capacity zero with an immediate blank line, capacity exhausted after
consuming a valid header, partial name/value, invalid headers skipped by config,
and non-complete initialized-wrapper restoration of full capacity. Include a
mixed-uninitialized tail test proving only the completed prefix is converted.
The use of the Rust 1.93 standard method is a documented verification-fork
toolchain requirement, not a claim that original upstream MSRV support remains
unchanged. Root approves implementation scope; this entry is design only.

### Derived Error equality: opaque model diagnosis (2026-10-05)

Read-only consultation inspected the isolated `derived-traits` generated COMA;
no solver was run for this diagnosis. `impl_PartialEq_for_Error/eq.coma`
contains exact seven-constructor discriminant matches, but declares
`deep_model_Error` without its body. Thus its inherited postcondition is not
provable from that context: two different Error constructors can have different
discriminants while an uninterpreted model function maps them to the same model
constructor. The positive result log records `vc_eq_Error` Timeout at about
26 seconds. The caller `error_equal_different_variants.coma` does contain the
full model definition; caller success cannot establish the derived body.

The likely cause is the new local derived-contract inheritance interacting
with `ctx.rs::param_env`: it adds concrete external-spec predicates such as
`Error: DeepModel`. Trait resolution can select this parameter assumption,
returning `TraitResolved::UnknownFound`, so `clone_map/elaborator.rs` emits an
opaque declaration despite the actual derived model being `logic(open)`.
This causal path remains to be confirmed by the implementation change.

Recommended narrow fix: for a local inherited derived contract, instantiate
all inherited predicates and prove them in `tcx.param_env(def_id)`, using a
fresh `infer_ctxt().ignoring_regions().build(TypingMode::non_body_analysis())`
and the existing `translation::traits::evaluate_additional_predicates` query.
That query treats ambiguous obligations as errors. Only `Ok(())` permits
using the native environment unchanged: the added assumptions have then all
been proved redundant. Use the native `tcx` environment, not recursive
`ctx.param_env`, and never prove a predicate in an environment already assuming
that predicate. Failure or ambiguity must fail closed for this narrow local
inheritance extension; do not drop generic bounds or assume concrete ones.
Generic inheritance with genuinely additional bounds needs a separate audit
of propagation to callers, whose current terminator check consults direct
external specs. This recommendation adds no trusted equality/model axiom.

Before spending solver time, confirm the derived Error and signed-enum body
COMA now contains each actual model match definition. Then prove the exact
derived body and refinement, retain the false-equality and false-discriminant
negative controls, and refresh evidence hashes. This entry records diagnosis
and a proposed compiler fix, not a completed fix or proof. Exact formatting
text and full Unicode support are separate open obligations.
