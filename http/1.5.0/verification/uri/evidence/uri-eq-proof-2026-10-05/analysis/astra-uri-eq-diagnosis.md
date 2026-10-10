# Same-type `Uri` equality proof attempt

This is a failed bounded proof batch, retained for diagnosis. The run used the current source-frozen URI profile, completed with exit code 1, and does not establish the same-type equality contract.

The independent solver-free task dump contains 17 body tasks and 4 refinement tasks. The proof JSON expands `vc_eq_Uri` to 19 terminal leaves: 16 passed and 3 remain open. The refinement target is independently complete at 4/4. Overall, 20 of 23 terminal leaves passed and 3 body leaves remain open.

Astra's read-only task diagnosis found that `arity/tasks/eq-Coma-vc_eq_Uri16.why` contains uninterpreted declarations for `deep_model_Scheme` at line 2382 and `deep_model_Authority` at line 2451, with no defining axioms. The scheme and authority getters expose their Views, while the `Option::ne` obligations compare DeepModels. The two nested split parents are proof JSON children 13 and 14, each with one open child; child 16 is directly open. The archived Why3 OCaml API replay preserves those nested parents and complete task contexts without invoking a solver or reparsing printer output.

The implicated source declarations are `#[logic]` on `DeepModel::deep_model` in `src/uri/scheme.rs:110` and `src/uri/authority.rs:837`, while the corresponding View definitions use `#[logic(open(super))]`. Astra's smallest sound repair proposal is to expose the existing pure DeepModel definitions with `#[logic(open(super))]`. That should make the existing definitions available in the parent URI module without changing runtime behavior or introducing trusted declarations. The source has not been changed in this evidence package; the proposal still needs a fresh bounded proof after coordination.
