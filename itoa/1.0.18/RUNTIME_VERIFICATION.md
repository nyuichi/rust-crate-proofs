# itoa 1.0.18 runtime verification status

## Scope map

The production `runtime` module contains the optimized implementation: a
`MaybeUninit` output buffer, the decimal-pair lookup table, reciprocal division
by 100, four-digit chunking, the specialized `u128` path, and output string
construction. Under `cfg(creusot)`, this phase compiles `runtime.rs` and proves
the actual shared `Unsigned::fmt` bodies for `u8`, `u16`, `u32`, and `u64`. The
public formatting facade still comes from `verification.rs`; raw
`Buffer::format`, signed adapters, and the `u128` formatter caller remain
outside this phase. The `u64` 16-digit chunk encoder called by that native
caller is now translated and proved as a separate component.
The recursive decimal model in `verification.rs` remains the formatter's
specification, and all existing model proofs are retained.

The first shared implementation boundary is `divmod100`: normal runtime callers
use this body, and Creusot translates and proves the same executable body as an
independent leaf. Phase 3 connects the actual `u8`, `u16`, `u32`, and `u64`
formatter bodies to the decimal model; adapters remain later phases.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated runtime proof |
| --- | --- | --- | --- | --- |
| Existing recursive decimal model and public verification-facing API | yes | yes (per current provenance record) | pre-existing ASCII-slice-to-`str` leaf | baseline only; does not cover runtime code |
| Shared optimized `divmod100` (Phase 1) | yes | yes | none added | yes; leaf only, callers remain pending |
| `DECIMAL_PAIRS` representation and indexed lookup lemma | yes | yes (2 VCs) | none | data bridge via CTFE; formatter use proved for `u8`, `u16`, `u32`, and `u64` |
| Actual optimized `Unsigned::fmt` body for `u16` | yes | yes (104 goals in current integrated runs; 105 targeted no-cache) | no formatter boundary | yes; default and all-features |
| Actual optimized `Unsigned::fmt` body for `u32` | yes | yes (102 goals in current integrated runs; 104 targeted no-cache) | no formatter boundary | yes; default and all-features |
| Actual optimized `Unsigned::fmt` body for `u64` | yes | yes (104 goals in current integrated runs; 105 targeted no-cache) | no formatter boundary | yes; default and all-features |
| Actual optimized `Unsigned::fmt` body for `u8` | yes | yes (98 goals) | no formatter boundary | yes; default and all-features |
| Actual `enc_16lsd` 16-digit chunk encoder body (Phase 5) | yes | yes (`enc_16lsd`: 55 goals; quad writer: 87; digit bridge: 50) | none | yes; component in both full crate proof configurations, not the `u128` caller |
| `u128_ext::mulhi` high-half product body (Phase 4) | yes | no (49/51 in the latest bounded attempt; two obligations remain) | none declared; its result contract is consumed by the conditional divider proof | pending |
| `div_rem_1e16` reciprocal divider body (Phase 4) | yes | 31/31 conditionally, assuming the `mulhi` result contract | no trusted attribute; the actual `mulhi` body is not yet proved | staged evidence only; not integrated |
| `u128` formatter caller | pending | no | none | pending |
| Raw `Buffer::format`, signed adapters, and output-string integration | pending | no | no new boundary planned | pending |

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

The remaining Phase 4 boundary is the actual 128-bit high-half multiply used
by the reciprocal divider. The runtime quotient is computed with the original
Granlund–Montgomery operation: multiply by
`76_624_777_043_294_442_917_917_351_357_515_459_181`, take the high half, then
shift it right by 51; the divider subtracts `quotient * 10^16` to obtain the
remainder. The staged divider contract states the exact quotient, remainder,
reconstruction, and remainder bound. No replacement arithmetic is used.

The divider components have completed focused proofs in a candidate based on
`51090b4`: `shift_by_51` passed 2/2 goals, `magic_quotient` passed 10/10,
`math::magic_shift_floor` passed 1/1, and `div_rem_1e16` passed 31/31. These
divider results are conditional because their callers consume the `mulhi`
postcondition while its body remains unfinished. The actual `mulhi` body has a
bounded 49/51 result; the two remaining formulas are:

```text
x * y == result * 2^128 + residual
quotient == result
```

The second obligation occurs after a call to
`exact_floor_from_split(x * y, result, residual, 2^128)`. The neighboring
`quotient == x * y / 2^128` obligation passed. This run stopped at its 90-second
bound; it does not show that either formula is unprovable. The issue is the
caller-side composition of the residual helper with the reconstruction and
exact-floor facts. No trusted declaration was added, and no complete `u128`
runtime proof is claimed.

The candidate patch, source, generated COMA, proof JSON, and bounded-run report
are preserved under
[`tools/creusot-toolpatch/proofs/phase4-conditional/`](../../tools/creusot-toolpatch/proofs/phase4-conditional/).
The divider patch and the `mulhi` candidate patch were developed separately
against `51090b4`; both change `u128_ext.rs`, so they must not be applied in
sequence. Each artifact's README gives its isolated replay steps. Use the pinned
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

Six narrow external models describe native standard-library operations used
by the Phase 3 formatter. These are assumptions about core APIs, not trusted
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

After recording the Phase 4 boundary, the unchanged `itoa/1.0.18` source passed
a fresh `tools/creusot-toolpatch/scripts/run-verify-all.sh
/workspace/rust-crate-proofs` run: default reported 134 proof units / 1,141
goals, and all-features reported 135 units / 1,145 goals. The normal native
test command passed 11 integration tests and 2 doctests; the release
all-features `--tests` command passed all 11 integration tests. The complete
logs are preserved in
`tools/creusot-toolpatch/proofs/phase4-conditional/logs/`. This validates the
documentation and evidence-package change against the current integrated
source; the conditional Phase 4 candidate patches were not applied to this run.
