# Step 2: Why3 context pruning on the actual generated obligation

Scratch worktree: `/tmp/itoa-mulhi-step2`, detached at published main `ef984af64f91b2ec645d7cd8e12b562415019f11`. Main worktree was not edited. The only source change in scratch is the cfg(creusot) representative caller in `itoa/1.0.18/src/u128_ext.rs` (line 323); runtime code is untouched.

## Baseline generated obligation

Command, from `itoa/1.0.18`:

```sh
timeout 120s ../../tools/creusot-toolpatch/scripts/run-proof.sh cargo creusot --simple-triggers=false prove mulhi_product_split_from_limbs --why3session --no-cache
```

The generated Why3 session records the two helper bodies and caller preconditions valid; the caller product assertion remains unknown for Z3 and times out for CVC5. The final ensures goal is valid. The actual failed leaf is `vc_mulhi_product_split_from_limbs.2.0` (`expl="assertion"`) in the preserved [Why3 session](why3session.xml) and [proof summary](proof.json):

- `verif/itoa_rlib/runtime/u128_ext/mulhi_product_split_from_limbs/why3session.xml`
- `verif/itoa_rlib/runtime/u128_ext/mulhi_product_split_from_limbs.coma`

The session's normal proof shape is 3/4 caller goals discharged, with only the product assertion blocked.

## Session export avenue

Command:

```sh
../../tools/creusot-toolpatch/scripts/run-proof.sh why3 session output \
  -L /tmp/creusot-data/share/why3find/packages/creusot \
  --filter-proved=no --filter-is-leaf=yes \
  --output-dir=/tmp/itoa-step2-session-output \
  verif/itoa_rlib/runtime/u128_ext/mulhi_product_split_from_limbs
```

Why3 exported six solver-input SMT-LIB leaves. The exact failed assertion leaf is preserved byte-for-byte in [`mulhi_product_split_from_limbs-vc-2-0.smt2.gz`](mulhi_product_split_from_limbs-vc-2-0.smt2.gz); decompress it to replay the SMT input. It contains both limb decomposition hypotheses, both helper product cuts, the direct product target, and the generic u128 wrapper axioms. Session export offers the solver task; it did not expose a persistent post-WP context-pruning edit/replay path. `why3 ide` is not a recognized CLI command in this installation, and `why3ide` is not installed.

## CLI transformation avenue

The available sound no-argument transformations were checked with `why3 show transformations`: `remove_unused` and `simplify_formula_and_task`. I selected the source assertion and exported its transformed task:

```sh
../../tools/creusot-toolpatch/scripts/run-proof.sh why3 prove \
  -L /tmp/creusot-data/share/why3find/packages/creusot \
  -F coma -a split_vc -a remove_unused -a simplify_formula_and_task \
  -g /tmp/itoa-mulhi-step2/itoa/1.0.18/src/u128_ext.rs:323@assertion \
  -P z3 -t 10 -m 1024 -o /tmp/itoa-step2-pruned-actual \
  verif/itoa_rlib/runtime/u128_ext/mulhi_product_split_from_limbs.coma \
  -T Coma -G vc_mulhi_product_split_from_limbs
```

The resulting `/tmp/itoa-step2-pruned-actual/mulhi_product_split_from_limbs-Coma-vc_mulhi_product_split_from_limbs.smt2` is byte-for-byte identical to the preserved session-exported `.2.0` leaf above. Thus these transformations did not prune the actual goal's context. The file remains 9,937 bytes and includes the generic quantified u128 axioms.

Running that exact CLI transformation pipeline with `-P z3` and without `-o` attempted the selected actual goal, but Z3 returned `Out of memory (1.81s)` under the required 1024 MiB cap. It did not prove the assertion. Normal `cargo creusot prove` has no persisted transformation/session entry from this experiment, so it continues to produce the baseline 3/4 result.

## Result

No supported persistent replay path was found in Why3 session export or the available normal CLI transforms. The exported actual task, baseline Why3 session, and proof summary are retained beside this report. No toolchain sources, trust assumptions, or runtime arithmetic were modified.
