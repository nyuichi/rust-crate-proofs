# Gate C source plan: runtime CPU capabilities

Status: design only. This note proposes an isolated Gate C delta on top of the
frozen Gate B patch. It does not change compiler or library source and makes no
solver claim.

## Claim and model

The checked predicate is `I(value: u8, avx2: bool, sse42: bool)` with the
closed total body:

```text
value == 0 || (value == 1 && avx2) || (value == 2 && sse42) || value == 3
```

For this registered static, the compiler supplies the *same two immutable
capability symbols* at the initializer, every store, and every load. The
symbols are compiler-bound model values for usable AVX2 and SSE4.2, including
the operating-system state needed to execute those instructions. They are not
caller parameters, independently fresh booleans, or arbitrary calls in the
predicate. The existing Gate B load remains a fresh `UInt8.t` constrained by
`I`; it still makes no latest-read or equal-load claim.

This predicate proves supported dispatch, not feature priority. It permits a
tag of `2` when both capabilities are true, even though the detector prefers
AVX2 in that case. The cache tag `0` is permitted as an uninitialized value,
but `get_runtime_feature` must return only `1`, `2`, or `3`.

## Source delta inventory

Keep the new compiler patch separate, named
`httparse-static-atomic-gate-c.patch`, and apply it after the reviewed Gate B
patch. Do not change Gate B's frozen patch or its evidence.

| Area | Planned source delta | Required guard |
| --- | --- | --- |
| Predicate and static audit | Extend `creusot/src/static_atomic.rs` with a Gate C registration mode for exactly three non-generic arguments `(u8, bool, bool) -> bool`. Extend the closed total expression checker to accept only those formal arguments, the existing exact `u8 View`, literals, comparisons, `!`, `&&`, and `||`. | Reject user-supplied capability expressions, external calls, trusted/opaque/prophetic predicates, generic parameters, and any unknown AST. Preserve Gate B's one-argument mode unchanged. |
| Capability identity | Add two compiler-known symbols for usable AVX2 and SSE4.2 and resolve them from the pinned standard-library/compiler model, not from user-provided names. Bind the same identities in each generated module. | The identities must not be caller arguments or independently chosen values. The detector identities must be exact resolved sysroot definitions and exact feature constants (`avx2`, `sse4_2`). A copied builtin annotation or same-name local item must fail. |
| Static VCs | Extend `creusot/src/backend/static_atomic.rs` and the `StaticAtomicLoad`/`StaticAtomicStore` lowering in `creusot/src/backend/program.rs` so initializer, store assertion, and fresh-load assumption all call `I(value, A, S)`. | Do not add a store history axiom, latest-value axiom, or new permission mint. Reuse Gate B's checked initializer/store goals and nondeterministic load. |
| Detector coverage | Extend the registered-body/access inventory in `creusot/src/static_atomic.rs` so it includes the actual `detect_runtime_feature` body and both exact detector sites, in addition to `get_runtime_feature`, all cache sites, and both dispatchers. | Every normal and proof MIR detector site must match and lower exactly once. Skipped, trusted, excluded, or unregistered writers/detectors fail closed. |
| Registration manifest | Give Gate C a versioned manifest mode while retaining Gate B v2 behavior. Record a fixed `x86-runtime-caps-v1` profile and the static/predicate registration; compute detector identities from compiler-resolved sysroot definitions rather than accepting DefPaths for capabilities. | The profile, target triple, source/cfg/target-feature inputs, compiler/patch/runner, and normal/proof body fingerprints must all enter the audit digest. Unknown profiles and Gate B manifests fail closed for Gate C. |
| Runtime source/model | In `src/simd/runtime.rs`, register `RUNTIME_FEATURE` with the three-argument predicate under verification cfg. Keep the runtime `AtomicU8`, ordering, detector calls, branch priority, and dispatcher bodies unchanged. | The model predicate's third and fourth cases remain explicit; no crate-specific trusted postcondition is added to `get_runtime_feature` or either dispatcher. |
| Standard/compiler TCB | Add narrowly scoped specs for the exact detector observations and for the target-feature call preconditions. The detector contract is `observation == true ==> corresponding usable capability`. | This is an explicit standard/compiler feature-safety TCB. It must include OS-enabled state and capability stability for the admitted process. It does not assert either capability is true, assert detector equality, or bodycheck CPUID/OS detection internals. |
| Build and proof profile | Freeze the target triple, Rust compiler, `std` feature, build-script output, Cargo target-feature list, and environment in both audit inputs. | Start on x86/x86_64 with `std` and `httparse_simd` enabled, Miri and SIMD-disable settings absent, and both `avx2` and `sse4.2` absent from target features. Other targets/profile states reject. |

The proof translation sites that consume Gate B's current single-argument
predicate are `creusot/src/backend/static_atomic.rs` and
`creusot/src/backend/program.rs`; the registered direct-call recognition is in
`creusot/src/translation/function/terminator.rs`. Gate C should change those
sites only in its isolated delta and keep Gate B's comparison and required-site
coverage checks intact. If emitting shared Why3 capability names requires a
new term or dependency representation, add it to the Gate C patch and test
identity across separate static and accessor modules before translating the
httparse body.

## Detector and target-feature boundary

The actual detector body in `src/simd/runtime.rs` tests AVX2 first, then SSE4.2,
then returns NOP. The pinned `std_detect` macro expansion contains a
`cfg!(target_feature = ...) || __is_feature_detected::<feature>()` shortcut.
The registered observations must be the exact resolved sysroot calls for
`avx2` and `sse4_2`; the macro's source strings are `"avx2"` and `"sse4.2"`.

`build.rs` emits `httparse_simd` when `std` is enabled and SIMD/Miri are not
disabled. It emits `httparse_simd_target_feature_avx2` and
`httparse_simd_target_feature_sse42` from `CARGO_CFG_TARGET_FEATURE`.
`src/simd/mod.rs` selects `runtime.rs` only on x86/x86_64 when SIMD is enabled
and both custom target-feature cfgs are absent. Gate C must additionally audit
the actual Rust target-feature cfgs and resolved MIR calls: a build with
`-C target-feature=+avx2` or `+sse4.2` can make the standard macro shortcut
return true without calling the detector. Setting
`CARGO_CFG_HTTPARSE_DISABLE_SIMD_COMPILETIME=1` only bypasses httparse's
compile-time module selection; it does **not** disable the macro shortcut.
Such a forced-runtime build is therefore not in this proof profile.

The target-feature entry obligation for each AVX2/SSE4.2 unsafe function must
come from an exact target-feature contract tied to its resolved definition
(`#[target_feature(enable = "avx2")]` or `"sse4.2"`). The dispatcher must prove
that precondition from the result of `get_runtime_feature`; an `unsafe` block
or a branch comment is not evidence. Treat this mapping as part of the
standard/compiler TCB and reject unknown feature strings or altered function
identities.

## VC order and closure

After review and a separate solver grant, use this order and stop at the first
non-Valid result:

1. A generic three-argument fixture: prove `I(0, A, S)`, a valid tag-1 store
   when `A`, a valid tag-2 store when `S`, and fresh loads constrained by the
   same capability symbols. Keep two reads independent.
2. `detect_runtime_feature`: prove its actual priority-ordered body returns
   `1`, `2`, or `3`; `1 ==> A` and `2 ==> S` follow only from the two exact
   detector TCB clauses.
3. `RUNTIME_FEATURE` initializer and `get_runtime_feature`: prove the cache
   invariant on every writer, prove no path returns tag `0` or an unsupported
   tag, and prove `result == 1 ==> A` and `result == 2 ==> S` on cached and
   detection paths.
4. Both `match_uri_vectored` and `match_header_value_vectored`: prove the four
   AVX2/SSE4.2 call preconditions from the result facts. Keep the SWAR fallback
   and header-name delegation in the coverage inventory.
5. Run the required-site index over the initializer, invariant body and its
   checked dependencies, detector body/calls, every load/store, `get` body,
   both dispatchers, and target-feature calls. A selected-caller proof is not
   closure if any registered body or site was skipped.

The runtime's detector and cache bodies must be translated from the actual
source. A contract on `detect_runtime_feature` or `get_runtime_feature` that
states the final supported result would turn the code under review into a
trusted assumption and is not acceptable.

## Fail-closed controls and limits

The Gate C audit should reject at least: tag `4`; tag `1` with the wrong
capability identity; a same-name fake detector; wrong generic feature
constant; target-feature-enabled builds that bypass a detector call; a
normal/proof detector-call or cfg mismatch; a skipped writer or detector; and
untranslated invariant dependencies. Wrong initial/store obligations should
be generated and classified by an authorized solver run. `latest-read` and
`equal-loads` remain abstraction limits, not alleged Rust counterexamples.

The new trusted boundary is only the exact detector-true-to-usable-capability
implication, persistent process capability, and target-feature call
precondition. Existing Gate B atomic history semantics remain their own TCB.
No claim follows about Rust's full memory model, the CPUID implementation, OS
feature discovery body, AVX2/SSE4.2 scanner correctness, byte/cursor refinement,
or the crate outside the required dispatch closure. Those obligations remain
in the separately owned backend runtime plan.
