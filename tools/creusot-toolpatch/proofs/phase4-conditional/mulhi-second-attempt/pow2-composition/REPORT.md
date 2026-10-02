# Mulhi product proof with a Power-sum-pruned Z3 fallback

## Result

`mulhi_core` now proves the exact postcondition

```text
result@ == x@ * y@ / 128.pow2()
```

and the crate-visible `mulhi` wrapper proves the same contract from the core contract. The former `trusted` attribute on `mulhi` is removed. No trusted annotations were added; the existing `mulhi_product_congruence` logic lemma and its requires, ensures, and body are retained. The limb implementation is unchanged at runtime: the added calls, assertions, snapshots, and contracts are gated by `cfg(creusot)`.

A new proved program lemma, `mulhi_product_split_from_limbs`, composes the existing one-sided limb-split lemmas and provides the joint product equality needed by `mulhi_core`. Its three VCs close: the two one-sided helper goals with ordinary Z3 and the joint product goal with `Z3-Pow2Pruned`.

## Driver and prover configuration

`tools/creusot-toolpatch/drivers/z3-pow2pruned.drv` imports Why3's stock Z3 driver and removes only `bv.Pow2int.Power_sum`:

```text
import "z3_4_12.drv"

theory bv.Pow2int
  remove prop Power_sum
end
```

The existing Z3 4.15.3 and CVC5 1.3.1 identities remain configured. `why3find.json` adds `z3-pow2pruned@4.15.3` alongside them. `run-proof.sh` copies the driver into its temporary Why3 config directory and registers that copy by absolute path, so Why3 can resolve it without modifying the Why3 data overlay. A fresh wrapper `why3 config list-provers` listed `Z3-Pow2Pruned 4.15.3`.

The driver removes the `Power_sum` premise while leaving each generated goal unchanged. Dropping a premise strengthens the proof obligation; it adds no axiom or trusted assumption.

The pruned and standard Z3 SMT exports for the small joint-product proof are archived as `mulhi-product-split-pruned.smt2.gz` and `mulhi-product-split-standard.smt2.gz`. Removing the single `Power_sum` assertion block from the 9,919-byte standard export makes it byte-identical to the 9,791-byte pruned export, including the final goal. The product goal remains present in both tasks.

## Verification

The full suite was run through the repository wrapper:

```sh
tools/creusot-toolpatch/scripts/run-verify-all.sh
```

It exited 0. Both the default and `--all-features` runs reported 224 libraries and 1,917 VCs, with no failed goals. In each run, `mulhi_core` passed all 74 generated VCs (the `vc_mulhi_core` body splits into 68 leaves; the module total also includes six callee obligations), and `mulhi` passed both VCs. The full stdout/stderr is preserved in `fullsuite.log.gz`.

Native tests on the same proof-only source candidate also passed: `cargo test --offline --locked` passed 11 integration tests and 2 doctests; `cargo test --offline --locked --release --all-features --tests` passed all 11 integration tests.

The focused joint-helper command, also run through the wrapper from `itoa/1.0.18`, was:

```sh
../../tools/creusot-toolpatch/scripts/run-proof.sh \
  cargo creusot --simple-triggers=false prove \
  mulhi_product_split_from_limbs --why3session --no-cache
```

It passed all three VCs, and its `proof.json` records `z3-pow2pruned@4.15.3` on the joint product goal. The generated `mulhi_core` proof session has no open goals, and the `mulhi` wrapper session closes both obligations.

## Preserved proof artifacts

- `mulhi_product_split_from_limbs.coma.gz`, `mulhi_product_split_from_limbs-proof.json`, and `mulhi_product_split_from_limbs-why3session.xml`
- `mulhi_core.coma.gz`, `mulhi_core-proof.json`, and `mulhi_core-why3session.xml`
- `mulhi.coma.gz`, `mulhi-proof.json`, and `mulhi-why3session.xml`
- The standard and pruned SMT exports used for the exact `Power_sum`-only comparison
- `fullsuite.log.gz`, containing both full-suite configurations

The wrapper's tool versions were Why3 `1.8.2+git`, why3find `v1.2.0+dev` (the available opam/build tree is named `why3find.1.3.0`), Z3 `4.15.3`, and CVC5 `1.3.1`.
