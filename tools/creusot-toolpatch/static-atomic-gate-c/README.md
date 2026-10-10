# Gate C: fixed x86 capability parameters for static atomic invariants

Gate C is an isolated compiler and translation fixture layered over frozen Gate
B. It introduces the registered predicate profile
`I(value: u8, avx2: bool, sse42: bool)` and routes the same compiler-owned
capability symbols through the initializer goal, every audited store assertion,
and every fresh nondeterministic load assumption.

The Why3 prelude symbols are the zero-argument Boolean functions
`creusot.prelude.StaticAtomicCaps.avx2_usable` and
`creusot.prelude.StaticAtomicCaps.sse42_usable`. They are uninterpreted and
unconstrained: there is no axiom that either is true. Since these are the same
qualified symbols in every generated module, they model stable process
capabilities without accepting names or values from a Rust caller or audit
manifest. The Gate C profile rejects non-x86_64 targets, explicit
`-C target-cpu`/`-C target-feature` overrides, and baseline target definitions
enabling AVX2 or SSE4.2. It also checks rustc's resolved
`Session::target_features` and merged `ParseSess::config` values, and rejects
AVX2/SSE4.2 if either appears there. Normal and proof reports record these
effective values. The generated invariant is checked against the exact
non-generic signature `fn(u8, bool, bool) -> bool` and a closed total expression
subset. Gate B v2's one-argument profile remains separately selectable.

The fixture deliberately defines a local `avx2_usable` function. Generated
invariant applications still use the fixed qualified capability symbols. The
fixture checks wrong signatures, an unsupported manifest profile, a
caller-supplied capability path, a `+avx2` target-feature override, and manual
`target_feature` cfg overrides passed through both `RUSTFLAGS` and
`CARGO_ENCODED_RUSTFLAGS`. Coverage labels accessor output
`program-body-lowered`; the label records translation only.

## Results and limits

The frozen `evidence/translation-20261005-effective-cfg` bundle records one
positive translation and six expected audit rejections. The follow-up
`evidence/typecheck-20261005-retained-package` bundle preserves the complete
generated Why3 package, its file manifest, the exact positive COMAs, and the
type-only commands and logs. All eight emitted COMA modules type-checked
against that package. Neither bundle ran a solver. These outputs establish
compiler translation, shared capability-symbol identity, effective target
profile checks, and COMA type correctness only.

The runtime detector is not yet linked to these constants. In particular, no
claim is made that the actual `is_x86_feature_detected!` calls imply either
symbol, that cached return values imply AVX2/SSE4.2 support, or that any
`#[target_feature]` call precondition follows. Those require exact resolved
sysroot detector and target-feature identities with the narrow detector-safety
TCB reviewed in the Gate C design note. Gate B atomic-history semantics remain
the existing standard-library TCB, and separate loads remain independent; no
latest-read or equal-load theorem is introduced.

## Reproduction

The source baseline is the pinned upstream commit
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`. `build-compiler.sh` archives that
commit, applies narrowcast, Gate A, frozen Gate B, then Gate C patches in order,
builds `creusot-rustc` with the pinned nightly and offline Cargo dependencies,
and generates the isolated prelude package. The builder copies the complete
generated Why3 package to
`/workspace/proof-tools/targets/httparse-static-atomic-gate-c/generated-prelude-package`
before deleting its temporary source tree. A sorted file manifest is kept next
to that directory as `generated-prelude-package.SHA256`; its SHA-256 is the
package identity recorded by the fixture runner. This is separate from the
active Creusot installation.

After the isolated compiler and the Gate C patch are present, run:

```sh
CREUSOT_STATIC_ATOMIC_GATE_C_RUSTC=/workspace/proof-tools/targets/httparse-static-atomic-gate-c/debug/creusot-rustc \
  tools/creusot-toolpatch/static-atomic-gate-c/run-gate-c-fixtures.sh
```

The script writes each run to a fresh directory under `/tmp`, hashes source,
fixture, compiler, patch, runner, target cfg, generated package, Why3 binary,
configuration, and audit inputs. For the positive case it invokes only
`why3 prove --type-only`, once per emitted `.coma`, with the retained package
directory first on `-L` and the generated module directory second. It does not
select a prover or claim solver results. The exact type-check commands and
logs are part of the retained-package evidence bundle.
