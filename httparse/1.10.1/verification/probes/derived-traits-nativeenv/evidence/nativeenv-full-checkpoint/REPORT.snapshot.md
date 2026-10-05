# Derived trait native environment checkpoint

This is a fresh verification checkpoint using an isolated Creusot compiler
with the local derived method environment fix. It uses the real `src/error.rs`,
the same derived enum fixtures as `derived-traits`, and a generic fixture that
checks the fail closed behavior. All 21 selected positive target files were
proved with this compiler. The two deliberately false negative claims were
type checked but not sent to a solver. Solver results in
`../derived-traits/REPORT.md` belong to the earlier compiler binary and are
not evidence for these new COMA files.

## Compiler change

The compiler is based on Creusot
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`, with the recorded narrowcast,
string model, and derived trait discriminant patches followed by
`httparse-derived-trait-nativeenv.patch`. The added source change is limited to
`creusot/src/ctx.rs`.

When a local method is automatically derived, inherits a trait external
specification, and has no explicit local contract, `TranslationCtx::param_env`
now checks the inherited specification's complete predicate list against the
method's native `tcx.param_env(def_id)`. It uses a fresh inference context in
non-body analysis mode and the existing `evaluate_additional_predicates`
checker. If all predicates resolve in that native environment, the method
body uses that unchanged environment. If any predicate is unsatisfied or
ambiguous, translation stops with an error. The compiler does not silently
drop a required bound.

Other paths retain their existing behavior: external methods still add their
external predicates, and local methods with explicit contracts do not inherit
the trait contract. The inherited method contract remains normalized in the
caller context, as before. At call sites, `translation/function/terminator.rs`
continues to check the external predicates against the caller's current
typing environment before resolving the called trait method. The
termination-graph caller path reads `ctx.param_env(called_id)`; for these
local derived methods, the omitted predicates were already proven from that
method's native environment. No call-site predicate check was removed.

This checkpoint exercises monomorphic `Error`, `SignedI8`, and `SignedI16`
implementations. It does not claim that every generic derived implementation
can inherit a trait specification. A generic `Generic<T>` fixture without a
native `T: DeepModel` bound is rejected with the expected diagnostic. This
demonstrates fail closed behavior, not general generic support.

## Translation and static checks

The positive harness generated 26 COMA files and all 26 passed direct Why3
`--type-only` parsing and type checking. The negative harness generated 14
files, including the two deliberately false selected claims; all 14 passed
the same type-only check. These are syntax and typing results, not solver
results. The generic fixture's translation stopped on the expected native
parameter environment diagnostic.

The fresh `impl_PartialEq_for_Error/eq.coma` contains a seven-arm definition
of `deep_model_Error`, mapping every actual Error variant to its corresponding
model constructor. The fresh `SignedI8` and `SignedI16` equality bodies each
contain their two-arm model definition. Each equality body also retains its
actual discriminant match. The model functions are definitions in the COMA,
not opaque declarations. The earlier `Any` opacity that prevented the equality
body proofs is absent from these three targets.

The selected proof targets are listed in `proof-positive.targets` (21
COMA files) and `proof-negative.targets` (2 files). `proof-nativeenv-eq.targets`
lists the three actual derived equality bodies; `proof-nativeenv-remaining.targets`
lists the other 18 positive files. Old proof results do not transfer to these
fresh COMA files. The three body targets were:

- `verif/httparse_derived_traits_harness_rlib/impl_PartialEq_for_Error/eq.coma`
- `verif/httparse_derived_traits_harness_rlib/impl_PartialEq_for_SignedI8/eq.coma`
- `verif/httparse_derived_traits_harness_rlib/impl_PartialEq_for_SignedI16/eq.coma`

The `evidence/nativeenv-checkpoint/` directory preserves the initial three
equality-body checkpoint. `evidence/nativeenv-full-checkpoint/` freezes the
same fresh COMA trees and inputs, all 21 selected positive results, and the
bounded runner and run identity. Its `SHA256SUMS` manifest verifies the full
bundle. `verify.sh` contains translation, type-only, and fail-closed modes;
the bounded proof commands are recorded in the evidence bundle, not exposed
as a general proof mode.

## Equality body proof checkpoint

Each selected `eq.coma` file had exactly one named verification condition.
Z3 4.15.3 proved each as `Valid`, and its saved Why3 session marks the same
named condition `proved="true"`:

| Target | VC | Result | Steps |
|---|---|---:|---:|
| `Error::eq` | `vc_eq_Error` | Valid | 12,328 |
| `SignedI8::eq` | `vc_eq_SignedI8` | Valid | 5,738 |
| `SignedI16::eq` | `vc_eq_SignedI16` | Valid | 5,738 |

These three equality-body runs used `why3find prove -s -j 1 --no-cache --time
30` through `run-proof.bash`, with the isolated package and config. There were
no cache hits. The session records contain the definitive selected-goal
results.
Each JSONL also contains several `<none>` fast-path entries, including
one-second `Timeout` results; they are not named goals in the corresponding
saved Why3 session. Those auxiliary attempts are retained in the JSONL and
are not counted as failures of the selected target. The target-specific
`proof.json` files and Why3 sessions are copied under
`evidence/nativeenv-checkpoint/eq-proofs/`.

## Remaining selected positive targets

The other 18 selected positive COMA files produced 25 named verification
conditions. All 25 were `Valid` under Z3 4.15.3. Together with the three
derived equality-body goals above, all 28 named conditions in the 21 selected
positive files are `Valid`. Per-target JSON output is preserved as
`evidence/nativeenv-full-checkpoint/remaining-proofs/NN.log`; the run script
stopped on the first non-`Valid` result and has a fixed 30 second per-goal
limit.

The direct runs used the isolated nativeenv binary, package, and Why3 config
through `run-proof.bash`, with one prover and a 1000 MiB limit. Each selected
COMA file was sent to Why3 once with `-P 'Z3,4.15.3' -t 30 -m 1000`. This
avoids Why3Find's automatic time escalation. The 18 files and all 25 named
results are tabulated in
`evidence/nativeenv-full-checkpoint/remaining-proofs/results.tsv`.

The selected Debug implementation body and its derived `fmt__refines`
condition are proved for the `formatter_extends` contract. That contract
states that formatting extends the prior formatter model; these goals do not
establish the exact rendered byte sequence for each Error variant. The two
deliberately false negative claims passed type checking only and were not
solver-run with this compiler. The Unicode byte examples still open in the
separate string checkpoint are also outside this selected batch. This report
therefore records a complete result for these 21 selected positive files, not
a full proof of every crate property.

## Isolated build provenance

- Compiler base: `437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`
- Native environment patch: `b6aca857fd852e64aa52090a356cf7da8cb4015a95670804d760f56691f3420a`
- Build script used for the proved compiler: `b5ac3458cb374fe0de31d95f56aea31dafd206c2ee0da5f6017caee33727d268`
- Current isolated reproduction script with frozen library-seed fallback: `990b58b7cce19efa2ecb81a66980eb3d9890f707972a8d22d0dac24baf786288`
- Original derived-traits build script dependency: `90c3ace4cf900cc2c1c817422ee1bb5271f09fc12fdfb1c40924fd834d7d4477`
- Isolated compiler binary: `ac90aa5735365497bd27167db0afa6e984569c0d97d55eeed7c9931e513889b5`
- Isolated generated and installed prelude: `cca0368e966eacff2188dae670d35678fd95e801a2fce5a3154a61d300b2798b`
- Isolated and global Why3 config: `e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`
- Isolated `creusot-std/src/std/num.rs`: `b777d9fb2cc922148df8718b9a175d249de2174fc5d57f22cf24a63af91256d2`
- Isolated `creusot-std/src/std/string.rs`: `e51e9dd373e683c63c424aca31bb4850f0437e1c5c56c325c8590e46670957df`
- Isolated `creusot-std/src/std/convert.rs`: `5a4346a05298dbf71fe16b35718ac57552105426a4e4cc9161176b4bf3ae5348`

The isolated library copy retains the previously reviewed `convert.rs`
contracts. It pins the pre-existing derived-traits `num.rs` snapshot and does
not include the in-progress shared `num.rs` change. The active compiler has not
been replaced or adopted. The earlier compiler binary (`55799f44…`) and
string-model binary (`a1ea9237…`) were not overwritten. The current
reproduction script uses the original isolated library seed when its three
recorded files match. If that seed is missing or its `num.rs`, `string.rs`, or
`convert.rs` hash differs, the script builds an isolated seed from the repository
libraries and replaces those three files with the frozen copies in
`evidence/nativeenv-full-checkpoint/toolchain/stdlib/`. This preserves the
tested `num.rs` hash even if the shared file changes later. The script update
only changes how that seed is recovered; it was not used to build the already
proved compiler binary.
