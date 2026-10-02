# Step 3: exact-goal Why3 context-pruning experiment

This bounded experiment tried a narrow Why3 plugin transformation on the actual
generated caller assertion `vc_mulhi_product_split_from_limbs.2.0`. The result
does not prove that assertion: after the transformation, Z3 returned
`Out of memory (1.80s)` at the standard one-job, 1024 MiB limit. The integrated
`mulhi_core` proof was not attempted.

## Transformation

[`mulhi_prune.ml`](mulhi_prune.ml) registers the no-argument transform
`mulhi_prune_product_leaf`. It makes no change unless all task guards match:

* the exact goal identifier is `vc_mulhi_product_split_from_limbs`;
* the strict hash of the complete post-WP formula is
  `-633486862402357400`; the generated formula is logged in
  [`task-diagnostic.log`](task-diagnostic.log); and
* exactly two sets of the 11 expected wrapper axioms occur as `Paxiom`
  declarations in `creusot/int.coma`, at source lines 958–992 and 1276–1310.

On that exact task the transformation deletes only those 22 `Paxiom`
declarations through Why3's task API. It preserves all other declarations,
including the goal, lemmas, definitions, and cut hypotheses. The before/after
log records 205 → 183 declarations, 22 → 0 selected wrapper axioms, and
`goal_unchanged=true` from `Term.t_equal_strict`. Removing assumptions makes
the VC stronger, so this operation does not add a proof premise. The custom
OCaml transformation itself is not kernel-checked; its small, guarded deletion
logic and the observed task diff are the soundness evidence for this experiment.

The plugin was compiled and loaded by direct `why3 prove` in the scratch
environment, after two `split_vc` transformations. This reached the actual
post-WP product-split leaf. Normal `cargo creusot prove` / `why3find` tactic
selection and replay were **not tested**. The plugin was not integrated into
the repository's runner or Why3 configuration.

## Solver result and artifacts

The unmodified actual task export is
[`mulhi-product-split-before.smt2.gz`](mulhi-product-split-before.smt2.gz)
(9,933 uncompressed bytes). The task after deleting the guarded wrapper axioms
is [`mulhi-product-split-after.smt2.gz`](mulhi-product-split-after.smt2.gz)
(4,992 uncompressed bytes). The direct transformed-task proof log is
[`z3-pruned-task.log`](z3-pruned-task.log); it reports the task diff followed
by Z3 `Out of memory (1.80s)`. The pre-transform goal/context diagnostic is
[`task-diagnostic.log`](task-diagnostic.log).

The diagnostic caller is the same small, untrusted representative caller
preserved under [`../step1-x-split/`](../step1-x-split/). It invokes both
one-sided product-split helpers under their exact split preconditions; the
helper and caller bodies are not integrated by this Step 3 experiment.

## Reproduction notes

Reproduction depends on the pinned Why3 1.8.2 / Creusot installation, its
Why3find Creusot package and generated COMA task, and a native OCaml compiler.
The scratch `why3.conf` contained a machine-local absolute path and is
deliberately not included. To repeat elsewhere, compile the checked-in source
as a native Why3 plugin using the environment's Why3 package:

```sh
ocamlfind ocamlopt -package why3 -c -o mulhi_prune.cmx mulhi_prune.ml
ocamlopt -shared -o mulhi_prune.cmxs mulhi_prune.cmx
```

Do not use `-linkpkg`: the scratch environment loaded Why3's existing modules
through dynlink, and bundling duplicate libraries caused a dynlink error. Make
a local Why3 config whose `[main]` section loads the plugin by its local
extensionless path, then set `WHY3CONFIG` to that local config. The following
command is environment-dependent: adjust the Creusot package path, generated
COMA task, and source path to the local scratch checkout.

```sh
WHY3CONFIG="$LOCAL_WHY3_CONFIG" \
timeout 30s ../../tools/creusot-toolpatch/scripts/run-proof.sh why3 prove \
  -L "$CREUSOT_DATA/share/why3find/packages/creusot" \
  -F coma -a split_vc -a split_vc -a mulhi_prune_product_leaf \
  -g "$SCRATCH/itoa/1.0.18/src/u128_ext.rs:323@assertion" \
  -P z3 -t 15 -m 1024 \
  verif/itoa_rlib/runtime/u128_ext/mulhi_product_split_from_limbs.coma \
  -T Coma -G vc_mulhi_product_split_from_limbs
```

`run-proof.sh` enforces one solver job and a 1024 MiB memory cap. The proof
result above came from this command without `-o`; the separate `-o` invocation
only exported the transformed SMT task and did not start a prover.

## Trust status

No source, runner, or toolchain change was integrated. The existing trusted
`mulhi` equation `result == x * y / 2^128` remains unchanged. This experiment
proves neither the final product substitution nor the high-half result, and
does not justify removing the existing wrapper trust.
