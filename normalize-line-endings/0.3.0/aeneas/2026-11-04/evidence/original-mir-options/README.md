# Unmodified-source extraction investigation (not used by the public report)

Original `src/lib.rs` bytes match `evidence/upstream.crate` (SHA-256
`1ae29ba0393192db102a26bf8db05c15ed05adc7c4474ad1bde05ef2bc937225`).
Tools are pinned to Aeneas `be86ab0f1989d04944292b345b24c435be3ad10a`,
Charon `08d7c9d50144bd84848212c24701253585e90d1f`, Lean 4.35.0.

## Official options

Charon extraction succeeds, but Aeneas rejects the original char switches.
`--mir built`, `promoted`, and `elaborated` all fail with `Unsupported switch
scrutinee`; see `mir-options-results.json` and per-stage logs. Optimized MIR
with Rust's `MatchBranchSimplification` explicitly enabled also fails; see
`*match-branches.log`. That pass merges compatible branch bodies, rather than
lowering arbitrary switches to comparisons. No applicable existing lowering
flag was found in the checked Charon CLI or Aeneas prepasses.

Source inspection of upstream Aeneas `d1e177bb9fdc17fa62e730d524ac905ba19776ed`
found the same unsupported case: `eval_switch` handles Bool and Integer scalar
scrutinees but has no Char case. That newer executable was not run.

## Separate IR experiment

`normalize_char_switch.py` replaces two original-source LLBC Copy-char switches
(three constant cases) with ordered Eq tests and Bool switches. Rust bytes and
the registered Aeneas executable remain unchanged. `replay.bash` reproduces it.
Fresh Bool locals preserve branch bodies and iterator calls.

Aeneas accepts the transformed LLBC with no generated sorry/axiom declarations.
An isolated copy of the existing Proofs.lean checks the resulting model:
`normalized-ir-proof-build.log` records success (1799 jobs), with only
propext, Classical.choice, and Quot.sound in `normalized-ir-axioms.log`.
`hashes.sha256` records input, transformer, and output hashes.

The new IR transformation is unverified and adds a trusted correspondence gap.
This is **not** an unchanged verification pipeline for the original crate.
The theorem covers the transformed model under the existing finite-list
iterator model. Because the IR correspondence is unproved, this experiment is
not adopted in the production pipeline or public report. Compiled artifacts
and caches are ignored.
