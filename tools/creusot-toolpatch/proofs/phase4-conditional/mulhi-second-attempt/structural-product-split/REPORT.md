# Mulhi product-split proof checkpoint

The accepted `mulhi_core` body and exact trusted boundary are unchanged from `e368af11038307bab7725293b4c4784e5cfd1415`. This checkpoint adds one ordinary `#[cfg(creusot)]` proof-only helper, `mulhi_factor_y_split_product`; it is not trusted and does not alter native runtime code.

## Evidence

- The captured unresolved `vc_mulhi_core.54` task is in [mulhi_core_failure_task.txt](mulhi_core_failure_task.txt). It contains the exact `y = y_hi * 2^64 + y_lo` premise and the goal repeats it in the product. The failure is solver/context handling, not a missing premise or a counterexample.
- The standalone helper has one VC, proved by Z3 4.15.3 in 0.007 seconds. The generated Why3 file, session, and proof metadata are in [mulhi_factor_y_split_product.coma](mulhi_factor_y_split_product.coma) and [helper-proof](helper-proof/).
- A larger exact-arithmetic certificate with all limb/carry preconditions left the product-substitution VC open; its captured task is [mulhi_certificate_failure_task.txt](mulhi_certificate_failure_task.txt). Applying the proven four-argument helper at the caller made its postcondition explicit, but the subsequent x-limb substitution still failed. A sparse helper with only the two split equations also left the same x-product congruence goal open.
- A miniature Why3 integer task retaining only its split premise proved with `clear_but` in 0.02 seconds / 49,716 steps. That is a diagnostic of the formula only, not a proof of the generated Creusot VC.
- On the actual generated COMA source, `why3 prove -F coma` parses the file, but the selected assertion does not expose a declaration named `Assert` at the `clear_but` transformation stage (`Symbol 'Assert' not found`). The unpruned selected goal then runs out of memory after 1.88 seconds. This blocks that session-level pruning route; it is not evidence against the theorem.

## Validation

- Focused `mulhi_factor_y_split_product` proof in the main worktree: passed, 1/1 VC (Z3 4.15.3, 0.007 seconds).
- `tools/creusot-toolpatch/scripts/run-verify-all.sh /workspace/rust-crate-proofs`: exit 0. Default and all-features proof runs both passed; `mulhi_core` remained 50/50 operation VCs under the accepted trusted result boundary.
- Native `cargo test --offline`: passed (11 integration tests, 2 doctests).
- Native `cargo test --release --offline`: passed (11 integration tests, 2 doctests).
- Native debug `cargo test --all-features --offline`: failed at link time because `no-panic` detected debug-build panic paths. This matches the existing crate verification notes. The supported release test command, `cargo test --release --all-features --tests --offline --locked`, passed all 11 integration tests.

## Checkpoint scope

The independent helper is retained for follow-up work. Exact `mulhi` closure remains open, so the established trusted boundary stays in place; no runtime code changed and no trust was added. The full-suite success verifies the accepted boundary and the standalone helper, not the desired end-to-end result contract.
