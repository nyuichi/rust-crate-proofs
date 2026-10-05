# itoa 1.0.18 runtime verification status

## Scope map

The production `runtime` module contains the optimized implementation: a
`MaybeUninit` output buffer, the decimal-pair lookup table, reciprocal division
by 100, four-digit chunking, the specialized `u128` path, and output string
construction. Under `cfg(creusot)`, the shared `Unsigned::fmt` bodies for
`u8`, `u16`, `u32`, `u64`, and `u128` are translated and proved. The public
formatting facade runs the actual `runtime::Buffer::format` body, and the
integrated default and all-features proofs pass on x86_64. The default run
reported 254 proof libraries / 2,083 VCs; all-features reported 269 / 2,145.
Boundary A uses a safe prefix-to-array `.try_into().unwrap()`, proved for all
12 concrete writers. Boundary B isolates the initialized
`MaybeUninit<u8>`-to-`u8` slice view in the sole new local trusted helper,
`assume_init_slice`; its caller proves initialization, while the trust covers
only the borrowed slice view, length, and bytes; physical permissions remain
unproved. These are acknowledged source changes from the preferred unchanged upstream path; the
exact contracts and removal conditions are in
[RUNTIME_BOUNDARY_BRIDGE.md](RUNTIME_BOUNDARY_BRIDGE.md). Phase 7 also proves
the actual signed `i8`, `i16`, `i32`, `i64`, and `i128` buffer writers,
including the `i128::MIN` path. The 64-bit `usize` and `isize` buffer-writer
adapters are proved on the x86_64 target; 16- and 32-bit pointer-width paths
remain outside this proof. The `u64` 16-digit chunk encoder called by the
native `u128` formatter is translated and proved separately. The recursive
decimal model in `verification.rs` remains the formatter's specification.

Phase 8 proves ASCII bounds directly from that canonical model and checks the
result on actual i8 and `i128::MIN` writer witnesses. The writer memory audit,
source-ground `MaybeUninit` contracts, and remaining runtime memory boundaries
are recorded in [RUNTIME_MEMORY_LEDGER.md](RUNTIME_MEMORY_LEDGER.md).

The first shared implementation boundary is `divmod100`: normal runtime callers
use this body, and Creusot translates and proves the same executable body as an
independent leaf. Phase 3 connects the actual `u8`, `u16`, `u32`, and `u64`
formatter bodies to the decimal model; Phase 7 proves the signed `i128`
buffer writers and the x86_64 pointer-width byte-writer adapters. The actual
public formatting and string-conversion bodies are also proved in the integrated
run. The only new local runtime trust is the narrow `assume_init_slice` view
conversion; no formatting arithmetic or table-correctness fact is trusted.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated runtime proof |
| --- | --- | --- | --- | --- |
| Existing recursive decimal model and retained model `Buffer` | yes | yes; string leaf has a proved ASCII precondition and ordinary body | none locally; standard/core models remain separately listed | yes; default and all-features |
| Shared optimized `divmod100` (Phase 1) | yes | yes | none added | yes; leaf and formatter callers |
| `DECIMAL_PAIRS` representation and indexed lookup lemma | yes | yes (2 VCs) | none | data bridge via CTFE; formatter use proved for `u8`, `u16`, `u32`, `u64`, and `u128` |
| Actual optimized `Unsigned::fmt` body for `u16` | yes | yes (113 goals in current integrated runs) | no formatter boundary | yes; default and all-features |
| Actual optimized `Unsigned::fmt` body for `u32` | yes | yes (116 goals in current integrated runs) | no formatter boundary | yes; default and all-features |
| Actual optimized `Unsigned::fmt` body for `u64` | yes | yes (119 goals in current integrated runs) | no formatter boundary | yes; default and all-features |
| Actual optimized `Unsigned::fmt` body for `u8` | yes | yes (104 goals in current integrated runs) | no formatter boundary | yes; default and all-features |
| Actual `enc_16lsd` 16-digit chunk encoder body (Phase 5) | yes | yes (`enc_16lsd`: 55 goals; quad writer: 87; digit bridge: 50) | none | yes; component called by the Phase 6 `u128` body |
| `u128_ext::mulhi_core` limb operations and exact high-half result (joint-helper closure) | yes | yes (body 68/68; core module 74/74) | none; exact quotient postcondition is proved from the limb result with operation checks retained | yes; default and all-features |
| `u128_ext::mulhi` wrapper and exact result contract | yes | yes (wrapper module 2/2) | none; delegates to the proved core | yes; default and all-features |
| `div_rem_1e16` reciprocal divider body (Phase 4) | yes | yes (31 goals) | consumes the proved exact high-half result contract | yes; default and all-features |
| Actual optimized `Unsigned::fmt` body for `u128` (Phase 6; strengthened in Phase 7) | yes | yes (147 goals; refinement 1) | consumes the proved exact `mulhi` high-half result contract | yes; default and all-features |
| Actual signed `i128` buffer writer and `i128::MIN` output witness (Phase 7) | yes | yes (suffix 24; signed writer 4; MIN witness 4) | no local formatter trust; exact core models for `unsigned_abs` and array borrowing; consumes the proved `mulhi` contract | yes; default and all-features |
| Actual signed `i8` buffer writer (Phase 7) | yes | yes (suffix 23; signed body 4 integrated / 16 targeted; all-input harness 3) | no local formatter trust; exact core models for `unsigned_abs` and array borrowing | yes; default and all-features |
| Actual signed `i16` buffer writer (Phase 7) | yes | yes (suffix 27; signed body 4; all-input harness 3) | no local formatter trust; exact core models for `unsigned_abs` and array borrowing | yes; default and all-features |
| Actual signed `i32` buffer writer (Phase 7) | yes | yes (suffix 27; signed body 4; all-input harness 3) | no local formatter trust; exact core models for `unsigned_abs` and array borrowing | yes; default and all-features |
| Actual signed `i64` buffer writer (Phase 7) | yes | yes (suffix 22; signed body 4; all-input harness 3) | no local formatter trust; exact core models for `unsigned_abs` and array borrowing | yes; default and all-features |
| 64-bit `usize`/`isize` buffer-writer adapters (Phase 7) | yes | yes (each cast 1; each formatter adapter 3) | no local trust; delegates to proved `u64`/signed `i64` bodies | yes on x86_64; default and all-features |
| Canonical decimal ASCII lemmas and actual writer witnesses (Phase 8) | yes | yes (three model lemmas: 1 goal each; i8 witness 3; `i128::MIN` witness 4) | none added | yes; default and all-features |
| Concrete `Sealed::write` bodies and trait refinements (all 12 x86_64 types) | yes | yes (focused: u8 20 goals; other 11 8 each; 12 refinements 1 each) | Boundary B helper only | yes; default and all-features |
| Actual public `Buffer::format` and runtime string conversion | yes | yes (`Buffer::format`: 3/5 VCs; slice conversion: 4/6, default/all-features) | one narrow local trust: `assume_init_slice` for initialized `MaybeUninit<u8>` slice view only | yes; default and all-features |
| 16/32-bit pointer-width fallbacks | not covered on x86_64 | no claim | none added | pending |

The integrated default and all-features `verify-all.bash` runs both exited 0.
Default reported 254 proof libraries / 2,083 VCs; all-features reported 269 /
2,145. The actual `Buffer::format` body discharged 3 VCs by default and 5 with
all features; initialized-suffix string conversion discharged 4 and 6. The
focused proofs discharged all 12 actual `Sealed::write` bodies and their 12
refinements, including the safe prefix conversions: the `u8` body used 20 VCs,
each other body used 8, and each refinement used 1 (120 VCs total). The focused
ASCII proof discharged `ascii_byte_map_to_utf8` in 30 VCs and
`ascii_bytes_are_utf8` in 3 VCs, with no trusted helper. The full run is
[`integrated/fullsuite.log.gz`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/integrated/fullsuite.log.gz);
the result and focused evidence are summarized in the
[`runtime boundary report`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/REPORT.md).

## Accepted runtime boundary changes

The source change was explicitly accepted to let Creusot analyze the actual
public method. It changes only the two earlier translation boundaries:

- **A:** the concrete sealed writer receives the existing 40-slot physical
  buffer, takes the prefix of its `MAX_STR_LEN`, and converts it safely to the
  fixed associated array using `.try_into().unwrap()`. The proof shows the
  conversion length matches for every sealed type. No unsafe or trusted
  contract is added here.
- **B:** `assume_init_slice` contains the raw slice reinterpretation. Its
  precondition is that every input slot is initialized; its postconditions
  preserve length and each logical byte. It assumes no ASCII or decimal fact.
  Creusot does not currently model this initialized-slice view conversion, so
  this operation alone carries local `#[trusted]`. The caller proves the
  logical initialization precondition; the physical permission transfer remains
  unproved.

The verification model's old trusted `decimal_slice_to_str` leaf was removed.
Its replacement requires a proved ASCII suffix and retains the same byte
postcondition. Its ordinary body and caller precondition pass in the current
integrated run; there is no additional local trusted string-conversion leaf.
The exact rationale and removal conditions are recorded in
[RUNTIME_BOUNDARY_BRIDGE.md](RUNTIME_BOUNDARY_BRIDGE.md).

## Proved `mulhi` closure

The joint-helper proof now closes the exact result contract from the actual
`mulhi_core` limb implementation: `result@ == x@ * y@ / 128.pow2()`. The
focused `mulhi_core` body discharged 68/68 goals, its core module discharged
74/74, and all 74 generated Why3 theory leaves were valid with no unresolved
goals. The `mulhi` wrapper has no `#[trusted]` annotation; its body proof
discharged 2/2 goals. Its exact contract is consumed by
the existing `u128` formatter and signed `i128` writer proofs, so this removes
the previous accepted arithmetic assumption without changing the executable
runtime body. The evidence and replay details are in the
[`Power_sum composition report`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/pow2-composition/REPORT.md).

The pre-bridge source candidate passed 11 integration tests plus 2 doctests
with `cargo test --offline --locked`, and 11 integration tests with
`cargo test --offline --locked --release --all-features --tests`. Its
post-closure `run-verify-all.sh` runs reported 224 proof libraries / 1,917
VCs in both configurations, with no failed goals. These are historical
pre-bridge results and do not include the current public runtime method or the
new reused-buffer regression test. Current integrated proof and native test
results are recorded in [Current end-to-end status](#current-end-to-end-status).
The historical proof log is
[`fullsuite.log.gz`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/pow2-composition/fullsuite.log.gz).

Phase 1 proves only the leaf contract: for `value < 10_000`, the returned
pair equals Euclidean quotient/remainder by 100, the remainder is below 100,
and the quotient-remainder reconstruction equals the input. The executable
statements are kept unchanged and single-sourced for normal Rust and Creusot.
No trusted declaration is introduced for this phase. The pre-existing trusted
string-construction leaf belongs to the separate recursive verification model;
it is not used to justify the `divmod100` body.

Phase 2 moves the production table declaration into the shared `decimal_pairs`
module while preserving the original runtime `static` and its byte-string
initializer. The formatter body continues to use the same indexed reads.

Creusot 0.11.0-dev cannot translate the safe immutable static definition; the
driver reports `Static { safety: Safe, mutability: Not, nested: false }` as an
unsupported definition kind. Changing the table to a `const` initialized by
`DecimalPairs(*b"...")` also fails translation with
`Unsupported constant value: Scalar(alloc94) of type &'?2 [u8; 200_usize]`.
These are Creusot translation limits for statics and the byte-string array
constant.

The module therefore has an explicit cfg representation difference: normal Rust
uses the unchanged byte-string literal in the runtime static; Creusot uses a
`const` built from an explicit 200-byte scalar array. Both scalar initializers
come from one macro. In the normal cfg, a const-evaluated loop checks all 200
scalar bytes against the 200 literal bytes, and a compile-time assertion fails
on any mismatch. Thus normal Rust compilation ties the Creusot const to the exact
runtime table value without a trusted axiom.

`decimal_pair_correct(n)` requires `n < 100`. Its postcondition proves the
actual proof-table cells at indices `2*n` and `2*n+1` equal
`48 + n/10` and `48 + n%10`, proves the latter index is below 200, and ties its
returned bytes to those cells. The helper reads the array cells directly. The
Creusot run proves both the constant setter and this indexed lookup contract (2
VCs).

## Phase 3: actual unsigned formatter bodies

The proof translates the production `impl_Unsigned!(u16)` body, including the
four-digit loop, two-digit tail, final masked digit, lookup-table reads, and
`MaybeUninit` writes. Its strong contract preserves the unwritten prefix,
initializes every output slot, and gives each output byte as the corresponding
element of `decimal_values(self@)`. At the initial `u16`-only checkpoint,
`./verify-all.bash` proved the body in 201 goals and its generated
trait-refinement obligation in one goal in each configuration. After extracting
the shared two- and four-digit writers for `u32`, the same `u16` body is
rechecked in 104 integrated goals with one refinement goal in each configuration.
A separate no-cache targeted replay discharges 105 body goals and one
refinement goal. The concrete method has no added precondition: a typed capacity lemma proves at body entry
that every `u16` fits its five-byte buffer. The final-digit writer proves
independently in 11 VCs and in 2 integrated goals. A call-site check consumes
the formatter contract after deriving the same capacity fact.

The one-slot writer is an inline extraction of the existing operation
`buf[offset].write(b'0' + last)`. The production mask expression
`remain as u8 & 15` and the chunking arithmetic remain in the actual formatter.
The extracted helper has proved bounds, exact byte, initialization, frame, and
suffix-concatenation contracts; it is not trusted. In normal builds its
`no_panic` instrumentation remains enabled. Under `cfg(creusot)`, the
`no_panic` attribute is omitted because that proc-macro instrumentation cannot
be translated; the Rust body and formatter contracts are unchanged.

The `u32` proof also extracts the formatter's existing two-digit and
four-digit table writes into inline helpers. Their executable bodies retain the
same table lookups and `MaybeUninit::write` calls; only proof guidance moved
behind these helper boundaries. Their contracts establish the selected digit
bytes, initialization, unchanged slots outside the write range, and composition
with an already initialized output suffix. The arithmetic loop, conversions,
and lookup table are unchanged.

The `u32` body and its trait refinement were first proved in a no-cache targeted
run: `fmt` discharged 104 goals and `fmt__refines` one goal. The corresponding
pair/quad write helpers discharged 35 and 67 goals, respectively, and the
shared `u16` formatter regression discharged 105 body goals plus one refinement
goal. The fresh integrated default and all-features runs each reported 102 goals
for `u32::fmt` and one refinement goal, and 104 goals for `u16::fmt` and one
refinement goal.

The `u64` formatter reuses the same shared macro body and dynamic-index
two- and four-digit writers. Its targeted no-cache proof discharged 105 `fmt`
goals and one trait-refinement goal. Fresh integrated default and all-features
runs each discharged 104 goals for `u64::fmt` and one trait-refinement goal.

The actual `u8` formatter body also uses the shared macro. Its
`size_of::<Self>() > 1` loop guard is false for `u8`, so the four-digit chunk
loop is unreachable; the original two-digit tail and final masked digit stay
on their regular path. A typed capacity lemma proves that every `u8` fits its
three-byte buffer; the formatter contract has no added precondition. The
targeted no-cache proof discharged 98 body goals and one refinement goal.

The staged Creusot build enables the runtime module and compiles the actual
`u8`, `u16`, `u32`, and `u64` unsigned formatter bodies. It omits the raw public
`Buffer::format` and string conversion adapters, signed wrappers, and other
unsigned bodies until their proof phases. It also translates and proves the
same `enc_16lsd` body used by the native `u128` implementation; the `u128`
formatter caller and reciprocal divider remain excluded. The ordinary Rust cfg
continues to compile the production runtime implementations.

## Phase 4: `u128` reciprocal divider boundary

The Phase 4 candidate reports and failed attempts below are historical records
from before the joint-helper closure. Their open-goal and accepted-trust
conclusions describe those checkpoints only; see [Proved `mulhi` closure](#proved-mulhi-closure)
for the current result.

At the original Phase 4 checkpoint, the remaining boundary was the actual
128-bit high-half multiply used by the reciprocal divider. The runtime quotient is computed with the original
Granlund–Montgomery operation: multiply by
`76_624_777_043_294_442_917_917_351_357_515_459_181`, take the high half, then
shift it right by 51; the divider subtracts `quotient * 10^16` to obtain the
remainder. The staged divider contract states the exact quotient, remainder,
reconstruction, and remainder bound. No replacement arithmetic is used.

At that historical checkpoint, a candidate based on `51090b4` had completed
focused proofs for the divider components: `shift_by_51` passed 2/2 goals, `magic_quotient` passed 10/10,
`math::magic_shift_floor` passed 1/1, and `div_rem_1e16` passed 31/31. The
integrated source keeps the original `mulhi` limb algorithm in
`mulhi_core`, with its operation-range and overflow proof obligations checked
separately. The public-in-crate `mulhi` entrypoint is a thin inline wrapper
whose exact trusted result contract is:

```text
result == x * y / 2^128
```

The user accepted this exact temporary proof boundary on 2026-10-02. At that
checkpoint, the wrapper's call to `mulhi_core` was not itself checked, so the
integrated proof did not connect the core's returned limb value to this
quotient equation. The core's arithmetic operations, casts, shifts, bounds,
and overflow checks remained checked; the high-half correctness of their
composed result was assumed. The joint-helper closure documented above has
since proved this equation from the core and removed that temporary boundary.

### Second isolated `mulhi` proof attempt

A follow-up candidate preserves the limb algorithm and attempts to prove the
core's exact result equation before removing the wrapper trust. Its independent
`mulhi_product_congruence` and `mulhi_product_expansion` helper bodies each
passed their single focused VC. An early caller variant reached 62/64 goals;
both failures were applications of the congruence helper's opaque
postconditions. Later variants discharged the first product substitution, but
the final `int-proof-assert` candidate reached 67/68 and left
`vc_mulhi_core.54`, the second product substitution, open:

```text
(x_hi * 2^64 + x_lo) * y
  == (x_hi * 2^64 + x_lo) * (y_hi * 2^64 + y_lo)
```

The preceding fact `y == y_hi * 2^64 + y_lo` is already available at that
assertion. Because this caller VC remained open, that candidate's exact-result
postcondition was not a proved consequence of `mulhi_core`; the then-integrated
wrapper continued to use the exact accepted trusted equation above. The
attempt snapshots, generated Why3 session, and focused result are preserved in
[`tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/REPORT.md`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/REPORT.md),
with the final candidate under
[`int-proof-assert/`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/int-proof-assert/).
At that historical attempt, the removal condition was to prove the wrapper
equation from the core while keeping its operation checks discharged.

### x-side product-split follow-up

The source now includes the ordinary `#[cfg(creusot)]` lemma
`mulhi_x_split_product`, symmetric to the y-side lemma. Its focused body proof
passes 1/1 VC. A small caller invokes both lemmas under their exact split
preconditions and still leaves the explicit two-sided product substitution
open: the run reports 3/4 goals, with the `assertion` task
`vc_mulhi_product_split_from_limbs.2.0` unknown for Z3 and timed out for CVC5.
The caller's result contract proves from its preconditions, but the explicit
assertion in the body does not discharge from the generated caller context. An
attempt to add both calls and the exact high-half result contract directly to
`mulhi_core` reached the 120-second run limit without a proof result. That
candidate kept only the independently proved x-side lemma; the trusted
`mulhi` result equation was unchanged at that point. The failed small caller and its
COMA, Why3 session, and proof JSON are preserved in
[`step1-x-split/`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/step1-x-split/).
With only this helper integrated, the crate's `run-verify-all.sh` passes in
both default and all-features configurations; scoped native test results are
recorded in the same report.

### Actual caller task pruning follow-up

The actual open assertion is `vc_mulhi_product_split_from_limbs.2.0`. Why3's
session export preserves the solver task, but no persistent context-pruning or
replay path was found. Applying `remove_unused` and
`simplify_formula_and_task` to this post-WP task left it byte-for-byte
unchanged, including the generic quantified u128 wrapper axioms. A bounded Z3
run of that transformed task returned `Out of memory (1.81s)` at the required
1024 MiB prover limit. The task, session, proof summary, commands, and results
are preserved in the
[`Step 2 checkpoint`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/step2-task-pruning/REPORT.md).

This experiment did not close the product split in `mulhi_core` or prove the
actual high-half result; the accepted trusted equation remained in place at
that checkpoint. No new trust or runtime change was introduced.

### Exact-goal Why3 pruning experiment

A scratch-only Why3 plugin matched the generated post-WP leaf using its exact
goal name, strict formula hash, and expected wrapper-axiom declarations. It
removed only 22 `Paxiom` declarations from two `creusot/int.coma` wrapper
families; the task log confirms that the goal formula stayed unchanged. The
actual assertion still did not close: Z3 returned `Out of memory (1.80s)` at
one job and 1024 MiB. The plugin was exercised through direct `why3 prove`;
normal `cargo creusot prove` / `why3find` replay was not tested, and no plugin
or runner change was integrated. The trusted `mulhi` equation remained
unchanged at that checkpoint. The plugin source, compressed before/after task exports, logs, and
environment-dependent reproduction notes are preserved in the
[`Step 3 checkpoint`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/step3-why3-pruning/REPORT.md).

### `Power_sum` SMT diagnosis

A follow-up isolated the imported Why3 lemma `bv.Pow2int.Power_sum` as a
quantifier-instantiation hotspot candidate: removing only that premise from
the original exported task, while retaining all 22 wrapper premises and the
exact goal, gave Z3 `unsat` in 0.025 s (18.61 MiB); the baseline still ran out
of memory, and its capped profile attributed 336 of 354 instantiations to the
`Power_sum` quantifier. The full matching trace was not captured, so the cause
remains a strong hypothesis. This hand-edited SMT diagnostic is not a Rust or
`mulhi_core` proof. Because `power_two_sum` is a general empty-body helper, the
next candidate was an additional Z3 driver variant that omits `Power_sum` while
keeping the ordinary driver available. Driver selection/replay was then
unverified; the wrapper trust was unchanged at that checkpoint. See the
[`Power_sum cause-analysis checkpoint`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/pow2-cause-analysis/REPORT.md)
for the reproduction task, logs, and plan.

### Structural split follow-up

A later scratch reproduction confirmed that the actual open Why3 task already
contains `y == y_hi * 2^64 + y_lo` over the same translated integer terms used
in the failed product substitution. The missing-premise explanation is
therefore ruled out. Cutting immediately after the limb decompositions did not
improve the caller proof. A small four-input `cfg(creusot)` helper and a
reduced Why3 theory prove in isolation, but the full certificate and a sparse
six-input lemma still left product substitutions open; these isolated results
did not prove the integrated VC. Direct selection of the generated assertion
for a COMA context cut was also unavailable in the current transformation
stage. The `mulhi` result contract remained trusted at that point. The task inspection and
follow-up experiments are recorded in
[`structural-product-split/REPORT.md`](../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/structural-product-split/REPORT.md).

The integrated Phase 4 `run-verify-all.sh` command completed with exit 0 in
both configurations. Default reported 168 proof libraries / 1,312 VCs;
all-features reported 169 / 1,316. The focused `mulhi_core` target passed
50/50 goals. The native default `cargo test --offline --locked` passed 11
integration tests and 2 doctests; optimized `cargo test --release
--all-features --tests --offline --locked` passed all 11 integration tests.
A separate full release/all-features run failed while linking the two doctests
with undefined `no-panic` symbols; the successful release result is scoped to
the 11 integration tests. The same 0/2 doctest link failure was reproduced on
baseline commit `bf86f1e` with `cargo test --doc --release --all-features
--offline --locked`, so it predates this Phase 4 change. The proof log is
`/tmp/phase4-verify-all-integrated.log`.

At the Phase 4 checkpoint, the `u128` formatter caller was still pending; the
actual shared body is proved in Phase 6 below. Phase 4 does not prove raw
`Buffer::format`, signed adapters, string conversion, or end-to-end `u128`
formatting.

The isolated candidate patches, generated COMA files, proof JSON, and bounded
run reports are preserved under
[`tools/creusot-toolpatch/proofs/phase4-conditional/`](../../tools/creusot-toolpatch/proofs/phase4-conditional/).
Those snapshots document development attempts; the integrated source and
metrics above are authoritative for the current checkout. The divider patch
and the earlier `mulhi` candidate patch were developed separately against
`51090b4`; both change `u128_ext.rs`, so they must not be applied in sequence.
Each artifact's README gives its isolated replay steps. Use the pinned
environment in
[`tools/creusot-toolpatch/records/versions.md`](../../tools/creusot-toolpatch/records/versions.md)
and run the focused no-cache proof targets. The saved `mulhi.coma` and
`proof.json` identify the exact obligations from that attempt. The patched
compiler and Why3 overlay are required for the 128-bit shift/cast obligations;
the checked-in toolpatch scripts document how to rebuild that environment.

The divider patch has a negative control: replacing one quotient assertion
with `quot < 0` made the target fail (26/27). This checks that the conditional
body proof responds to a false claim; it does not remove the `mulhi` dependency.

## Phase 5: actual 16-digit chunk encoder

The original `enc_16lsd` runtime body is moved from `runtime.rs` to
`src/enc_16lsd.rs`, where both normal Rust and Creusot compile the same
executable loop. Its four table reads and four `MaybeUninit::write` operations
are grouped in `write_decimal_quad`; the reads and writes themselves retain the
original expressions. The loop still consumes the input in four-digit chunks
using `% 10_000`, `/ 10_000`, and `divmod100`. No replacement formatter or
trusted local writer is introduced.

The quad writer contract proves that exactly its selected four slots are
initialized with the corresponding fixed-width digits and that every slot
outside that range preserves its prior `Option<u8>` state. The enclosing
encoder contract proves all 16 output slots initialized with
`fixed_width_decimal_values(n, 16)`, while preserving the prefix and suffix
outside its output region. Creusot proves the writer in 87 goals, the runtime
quad-to-digit bridge in 50, and the encoder body in 55. An isolated negative
control changed the first loop's chunk relation to a false assertion; Creusot
rejected it.

This loop uses `(1..4).rev()`. The external `DoubleEndedIteratorSpec` model was
narrowed to `Range<usize>` and corrected to describe the native reverse
iteration: visited length is `original_end - current_end`, and visited item
`i` is `original_end - 1 - i`. The previous generic model had the length
subtraction reversed and omitted the `-1`, which did not describe `next_back`.
The model adds no generic termination assumption or `check(terminates)`
annotation for the iterator. This is a narrow external specification of
`Range<usize>::next_back`; its removal condition is verification of the native
core iterator implementation or an equivalent source-level specification.

The proof-only arithmetic and loop annotations are under `cfg(creusot)`; normal
Rust uses the same executable body. The `no_panic` attribute remains enabled in
normal builds and is omitted under Creusot because its proc-macro expansion is
not supported by the translator. This omission affects only instrumentation,
not the encoder arithmetic or its contracts.

### Standard-library model assumptions used by Phase 3

Narrow external models describe native standard-library operations used by
the Phase 3 formatter. These are assumptions about core APIs, not trusted
local formatter functions:

- `MaybeUninit::write` requires the old value to be `None` or resolved, then
  establishes the exact new `Some(value)` state. This matches the native
  operation, which overwrites without dropping an old value. The formatter's
  element type is `u8`; separate invariants track initialized slots, and the
  proof does not read an uninitialized prefix.
- `TryFrom<i32> for u8` establishes `Ok(value)` only when the input is in
  `0..=255`; it makes no success claim outside that range. The shared
  `TryInto` model only forwards the concrete `TryFrom` precondition and
  postcondition. In particular, the formatter does not assume that `999` or
  `10_000` converts to `u8`.
- `TryFrom<i32> for u16` establishes that each in-range value converts to
  `Ok(value)`. The formatter uses it for the concrete constants `999` and
  `10_000`; the blanket `TryInto` model only forwards the associated
  `TryFrom` precondition and postcondition and assumes no generic totality.
  The existing `Result::expect` model requires `Ok`, which the range facts
  prove for those constants.
- `TryFrom<i32> for u32` establishes `Ok(value)` when the input is
  nonnegative; every nonnegative `i32` fits in `u32`. The same blanket
  `TryInto` forwarding model and `Result::expect` contract apply to the
  formatter's concrete constants.
- `TryFrom<i32> for u64` establishes `Ok(value)` when the input is
  nonnegative; every nonnegative `i32` fits in `u64`. The same blanket
  `TryInto` forwarding model and `Result::expect` contract apply to the
  formatter's concrete constants.
- The `get_unchecked` standard model retains its in-bounds/result contract and
  has a checked termination classification for the table's built-in `usize`
  indices.

The native standard-library source bodies are not translated as part of this
crate proof. Removal condition: verify the native bodies or integrate
equivalent source-level models with the same exact preconditions and
postconditions. These models provide no decimal-formatting arithmetic facts.

### Standard-library model assumptions used by Phase 7

The signed `i8`, `i16`, `i32`, `i64`, and `i128` buffer writers use three
additional external models for native core operations. They are assumptions about those operations, not
trusted formatter functions:

- Array `IndexMut` is modeled only for the two index forms used by the writer:
  `usize` and `RangeFrom<usize>`. The model requires the standard
  `SliceIndexSpec` bounds condition and describes the selected current and
  final views, unchanged array length, and the frame outside a `RangeFrom`
  index. A local whitelist prevents the contract from applying to unrelated
  index types. The pinned core array implementation is at
  `library/core/src/array/mod.rs:394–403`; the corresponding slice-index
  implementations are at `library/core/src/slice/index.rs:214–279` and
  `:541–588`.
- `TryFrom<&mut [T]> for &mut [T; N]` requires the slice length to equal `N`.
  Under that condition it returns `Ok` with the same current and final views
  as the input slice, including pointwise equality for each array element.
  This models the exact-length branch in pinned core at
  `library/core/src/array/mod.rs:312–333` and
  `library/core/src/slice/mod.rs:862–878`; the unsigned suffix slices have 3,
  5, 10, 20, and 39 slots for the i8, i16, i32, i64, and i128 writers.
- Primitive `unsigned_abs` for `i8`, `i16`, `i32`, `i64`, and `i128` returns
  the mathematical absolute value, including each signed minimum. This matches
  the core integer macro implementation using `wrapping_abs` followed by the
  corresponding unsigned cast (`library/core/src/num/int_macros.rs:2397–2403`
  and `:2421–2423`). The integrated signed-body proofs consume the `i8`,
  `i16`, `i32`, `i64`, and `i128` instances.

The standard-library bodies are not translated in this crate proof. Removal
condition: verify those native implementations or replace the external models
with verified source-level adapters that preserve these exact bounds and
current/final view relations. These models establish no decimal digit facts.

For a negative control, an isolated copy changed the first four-digit loop
relation from equality to inequality. Creusot then failed specifically on that
assertion, confirming that this loop proof is sensitive to the actual chunk
arithmetic.

Immediately after Phase 1 and before the Phase 2 table patch, the default and
`--all-features` `verify-all.bash` runs both passed. Each configuration produced
69 proof units and 190 split goals with zero failures. The `magic_quotient`
arithmetic lemma and the actual shared `divmod100` body each discharged one VC.
The crate's established recursive-model obligations also remained green.

## Runtime test checks

The earlier results below are phase-scoped historical checks. The current
primary workspace passes 12 integration tests and 2 doctests with default
features. The release all-features run also passes 12 integration tests and 2
doctests with fat LTO, one codegen unit, and matching `RUSTDOCFLAGS`. The exact
commands and logs are in the [native runtime check record](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/native/README.md).

The no-panic investigation found two separate instrumentation issues. First,
the private `write_decimal_digit`, `write_decimal_pair`, and
`write_decimal_quad` helpers had no-panic attributes despite requiring caller-
supplied bounds or value ranges, so their contracts cannot promise unconditional
panic freedom. The accepted correction removes only those
three checker attributes; the public `Buffer::format` and sealed writer
attributes remain. This changes metadata only: executable bodies and Creusot
contracts are unchanged, and no additional trusted fact is introduced.
Second, rustdoc's generated doctest binaries need the release optimization
flags explicitly in `RUSTDOCFLAGS`; with those flags, both release
all-features doctests pass. The older test results below predate the added
reused-buffer regression test and the helper-attribute correction.

After extracting the shared leaf, `cargo test --manifest-path Cargo.toml`
passes all 11 integration tests and 2 doctests. The release test command
`cargo test --manifest-path Cargo.toml --tests --all-features --release`
passes all 11 integration tests. `--tests` excludes doctests from that
all-features release run. These normal and release commands also passed after
Phase 2 extracted the table declaration; a normal `cargo check` passed the
compile-time table equality assertion. The same commands were rerun after
adding the actual `u32`, `u64`, and `u8` formatter bodies and both passed: the
normal run had 11 integration tests and 2 doctests, and the release all-features
run had 11 integration tests. Setup also recorded that
`cargo test --all-features` fails at the `no-panic` linker step in debug, and
`cargo test --all-features --release` passes integration tests but fails while
linking doctests. The all-features proof configuration separately enables
`no-panic`; it does not run the upstream test suite.

A diagnostic copy of original upstream HEAD passed 24 tests across 14 test
targets after removing only the three private-helper `no_panic` attributes.
A mirrored current-source candidate passed 12 tests with fat LTO and one
codegen unit; the primary-workspace result is recorded above. The
baseline diagnostic run used this command:

```sh
env PATH=/tmp/cargo-home/bin:$PATH \
  RUSTUP_HOME=/tmp/rustup-home \
  RUSTUP_TOOLCHAIN=nightly-2026-02-27 \
  CARGO_HOME=/tmp/cargo-home \
  CARGO_NET_OFFLINE=true \
  CARGO_PROFILE_RELEASE_LTO=fat \
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
  CARGO_TARGET_DIR=/workspace/proof-tools/targets/itoa-boundary-baseline-native \
  cargo test --offline --locked --release --all-features --tests \
    --manifest-path /tmp/itoa-boundary-baseline/itoa/1.0.18/Cargo.toml
```

The mirrored run is separate from the primary-workspace result above.

## Baseline and run record

The crate-scoped verification command is `./verify-all.bash`, run from
`itoa/1.0.18`. It invokes both the default and `--all-features` Creusot proof
configurations, with Cargo networking disabled. The all-feature run enables the
optional `no-panic` dependency; it is not an upstream test-suite run.

Baseline at source commit `c6d8352aecd423d311f26e64bef3ed37ec950b84`:
`./verify-all.bash` from this directory. Both the default and `--all-features`
configurations passed before Phase 1 changes: each translated 67 libraries
and discharged 188 verification conditions with zero failures. The driver was
`cargo-creusot 0.11.0-dev` from upstream commit
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`, using `nightly-2026-02-27`
(`rustc 1.95.0-nightly (6a979b3e3 2026-02-26)`, with `rustc-dev` and
`llvm-tools`). The root README records `creusot-libs` commit
`7a48f5a5b1cb15a11c4e744568ca187331a30025`, while the vendored library declares
`0.11.0-dev`; the matching driver above is built from the latter source line.

Why3 was `1.8.2+git` at commit
`2c0f2992af85f82f3eda0f158dcf10e62e0db875`; Why3find was `v1.2.0+dev` at
`3a98fc320b9cbf2e71860da1c8dc188a966eee96`. Solver versions were Alt-Ergo
2.6.2, Z3 4.15.3, CVC4 1.8, and CVC5 1.3.1.

The proof environment used `/tmp/rustup-home`, `/tmp/cargo-home`,
`/tmp/creusot-data`, `/tmp/creusot-cache`, and `/tmp/creusot-config` (symlinked
to `/workspace/proof-tools` to avoid `/tmp` disk limits), with
`LD_LIBRARY_PATH` including the nightly toolchain's `lib` and
`/tmp/local-ocaml/usr/lib/x86_64-linux-gnu`. `verify-all.bash` sets
`CARGO_NET_OFFLINE=true`; Why3 proof runs were outside the sandbox because they
use Unix sockets. Baseline used
`CARGO_TARGET_DIR=/workspace/proof-tools/targets/itoa-baseline`; Phase 1 used
`CARGO_TARGET_DIR=/workspace/proof-tools/targets/phase1`.
After Phase 2, `./verify-all.bash` passed in both default and `--all-features`
configurations with 70 proof units and 192 split goals per run (up from 69 / 190
after Phase 1). The Phase 2 change adds no trusted declarations. The proof
configuration change to a scalar `const` is paired with the normal-config CTFE
byte equality assertion described above.

At the initial `u16`-only Phase 3 checkpoint, a clean crate-local
`./verify-all.bash` run passed in both configurations. Each configuration
proved 103 libraries and 514 reported split goals with zero failures; `fmt_u16`
contributed 201 goals and `fmt__refines` one. The ordinary test command passed
11 integration tests and 2 doctests; the optimized all-features test command
passed all 11 integration tests.

After adding the actual `u32` formatter helpers, a fresh `./verify-all.bash`
completed with exit 0 for both default and `--all-features` configurations.
The log is `/tmp/itoa-u32-verify-fresh-main.log`; each config reports
`u32::fmt` at 102 goals and its refinement at one, and `u16::fmt` at 104 with
one refinement. A separate no-cache targeted replay is recorded at
`/tmp/itoa-u32-bodies-current.log`; it proves `u32::fmt` at 104 goals and its
refinement at one, and rechecks `u16::fmt` at 105 plus one refinement.

After adding the actual `u64` formatter body, a fresh `./verify-all.bash`
completed with exit 0 for both configurations; the log is
`/tmp/itoa-u64-verify-fresh-main.log`. Each run reports 104 goals for
`u64::fmt` and one refinement goal, 102 for `u32::fmt` and one refinement
goal, and 104 for `u16::fmt` and one refinement goal. The no-cache targeted
U64 body/refinement replay is in `/tmp/itoa-u64-targeted-final.log` (105 and
one goal). The U64 negative control changed the first four-digit chunk
equality to inequality and failed at that assertion; the actual source was
restored and re-proved. Native test logs are
`/tmp/itoa-u64-tests-default.log` and
`/tmp/itoa-u64-tests-allfeatures-release.log`; they contain 11 integration
tests plus 2 doctests in the default run and 11 integration tests in the
all-features release run.

After enabling the actual `u8` formatter body, a fresh translation followed by
`./verify-all.bash` completed with exit 0 for both configurations; the log is
`/tmp/itoa-u8-verify-fresh-main.log`. Each run discharged 98 goals for
`u8::fmt` plus one refinement goal, along with the existing u16/u32/u64 proofs
(104/102/104 formatter goals and one refinement goal each). The no-cache
targeted U8 body and refinement runs are in
`/tmp/itoa-u8-targeted-body.log` (98) and
`/tmp/itoa-u8-refines-targeted.log` (1). Default tests passed 11 integration
tests and 2 doctests; all-features release tests passed 11 integration tests,
recorded in `/tmp/itoa-u8-tests-default.log` and
`/tmp/itoa-u8-tests-allfeatures-release.log`.

After adding Phase 5, a fresh `./verify-all.bash` completed with exit 0 in both
configurations. The log is `/tmp/phase5-official-fresh.log`: default reported
134 proof units and 1,141 split goals; `--all-features` reported 135 units and
1,145 goals. Both runs proved `enc_16lsd` (55 goals), `write_decimal_quad`
(87), and its runtime digit bridge (50), while retaining the Phase 3 formatter
proofs. `cargo test --manifest-path Cargo.toml` passed 11 integration tests and
2 doctests; `cargo test --manifest-path Cargo.toml --tests --all-features
--release` passed all 11 integration tests. Their logs are
`/tmp/phase5-tests-default.log` and
`/tmp/phase5-tests-release-allfeatures.log`.

## Phase 6: actual `u128` formatter body

The native `Unsigned::fmt` implementation for `u128` now shares one body
between normal Rust and Creusot. It retains reciprocal division, the 16-digit
encoder calls, four-digit and two-digit table writes, and the final masked
digit. Its contracts prove the exact returned offset, initialized output
suffix, canonical decimal bytes, and preservation of the unwritten prefix.
The capacity fact is derived inside the body from
`decimal_values_len_u128`; no input precondition was added.

At the Phase 6 checkpoint, the body and its trait refinement passed 140 and 1
goals in both full-suite configurations. The default run reported 182 proof
libraries / 1,637 VCs; all-features reported 183 / 1,641, with zero failed
goals. At this Phase 6 checkpoint, the result depended on Phase 4's exact trusted
`mulhi` high-half contract; the divider and formatter proofs did not derive
that contract from the limb core. Default native tests passed 11 integration tests and 2
doctests, and release all-features integration tests passed all 11. Logs are
`/tmp/phase6-verify-all-integrated.log`,
`/tmp/phase6-native-test-default.log`, and
`/tmp/phase6-native-test-release-allfeatures.log`.

This phase proved the actual unsigned `u128` formatter body. The Phase 7
checkpoint below strengthens its state-sequence interface and proves the
signed `i128` buffer writer. Raw `Buffer::format`, the borrowed-`str`
conversion, and the remaining signed wrappers are still pending.

## Phase 7: signed `i128` buffer writer

The signed writer keeps the original one-byte leading gap, calls the actual
`u128::fmt` body on the 39-byte suffix through the original
slice-to-array `try_into().unwrap()`, and writes `'-'` for negative values.
Its contract preserves the unwritten prefix and proves the exact initialized
suffix bytes and returned offset. A concrete `i128::MIN` witness establishes
that all 40 slots are initialized and match the canonical signed decimal
sequence; the unsigned suffix has 39 digits and the returned start is zero.

The integrated default proof run reported 196 libraries / 1,696 VCs; the
all-features run reported 197 / 1,700, with zero failed goals. It proved the
actual `u128::fmt` body (147 VCs and one refinement), `write_u128_suffix_i128`
(24), `signed_write_i128` (4), and the `i128::MIN` whole-buffer witness (4).
At this Phase 7 checkpoint, the underlying `u128` arithmetic was still
conditional on the accepted exact `mulhi` high-half result contract. No
formatter function was trusted locally.

The native default suite passed 11 integration tests and 2 doctests. Release
all-features `--tests` passed all 11 integration tests. Logs are
`/tmp/phase7-i128-final-verify-all.log`,
`/tmp/phase7-i128-native-default.log`, and
`/tmp/phase7-i128-native-release-all-features-tests.log`.

Raw `Buffer::format`, the borrowed-`str` conversion, and pointer `usize` and
signed `isize` adapters were still pending at this signed-i128 checkpoint.

## Phase 7: signed `i16`, `i32`, and `i64` buffer writers

The shared small-signed writer macro extracts the native `unsigned_abs`,
capacity gap, actual unsigned formatter call, and optional sign write without
changing that operation sequence. The i16 and i32 paths preserve one leading
slot and call the actual u16/u32 formatter on the remaining 5/10 slots. The i64
path calls the actual u64 formatter on its 20-slot buffer; the signed capacity
lemma proves every i64 magnitude uses at most 19 digits, leaving one slot for
the sign when needed, including `i64::MIN`. Each signed writer proves canonical
bytes, initialization, returned offset, and the unwritten-prefix frame for all
inputs. Their all-input harnesses consume those contracts.

The current integrated default proof reported 210 libraries / 1,869 VCs and
all-features reported 211 / 1,873, with zero failed goals. The i16 writer and
harness discharged 4 and 3 goals, with a 27-goal suffix writer. The i32 writer
and harness discharged 4 and 3 goals, with a 27-goal suffix writer. The i64
writer and harness discharged 4 and 3 goals, with a 22-goal suffix writer; its
typed decimal-capacity lemma discharged one goal. At this Phase 7 checkpoint, the result inherited the Phase 4 `mulhi` result
contract only along the `u128` formatting path; the current focused closure
proves that contract from the core.

Native default tests passed 11 integration tests and 2 doctests; release
all-features `--tests` passed all 11 integration tests on the exact patch
source. The integrated proof log is
`/tmp/phase7-small-signed-main-verify-all.log`; focused and native logs are in
`/tmp/phase7-small-current-i8/`.

Raw `Buffer::format`, the borrowed-`str` conversion, and pointer `usize` and
signed `isize` adapters were still pending at this signed-small checkpoint.

## Phase 7: signed `i8` buffer writer

The original i8 path is extracted into `signed_write_i8` and
`write_u8_suffix_i8`; the normal runtime uses these same functions. The suffix
helper retains the one-byte capacity gap, exact-length slice-to-array
conversion, and call to the actual `Unsigned::fmt(u8)` body. The signed helper
keeps the original `unsigned_abs` call and optional `'-'` write. Its contract
proves the canonical signed bytes, initialization, returned offset, and
preservation of the unwritten prefix for every i8 value.

To let the signed adapter consume the actual unsigned formatter result, the
shared `Unsigned::fmt` contract now also states that the initialized output
suffix's logical slot-state sequence equals `decimal_values(self@)` and that
the unwritten prefix's slot states are preserved. All previous pointwise byte,
initialization, length, and prefix contracts remain in place. A small proved
bridge equates the byte and slot-state ghost views only on ranges whose slots
are already known initialized.

At the i8 checkpoint, the integrated default run reported 200 proof libraries
/ 1,771 VCs and all-features reported 201 / 1,775. The actual
unsigned formatter bodies discharged 104 goals for u8, 113 for u16, 116 for
u32, and 119 for u64; each has one trait-refinement goal. The i8 suffix writer
discharged 23 goals, the signed writer 4 in the integrated run (16 in its
focused target), and its all-input contract harness 3. The state-view bridge
discharged one goal. The actual u128 and i128 proof counts remain as recorded
above. At that i8 checkpoint, the `u128` path still inherited Phase 4's accepted
`mulhi` result contract.

The native default suite passed 11 integration tests and 2 doctests. Release
all-features `--tests` passed all 11 integration tests. Logs are
`/tmp/phase7-i8-main-verify-all.log`,
`/tmp/phase7-i8-main-native-default.log`, and
`/tmp/phase7-i8-main-native-release-all-features-tests.log`.

At this i8 checkpoint, raw `Buffer::format`, the borrowed-`str` conversion,
pointer adapters, and signed wrappers other than i8/i16/i32/i64/i128 remained
pending.

## Phase 7: 64-bit pointer-width buffer writers

On x86_64, the `usize` adapter uses a same-width `u64` cast bridge and calls
the actual `Unsigned::fmt(u64)` body. The `isize` adapter uses a same-width
`i64` cast bridge and calls the actual `signed_write_i64` body. Both adapter
contracts establish the canonical initialized output suffix, returned offset,
and preservation of the unwritten prefix; the cast bridges preserve the
integer view and decimal model. The production `Integer` implementation
dispatches to these same helpers on 64-bit targets. The 16- and 32-bit fallback
dispatches are unchanged and are not covered by this target-specific proof.

The integrated default proof reported 214 libraries / 1,877 VCs, and the
all-features proof reported 215 / 1,881, with no failed goals. Each cast bridge
discharged one VC and each buffer-writer adapter discharged three. The full
suite log is `/tmp/phase7-pointer64-19df-main-verify-all.log`. Native default
tests passed 11 integration tests and 2 doctests; release all-features
`--tests` passed all 11 integration tests. Their logs are
`/tmp/phase7-pointer64-19df-native-default.log` and
`/tmp/phase7-pointer64-19df-native-release-all-features-tests.log`.

This proves the pointer-sized numeric buffer writers on the current x86_64
target. The normal `Sealed::write` path still converts the initialized byte
suffix into `&str`, and public `Buffer::format` still crosses raw buffer
memory; neither memory boundary is included here.

## Phase 8 checkpoint: initialized ASCII suffixes and earlier runtime boundaries

This section records the Phase 8 state before the accepted Boundary A/B source
changes. Its statements that the actual public method is excluded and the
verification model has a trusted string leaf are historical; current source
status is described in [Accepted runtime boundary changes](#accepted-runtime-boundary-changes)
and [Current end-to-end status](#current-end-to-end-status).

The new `decimal_values_ascii`, `signed_decimal_values_ascii`, and
`integer_decimal_values_ascii` lemmas reuse the existing canonical decimal
models. Each model lemma passed its focused one-goal proof. The actual signed
i8 writer witness passed with 3 goals, including the initialized suffix ASCII
assertion, and the actual signed `i128::MIN` witness passed with 4 goals,
including the complete 40-slot initialization and ASCII assertion. The full
dual-configuration `verify-all.bash` run passed with 217 proof libraries /
1,880 VCs by default and 218 / 1,884 with all features, with no failed goals.
Native `cargo test --offline --locked` passed 11 integration tests and 2
doctests; release `cargo test --release --all-features --tests --offline
--locked` passed all 11 integration tests. Logs are
`/tmp/phase8-runtime-verify-all.log`, `/tmp/phase8-native-default.log`, and
`/tmp/phase8-native-release-all-features-tests.log`.

The writer contracts establish that every byte in `[offset, N)` is initialized
and matches its unsigned or signed decimal model, while `[0, offset)` retains
its old `MaybeUninit` state. The source model of `MaybeUninit::write` records
the transition to `Some(value)`, and the new lemmas prove the canonical bytes
are ASCII. No initialization-state fact was missing. `Buffer::format`'s
40-byte-to-typed-buffer pointer reinterpretation and
`slice_buffer_to_str`'s raw `MaybeUninit` slice conversion and unchecked `&str`
construction remain outside Creusot; the recursive model's trusted
initialized-`[u8; 40]` string leaf does not prove those runtime operations.
Only the 64-bit `usize` and `isize` writer adapters are covered on x86_64; the
16- and 32-bit fallback adapters remain pending. At this Phase 8 checkpoint,
the `u128` writer inherited the then-accepted `mulhi` high-half contract; the
current joint-helper closure proves that equation from the core. Full
source details and standard-library assumptions are in
[RUNTIME_MEMORY_LEDGER.md](RUNTIME_MEMORY_LEDGER.md). The Creusot/Verus boundary
contracts, Phase 11 partial Verus result, and exact remaining raw-memory gaps
are recorded in [CROSS_TOOL_BOUNDARIES.md](CROSS_TOOL_BOUNDARIES.md); the
standalone post-conversion Verus lemma is
[verus/ascii_bytes_to_str.rs](verus/ascii_bytes_to_str.rs).

## Current end-to-end status

The current x86_64 integrated Creusot run passes in both configurations.
Default reported 254 proof libraries / 2,083 VCs; all-features reported 269 /
2,145. The actual `Buffer::format` body discharged 3 VCs by default and 5 with
all features. The initialized-suffix string conversion discharged 4 and 6.
The archived full output is
[`integrated/fullsuite.log.gz`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/integrated/fullsuite.log.gz),
and the aggregate results are in the
[`runtime boundary report`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/REPORT.md).

All 12 concrete x86_64 writers (five unsigned, five signed, and `usize`/`isize`)
and all 12 refinements pass. In the focused run, `u8::write` used 20 VCs, each
other writer used 8, and each refinement used 1, for 120 VCs total. The
all-features instrumentation discharges 24 VCs for the `u8` writer and 10 for
each other writer. Each concrete `Sealed::write` implementation states its own
exact decimal-byte and maximum-length postconditions. The `u128` formatter and
signed `i128` writer consume the exact high-half equation proved from
`mulhi_core` (68/68 focused goals; see [Proved `mulhi` closure](#proved-mulhi-closure)).
The focused writer log is
[`writers/focused-all/focused-proof.log`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/writers/focused-all/focused-proof.log).
The ASCII proof discharged `ascii_byte_map_to_utf8` in 30 VCs and
`ascii_bytes_are_utf8` in 3, with no trusted helper; its log is
[`ascii/focused-final/focused-proof.log`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/ascii/focused-final/focused-proof.log).

Boundary A uses the safe `.try_into().unwrap()` prefix conversion; the focused
concrete writer proofs establish success for all 12 sealed types. Boundary B
is isolated in `assume_init_slice`: its caller proves the initialization
precondition in the logical writer model, and its local trust describes only
the borrowed slice view, length, and bytes. The physical permission transfer
remains unproved. No formatting arithmetic or table-correctness fact is
trusted. The recursive model's former trusted `decimal_slice_to_str` leaf has
been removed; its ordinary replacement requires a proved ASCII suffix and
retains the output-byte postcondition. The formatter uses the proved
`CharExt::to_utf8` logic model, discharged in 2 VCs; that proves the model, not
Rust core's runtime character encoder. Creusot's standard/core operation models
remain assumptions. The `cfg(not(creusot))` sealed-trait declaration, the
conditional `no_panic` instrumentation, and the cfg-specific table
representations are metadata/model accommodations; the compile-time table
equality check ties the representations, and no executable formatter body is
excluded from the proof target. The 16- and 32-bit `usize`/`isize` paths remain
outside this x86_64 proof scope.

Native checks also pass: the default debug command and the optimized
all-features command each passed 12 integration tests and 2 doctests. The latter
uses fat LTO, one codegen unit, and explicit matching `RUSTDOCFLAGS`; exact
commands and logs are in the
[`native runtime check record`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/native/README.md).

The pre-bridge Phase 12 runs reported 218 libraries / 1,884 VCs before the
`mulhi` closure and 224 / 1,917 afterward, with no failed goals. Those runs
proved the then-current writer and arithmetic components but excluded the
public runtime method; they are not current integrated totals. The partial
Verus artifact reported 1 verified / 0 errors for an already-formed ASCII
`&[u8]`; its result does not cover the raw slice conversion and relies on
vstd's assumed `from_utf8_unchecked` specification. Historical logs are
`/tmp/itoa-phase12-verify-all.log`, `/tmp/itoa-phase12-native-default.log`,
`/tmp/itoa-phase12-native-release-tests.log`, and
`/tmp/itoa-phase12-verus-ascii.log`.

The current `CharExt::to_utf8` logic model is an open Unicode UTF-8 encoding
definition built from `utf8_byte`; its focused proof discharged 2 logic VCs.
This proves the mathematical model consumed by the ASCII witness, not Rust
core's runtime character encoder. The focused report is
[`stdlib-utf8/REPORT.md`](../../tools/creusot-toolpatch/proofs/runtime-boundary-bridge/stdlib-utf8/REPORT.md).

The pre-bridge assumption ledger no longer includes the `mulhi` equation. For
the current source, the one intended local trust is `assume_init_slice`; narrow
Creusot standard/core operation models remain separate, and the Verus
`from_utf8_unchecked` specification applies only to its separate partial
artifact. Standard-library operation models used in the current Creusot model
remain assumptions. The recursive model's former trusted leaf is removed. No
decimal-correctness or cross-tool memory contract is trusted. The current A/B
contracts, historical raw-pointer probes, and Kani/Miri feasibility assessment
are in [CROSS_TOOL_BOUNDARIES.md](CROSS_TOOL_BOUNDARIES.md). Aggregate proof and
native test results are recorded above and in the linked reports.

### Historical pre-integration verification of the boundary note

Before the Phase 4 source integration, the unchanged `itoa/1.0.18` source
passed a fresh `tools/creusot-toolpatch/scripts/run-verify-all.sh
/workspace/rust-crate-proofs` run: default reported 134 proof units / 1,141
goals, and all-features reported 135 units / 1,145 goals. The normal native
test command passed 11 integration tests and 2 doctests; the release
all-features `--tests` command passed all 11 integration tests. The complete
logs are preserved in
`tools/creusot-toolpatch/proofs/phase4-conditional/logs/`. This run is a
historical validation of the boundary documentation and evidence package; the
conditional Phase 4 source had not yet been applied.
