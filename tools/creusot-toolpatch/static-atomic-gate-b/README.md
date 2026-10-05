# Static `AtomicU8` invariant Gate B

Gate B is an isolated Creusot compiler patch and fixture for translating a
registered, private immutable `static AtomicU8` through initializer, load, and
store proof obligations. It is a translation checkpoint, not a discharged proof
or a verification of httparse's dispatch implementation.

## Scope and proof meaning

For a registered static and predicate `I(u8)`, Gate B emits:

- one initializer goal `I(initial)` with the initializer represented as a typed
  `UInt8.t` literal;
- one checked assertion `I(value)` at every audited store;
- one fresh nondeterministic `UInt8.t` observation followed by assumption
  `I(observation)` at each audited load.

Loads use `IntermediateStmt::Any`, not a pure `Any.any_l` value. Separate loads
are independent observations. This over-approximates value histories when every
initial value and every store preserves `I`; it does not represent latest-read
or read-to-read relationships. `UInt8.t` is the bounded Why3 carrier
`< range 0 0xFF >` in the installed `creusot.int` model, so a fresh value is
already in the Rust `u8` domain; Gate B adds no numeric range axiom.

The invariant is accepted only for an exact local, non-generic, non-prophetic
logic predicate with no preconditions, no trusted/no-translate/builtin
boundary, and a body in a closed total syntax subset: the predicate argument,
Boolean/integer literals, total Boolean comparisons and connectives, `!`, and
the registered primitive `u8` `View` operation. The `View` call must resolve to
the compiler's exact `View` trait method and its `View for u8` implementation,
with the expected builtin symbol and type signature. A local function cannot
spoof this by copying the builtin string. Unknown calls, partial operators,
external dependencies, and prophetic predicates fail closed.

Normal-cfg and proof-cfg reports carry the crate, target, source, compiler,
patch, runner, fixture, run, and input digests. Gate B compares their access
inventories and executable MIR before it enables any load assumption. Its MIR
comparison normalizes only session-local hygiene context numbers (`(#N)`) in
recognized `span`, `pat_span`, and `opt_match_place` debug fields whose values
have the compiler's printed file/line/column shape. Unknown formats fail
closed. It retains and compares statement/control-flow data, types, resolved
local definition paths, atomic call sites, and literal contents. A dedicated
negative fixture uses cfg-dependent panic strings containing both Span-looking
text and escaped quotes; the normal/proof MIR comparison rejects those literal
differences. Raw reports are preserved in the evidence bundle. The source file
digest remains part of the report identity.

This gate still depends on the compiler's registered-atomic audit and the
standard atomic value-history semantics. It does not prove Rust's or the
compiler's full memory model, instantiate a general static resource/permission,
or mechanically connect proof translation to a particular runtime dispatch
algorithm. In particular, the current predicate interface cannot state
httparse's CPU-feature-dependent cache invariant. Do not describe this fixture
as proof of `httparse::simd::DISPATCH`, runtime refinement, or full crate
verification.

## Required targets and expected results

[`required-targets.tsv`](required-targets.tsv) is the mandatory target manifest.
The runner checks that every scenario is represented and includes its digest in
the normal/proof input fingerprints. It checks generated module content and
coverage; the run also records per-module SHA-256 values.

The positive fixture must enumerate the predicate, initializer, every accessor,
every exact atomic site, and predicate dependency translation in
`static-atomic-coverage.tsv`. `wrong-init` and `wrong-store` must emit their
`I(2)` obligations. The frozen translation result matrix labels them
*unclassified* because no solver had run at that checkpoint; the bounded
follow-up below records their timeouts and keeps them unclassified.
`latest-read` and `equal-loads` produce assertions that the fresh-`I`
abstraction is expected not to prove; neither is presented as a Rust
counterexample. `skipped-writer`, `partial-predicate`, `prophetic-predicate`,
`spoof-predicate`, and `span-literal-mismatch` must fail closed without a Gate B
module or coverage index.

## Reproduce translation checks

The upstream Creusot source pin is
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`. The local source import used while
developing Gate B had commit `e81a5f61311dffaf8c38f970b2460662d6efdc7b`; that
local import commit is not the upstream pin. Rebuilds start from an archive of
the upstream pin and apply the narrowcast patch, Gate A patch, and Gate B patch
in that order. Gate A remains a separate frozen patch. The Gate B delta SHA-256 is
`4f70d3d5bf8e72355b80a98829cb927423ee9e665c02083973b43ca0d5f4d2d8`; the
target-local compiler binary used for the frozen run is
`8ec09ce81c700852971af51476c8e89d33247a0ffd7531ac66974b3e808b0edd`.

```bash
cd /workspace/rust-crate-proofs
tools/creusot-toolpatch/static-atomic-gate-b/build-compiler.sh
tools/creusot-toolpatch/static-atomic-gate-b/run-fixtures.sh
```

The build script reads the pinned commit from
`/workspace/proof-tools/creusot-source` (override with
`CREUSOT_STATIC_ATOMIC_SOURCE_REPO`), archives it into a temporary directory,
checks the three patch hashes, applies them in order, and builds offline. It
does not depend on a scratch source tree. The build requires the pinned source
commit, the exact patches, `nightly-2026-02-27` with `rustc-dev` and `rust-src`,
and the already available offline Cargo dependencies. Set
`CREUSOT_STATIC_ATOMIC_TARGET_DIR` to change its isolated Cargo target directory.

The runner invokes normal `cargo check` and `cargo creusot` translation only; it
does not invoke Why3 or a solver. It expects the target-local compiler binary at
`/workspace/proof-tools/targets/httparse-static-atomic-gate-b/debug/creusot-rustc`.
Set `CREUSOT_STATIC_ATOMIC_RUSTC` and
`CREUSOT_STATIC_ATOMIC_GATE_B_RUN_ROOT` to override the binary or generated
evidence location.

## Frozen translation evidence

The finalized typed-initializer 2026-10-05 run is stored in
[`evidence/final-9b120a5fa27a20f0b78ea63db3d7ad46942d5e85acdc6c8ca936d38f43843403`](evidence/final-9b120a5fa27a20f0b78ea63db3d7ad46942d5e85acdc6c8ca936d38f43843403).
It includes raw normal/proof audit reports, per-case manifests and logs,
translated COMA modules for accepted scenarios, the mandatory coverage indexes,
the exact result matrix, and the COMA SHA-256 table. Verify the bundle with
`sha256sum -c MANIFEST.SHA256` from that directory.

Results: five scenarios translated and their generated obligations/modules
were inspected; five negative audit scenarios were rejected as designed. The
normal/proof literal negative preserves `(#91)` and `(#93)` as distinct string
contents in the raw MIR reports. The initializer goal is
`value_invariant (0: UInt8.t)`; the wrong-initializer case emits
`value_invariant (2: UInt8.t)`. An earlier pre-fix Why3 typecheck rejected an
untyped integer initializer before starting a prover; its raw log and input
hashes are preserved under `prior-attempts/` in the final evidence bundle. The
corrected COMA's solver status is recorded in the follow-up below.


## Solver follow-up (2026-10-05)

The positive COMAs were type-checked and run through Why3 with Z3 4.15.3, one
prover, 30 seconds, and 1000 MiB. The separate, hash-verified run record is in
[`evidence/proof-positive-20261005T053304Z`](evidence/proof-positive-20261005T053304Z).
The initializer goal `I(0)`, `store_one`'s `I(1)` assertion, and `read_cache`'s
body VC were all `Valid`. `read_twice.coma` type-checks and contains two
independent fresh-`I` observations, but Why3 emits no goal for it because the
fixture has no postcondition, assertion, or equality claim. It is translation
and type-check evidence only; no equality or read-to-read property was proved.

The frozen coverage index uses `program-goals-emitted` for each accessor. For
`read_twice`, this label does not mean a Why3 VC was emitted. Treat the row as
program-body translation coverage; a future source refinement should call this
status `program-body-lowered` and report VC generation separately. The frozen
coverage bytes and their hash are retained unchanged.

Bounded negative runs for wrong initialization `I(2)` and an actual body store
of `2` are recorded in
[`evidence/negative-proof-20261005T053940Z`](evidence/negative-proof-20261005T053940Z).
Both timed out at 10 seconds under Z3 4.15.3, one prover, and 1000 MiB. Neither
raw log contains an `Invalid` result, model, or counterexample. Both results
remain unclassified; no negative conclusion or larger-budget claim is made.

These results concern only the Gate B synthetic value-invariant fixture and its
translated obligations. They do not establish httparse dispatch correctness,
Rust's full atomic memory model, or the soundness of the compiler support patch.
