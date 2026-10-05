# Derived trait native environment checkpoint

This is a fresh verification checkpoint using an isolated Creusot compiler
with the local derived method environment fix. It uses the real `src/error.rs`,
the same derived enum fixtures as `derived-traits`, and a generic fixture that
checks the fail closed behavior. The three actual derived equality body goals
were proved with this compiler. The other selected positive and negative
targets have not been sent to a solver with this compiler. Solver results in
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

The selected proof targets remain listed in `proof-positive.targets` (21
targets) and `proof-negative.targets` (2 targets). The next solver checkpoint
should evaluate the remaining positive targets from the fresh nativeenv
translation; old results do not transfer. The three body targets already
proved in this checkpoint are:

- `verif/httparse_derived_traits_harness_rlib/impl_PartialEq_for_Error/eq.coma`
- `verif/httparse_derived_traits_harness_rlib/impl_PartialEq_for_SignedI8/eq.coma`
- `verif/httparse_derived_traits_harness_rlib/impl_PartialEq_for_SignedI16/eq.coma`

The `evidence/nativeenv-checkpoint/` directory freezes the fresh positive and
negative COMA trees, exact harness inputs, target lists, translation and type
check logs, the generic fail closed result, and the three equality proof
sessions. Its `SHA256SUMS` manifest verifies those files. `verify.sh` contains
translation, type-only, and fail-closed modes; the bounded proof commands are
recorded in the evidence bundle, not exposed as a general proof mode.

## Equality body proof checkpoint

Each selected `eq.coma` file had exactly one named verification condition.
Z3 4.15.3 proved each as `Valid`, and its saved Why3 session marks the same
named condition `proved="true"`:

| Target | VC | Result | Steps |
|---|---|---:|---:|
| `Error::eq` | `vc_eq_Error` | Valid | 12,328 |
| `SignedI8::eq` | `vc_eq_SignedI8` | Valid | 5,738 |
| `SignedI16::eq` | `vc_eq_SignedI16` | Valid | 5,738 |

The runs used `why3find prove -s -j 1 --no-cache --time 30` through
`run-proof.bash`, with the isolated package and config. There were no cache
hits. The session records contain the definitive selected-goal results.
Each JSONL also contains several `<none>` fast-path entries, including
one-second `Timeout` results; they are not named goals in the corresponding
saved Why3 session. Those auxiliary attempts are retained in the JSONL and
are not counted as failures of the selected target. The target-specific
`proof.json` files and Why3 sessions are copied under
`evidence/nativeenv-checkpoint/eq-proofs/`.

This closes only the three actual `PartialEq::eq` bodies. It does not close
the remaining 18 selected positive target files, any of the two deliberately
false negative claims, Debug's exact rendered text, or the open Unicode byte
examples from the earlier string checkpoint. The negative files in this
checkpoint passed type checking only; no new negative solver result exists.

## Isolated build provenance

- Compiler base: `437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`
- Native environment patch: `b6aca857fd852e64aa52090a356cf7da8cb4015a95670804d760f56691f3420a`
- Isolated build script: `b5ac3458cb374fe0de31d95f56aea31dafd206c2ee0da5f6017caee33727d268`
- Isolated compiler binary: `ac90aa5735365497bd27167db0afa6e984569c0d97d55eeed7c9931e513889b5`
- Isolated generated and installed prelude: `cca0368e966eacff2188dae670d35678fd95e801a2fce5a3154a61d300b2798b`
- Isolated and global Why3 config: `e1124888733158546f027f02926484f591bed63bb5e10f861a6c93a54330ea07`
- Isolated `creusot-std/src/std/num.rs`: `b777d9fb2cc922148df8718b9a175d249de2174fc5d57f22cf24a63af91256d2`
- Isolated `creusot-std/src/std/string.rs`: `e51e9dd373e683c63c424aca31bb4850f0437e1c5c56c325c8590e46670957df`
- Isolated `creusot-std/src/std/convert.rs`: `5a4346a05298dbf71fe16b35718ac57552105426a4e4cc9161176b4bf3ae5348`

The isolated library copy retains the previously reviewed `convert.rs`
contracts. It pins the pre-existing derived-traits `num.rs` snapshot and does
not include the in-progress shared `num.rs` change. The earlier compiler
binary (`55799f44…`) and string-model binary (`a1ea9237…`) were not overwritten.
