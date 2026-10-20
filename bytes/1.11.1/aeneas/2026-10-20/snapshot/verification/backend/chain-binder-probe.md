# Chain trait-field binder probe

Status: **confirmed locally; experimental fix built and tested, not integrated**.

The minimal Rust fixture in [`trait-binder-repro/src/lib.rs`](trait-binder-repro/src/lib.rs) declares a trait method `chain<U>` and calls its default implementation. Charon accepts the fixture and Aeneas at the pinned revision emits an implementation dictionary field equivalent to:

```lean
chain := fun (U : Type) => Chain.chain.default
```

The trait field expects an implicit generic binder, so Lean reports the explicit `Type` lambda as an unexpected first argument. The failure is recorded in [`trait-binder-repro/lean-check.log`](trait-binder-repro/lean-check.log), with exit status `1` in `lean-check.exit`.

An isolated Aeneas source copy changes the binder visibility selected for an implementation method: it reads `explicit_info` from the matching trait declaration method signature rather than from the implementation function binder. The proposed source diff is [`aeneas-trait-binder-candidate.patch`](aeneas-trait-binder-candidate.patch). The copied tool built successfully (`aeneas-chainfield-copy-build.exit` is `0`) and translated the same Rust fixture to `fun {U : Type} => …`; Lean accepts that generated file (`trait-binder-repro/candidate-lean-check.exit` is `0`). This is a source-to-generated-Lean regression, not a hand-edited generated file.

The candidate was also run against the current whole-crate LLBC at `evidence/adapted-whole-final/crate.llbc` (SHA256 `6a14193286bc71ff9f5df90facfa5e5662ed61317cd2bb5b74bc7410e98043c2`), with the same method filtering as the baseline. The generated candidate differs in 17 `chain`/`chain_mut` field binder annotations; implementation bodies are unchanged. For the Lean check, the copied output received only the existing type-only `FloatTypes` import. The whole module still fails: mutable-reference methods have additional backward-state continuation results that do not match the trait dictionary fields, among other downstream extraction/type errors. See `whole-current-candidate-extract.log`, `whole-current-candidate-lean.log`, and the expanded diagnostics in `whole-candidate-maxerrors.log`. This candidate therefore resolves the isolated binder defect but does not make the whole module typecheck.

Do not integrate this patch as written. The probe uses unconditional trait and method lookups (`TraitDeclId.Map.find` and `List.find`) in a shared extraction path. A production change must handle unavailable/filtered trait declarations or methods and be confined to the Lean backend, with appropriate regression coverage. The pinned canonical Aeneas source and binary were left unchanged by this experiment. The candidate source hashes are recorded in `aeneas-printer-source-hashes.txt`; the copied tool commit is in `aeneas-tool-commit.txt`.

## Upstream comparison

The pinned Aeneas revision is `e62dd83d133d9043e7d3628096838b7430f36565` (2026-10-17). The locally inspected `origin/main` is `fc321a28317f458338ce01fe684927eae0fa8f9e` (2026-10-19), one commit later, titled “Fix several unsupported cases in symbolic evaluation (#1457).” That commit changes symbolic evaluation code and does not modify `src/extract/Extract.ml`; the latest inspected extractor still uses the implementation function's binder visibility in this path. This exact chain-field binder mismatch was not found in the issues and pull requests searched; that does not establish that no report exists elsewhere.

Related open reports include Aeneas [#1152](https://github.com/AeneasVerif/aeneas/issues/1152), which has the same kind of trait-field/result-shape incompatibility for an extra mutable backward continuation, and [#1215](https://github.com/AeneasVerif/aeneas/issues/1215), the broader generic-types-with-mutable-references limitation. Aeneas [#961](https://github.com/AeneasVerif/aeneas/issues/961) and [#1046](https://github.com/AeneasVerif/aeneas/issues/1046) describe related trait/implementation or closure-state mismatches, but are not established as the same binder-visibility defect. Fetched states and issue text are in `upstream-2026-10-20/issue-evidence.json`.

## Charon selector probe

On the fixture, `--start-from 'alloc::vec::{_}::set_len'` is parsed and normalized to an impl-pattern selector, then rejected by the resolver because impl patterns are only supported as the first path element. The exact message is in `selector-impl-pattern.log`; it is a selector-resolution limitation, not a lexical parse failure. The alternate `alloc::vec::_::set_len` selector reached rustc but triggered a rustc metadata panic involving `PhantomData`; see `selector-glob-pattern.log`. That alternate result is inconclusive. Charon [#399](https://github.com/AeneasVerif/charon/issues/399) and [#404](https://github.com/AeneasVerif/charon/issues/404) concern related name-matcher limitations, but do not establish that the exact nested-impl path behavior is fixed or covered.
