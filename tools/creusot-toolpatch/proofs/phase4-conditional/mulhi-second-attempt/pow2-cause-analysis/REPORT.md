# Cause analysis: `Power_sum` and the `mulhi` SMT timeout

This is an SMT-level diagnosis from the clean source snapshot at commit
`133399b`. It changes no Rust source, Why3 task, prover configuration, or trust
annotation. The successful transformed-SMT result is diagnostic evidence, not
a proof of `mulhi_core` or the crate.

## Findings

The relevant imported lemma is Why3 `bv.Pow2int.Power_sum`:

```text
forall n m. n >= 0 /\ m >= 0 -> pow2 (n + m) = pow2 n * pow2 m
```

The original generated product-split task is preserved in the neighboring
[`Step 3 checkpoint`](../step3-why3-pruning/REPORT.md). On that task, with all
22 wrapper premises still present, removing only the SMT assertion named
`Power_sum` leaves the goal and every other assertion unchanged. Z3 reports
`unsat` in 0.025 s with 18.61 MiB maximum memory. The exact derived task, raw
result, and minimal reproduction script are saved beside this report. Since
removing a premise makes the obligation stronger, this diagnostic does not
add assumptions.

The earlier Step 3 post-wrapper-pruning task still returned `out of memory`
(2.569 s). On that same task, removing only `Power_sum` also returned `unsat`
(0.022 s, 18.56 MiB); removing only `Power_s` or `pow2pos` still returned
`out of memory`. These comparison logs are preserved beside the main result.
A capped baseline quantifier profile returned `unknown` after 30,504
resource-limit steps with 354 quantifier instantiations. The archived normalized
[`post-prune-baseline.smt2.gz`](post-prune-baseline.smt2.gz) retains Why3's
assertion order: `Power_s` is the third quantified assertion (line 23),
`Power_sum` the fourth (lines 27–29), and `pow2pos` the fifth (line 31), after
the two representation-range quantifiers. The profile IDs follow that
quantifier order (`k!23`, `k!29`, `k!31`), so `k!29` is the `Power_sum`
quantifier and accounts for 336 instances. This is consistent with a
matching/instantiation explosion around `Power_sum`, but a full trigger trace
was not captured, so the mechanism remains a strong hypothesis rather than a
completed trace.

The pinned Why3 standard library explains why the ordinary `remove_unused`
transform kept this lemma: `stdlib/bv.mlw:20–22` declares `Power_sum` and marks
it with `meta "remove_unused:dependency" lemma Power_sum, function pow2`.
The `mulhi` goal uses `pow2`, so that dependency retains the lemma. The Rust
helper [`power_two_sum`](../../../../../../itoa/1.0.18/src/u128_ext.rs) has an
empty body and a general `pow2` postcondition, so removing `Power_sum` from all
prover tasks could obstruct unrelated VCs.

The saved before/after SMT tasks and logs are solver inputs and diagnostics,
not a normal `cargo creusot prove` run. No Rust-level caller, `mulhi_core`,
high-half postcondition, or crate-integrated proof was established here.

## Reproduce the strongest diagnostic

The exact original generated SMT task is
[`mulhi-product-split-before.smt2.gz`](../step3-why3-pruning/mulhi-product-split-before.smt2.gz).
The saved output after removing only its named `Power_sum` assertion is
[`before_without_Power_sum.smt2.gz`](before_without_Power_sum.smt2.gz), with
the original successful Z3 transcript in [`success.log`](success.log). To
repeat that bounded test from the target crate directory, use the repository
wrapper; the script writes its temporary SMT file under a temporary directory,
prints the prover transcript, and exits unsuccessfully unless the first result
is exactly `unsat`:

```sh
cd itoa/1.0.18
Z3_BIN=/tmp/creusot-data/bin/z3 \
../../tools/creusot-toolpatch/scripts/run-proof.sh \
  python3 ../../tools/creusot-toolpatch/proofs/phase4-conditional/mulhi-second-attempt/pow2-cause-analysis/reproduce_power_sum_removal.py
```

The Z3 path is installation-specific; set `Z3_BIN` to the pinned environment's
binary. The captured post-wrapper-pruning baseline OOM and quantifier profile
are [`post-prune-baseline-oom.log`](post-prune-baseline-oom.log),
[`post-prune-baseline-qi-profile.log`](post-prune-baseline-qi-profile.log),
and the corresponding normalized input
[`post-prune-baseline.smt2.gz`](post-prune-baseline.smt2.gz). `Power_s`-only
and `pow2pos`-only removal logs are included as the two negative comparisons.

## Next proof plan

1. Try an additional Z3 driver/prover variant using Why3's existing
   `theory bv.Pow2int / remove prop Power_sum / end` driver directive. Keep the
   standard Z3 driver available for VCs that need the generic lemma. Check that
   the variant leaves the original goal and other premises unchanged and
   omits only the `Power_sum` premise from the solver input. An all-task driver
   replacement is not the plan.
2. Confirm the new variant is actually selected and its solver-input SMT omits
   only `Power_sum`. Verify the real `why3find`/Cargo wiring (do not assume the
   crate's `why3find.json` `drivers` field configures prover drivers; it is used
   by the `axioms.ml` extraction path). Then prove the representative split
   caller 4/4 with `--no-cache` and a successful replay. The existing
   `prepare-why3-overlay.sh` can provide a scratch Why3 data overlay for
   testing, but this route has not yet been exercised.
3. Apply the variant to the saved 67/68 `mulhi_core` candidate by transplanting
   only its exact ensures and necessary proof suffix into the current source;
   preserve current helpers and implementation improvements. Require all
   assertions and the exact result postcondition to pass before removing the
   existing trusted wrapper equation. A green trailing ensures after an open
   assertion is still a failed core proof. Then run the target crate's
   integrated default and all-features verification and appropriate native
   tests.

Run proof commands serially through `run-proof.sh`, with one prover and the
existing 1024 MiB cap. The above steps are a plan only; none of the driver,
Cargo, caller, or core work has been performed in this checkpoint.
