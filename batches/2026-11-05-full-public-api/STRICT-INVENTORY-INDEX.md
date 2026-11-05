# Strict inventory index and current public results

This index covers all13 final scoped ledger reconciliations. Public reports currently contain423 model contracts with their stated configurations, domains and trust boundaries. Neither a final reconciliation nor a source-identity match establishes full API verification, complete hidden/nightly coverage or all downstream trait instantiations.

| Crate | Strict source inventory | Report / revision | Current mappings |
|---|---|---:|---:|
| ascii 1.1.0 | [strict-four](diagnostics/strict-four-crate-reconciliation/README.md) | [28 / 7](https://proofs.rs/report/28) | 51 |
| base16ct 1.0.0 | [base16](diagnostics/base16ct-strict-reconciliation/README.md) | [31 / 3](https://proofs.rs/report/31) | 3 |
| bit_field 0.10.3 | [strict-four](diagnostics/strict-four-crate-reconciliation/README.md) | [34 / 5](https://proofs.rs/report/34) | 27 |
| byteorder 1.5.0 | [strict-four](diagnostics/strict-four-crate-reconciliation/README.md) | [32 / 5](https://proofs.rs/report/32) | 42 |
| byteorder-lite 0.1.0 | [strict-four](diagnostics/strict-four-crate-reconciliation/README.md) | [33 / 5](https://proofs.rs/report/33) | 64 |
| either 1.19.0 | [either](diagnostics/either-strict-reconciliation/README.md) | [27 / 7](https://proofs.rs/report/27) | 40 |
| endian-type 0.2.0 | [strict-three](diagnostics/strict-three-crate-reconciliation/README.md) | [26 / 2](https://proofs.rs/report/26) | 66 |
| nohash-hasher 0.2.0 | [strict-three](diagnostics/strict-three-crate-reconciliation/README.md) | [23 / 3](https://proofs.rs/report/23) | 14 |
| normalize-line-endings 0.3.0 | [strict-three](diagnostics/strict-three-crate-reconciliation/README.md) | [21 / 5](https://proofs.rs/report/21) | 1 |
| oorandom 11.1.5 | [prng-four](diagnostics/prng-four-crate-reconciliation/RECONCILIATION.md) | [25 / 6](https://proofs.rs/report/25) | 20 |
| rand_pcg 0.10.2 | [prng-four](diagnostics/prng-four-crate-reconciliation/RECONCILIATION.md) | [29 / 5](https://proofs.rs/report/29) | 25 |
| rand_xorshift 0.5.0 | [prng-four](diagnostics/prng-four-crate-reconciliation/RECONCILIATION.md) | [24 / 7](https://proofs.rs/report/24) | 1 |
| rand_xoshiro 0.8.1 | [prng-four](diagnostics/prng-four-crate-reconciliation/RECONCILIATION.md) | [30 / 3](https://proofs.rs/report/30) | 69 |

`STRICT-INVENTORY-INDEX.json` records each packet’s original public commit/path together with the current exact run/source/SARIF/readback hashes. `current-public-reports.json` is the current report ledger; `proof-updates.json` current_* fields and current_proof_mappings identify current guarantees. Older integration prose and frozen source packets remain historical.

## Historical count differences

- prng-four: 658 default function entries minus151 standard derive entries leaves507;Historical107 configured model keys and400 mapping gap included109 conditional generic families. The current two exact range and six conditional generic mappings increase this group to115 machine keys, leaving392 saved catalogue entries without machine mappings; separate Kani and conditional supplements are not evidence-free gaps. Seed512 public data constructor is separate. Serde40 nonstandard derive and20 Deserialize defaults remain obligations. Source/configuration observations and inherited/blanket predicates are distinct; no feature/configuration counts are summed as independent APIs.
- strict-three: Preserved nohash std199 ledger becomes189 candidates(159 ordinary+30 gated),no_std50=42+8; ten RandomState alias projections inapplicable. Normalize86 splits65+15+6 unsatisfied; endian174 splits126+2 gated+46 excluded std derive. Counts are within the preserved inputs, whose commands did not explicitly request hidden items. They are not exhaustive hidden/nightly totals; generic bounds remain conditions. Normalize primary1 is adapted-source, with distinct original-source supplements.
- strict-four: Original checkpoint matched ASCII37/BitField27/byteorder42/Lite64=170 keys, with35 shadowed provided defaults. Current ASCII51 adds fourteen mappings beyond the original ASCII37 checkpoint: four typed vector conversions, six typed borrowed views two Error descriptions and two generic forwarding/description contracts; current group total184. Local/generic/inherited/default/structural axes remain separate; Kani-only and callback-composition guarantees are not extra Lean mappings. The private held-four notes in the original snapshot are historical, not current publication state.
- base16: Default/minimal56 callable/default ledger rows divide31 ordinary conditional+10 gated+14 std-derive exclusions+1 shadow; all69 divide42+10+14+3. Base16 inputs explicitly document hidden items. This differs from the preserved three-crate input filter; arbitrary-size codec/Formatter/UTF8/allocation/encoded_len overflow remain outside current claims. Bounded Display/codec Kani are separate.
- either: 2096 original rows retained across four configurations; shadow defaults50/50/38/40; two constructor operations have four aliases; seven serde-on operations remain unproved. Binder IR is diagnostic, not a trait-solver certificate. Owner/impl/trait/method/HRTB scope, foreign Result self and unknown lifetime/Sized/associated/type conditions remain explicit; default-only proof domains are not promoted across configurations.

The packet baseline mapped counts sum to401. The current423 adds fourteen ASCII mappings beyond its original37 checkpoint and eight Oorandom range/generic mappings beyond its original12 checkpoint. The strict-four group is now184 (historical170), and the PRNG group115 (historical107). Historical typed-vector release evidence is the exact typed remote readback under `runs/ascii/aa2d4efc-f0be-4d5d-8ef0-35fc63de1c2b/`; two From cards have null/empty catalogue safety/signature metadata rather than independently verified rustdoc signatures. Their safe source headers are separately pinned by the reviewed finite source validator.

## Resume boundary

All13 current report updates are complete. `python3 resume.py` is a plan-only no-op for already published rows and prints the current423 total. It does not authorize reusing an old run for a changed proof snapshot. Current ASCII known41/46/49/51 snapshots and their existing identities remain pinned. Future snapshots require separate review, exact source identity, fresh recording and public readback. Oorandom six-family finite production binding is published; unsafe clone arbitrary-destination coverage is incomplete. Unknown snapshots, malformed/private type keys and generic erasure are rejected.


## Oorandom API32 and pointer boundary

[The observed32-row ledger and limits index](diagnostics/oorandom-api32-reconciliation/README.md) separates current primary20 (fourteen local plus six conditional external families) from float2 and external10 supplemental evidence; its original14-row publication checkpoint is preserved as history. Every row having some contract or restricted evidence is not full API completion. [Pointer-model followup](diagnostics/oorandom-clone-pointer-options/RESULTS.md) retains object-bits8 failure and source-only PointerGenerator investigation; arbitrary valid destination sizes/placements/provenance remain unproved. Generic6 finite production binding is complete at revision6; its callbacks/bounds/custom-export trust remain conditional. These docs add zero machine contracts and change no recorded public source/run.


## Remaining arbitrary-destination clone obligations

[Source/IR intrinsic-delegation investigation](diagnostics/oorandom-clone-pointer-options/intrinsic-delegation/PLAN.md) reads already verified fixed-slot GOTO artifacts. Those builds select TrivialClone copy_nonoverlapping(Self,1); this is not a new solver proof or a compiler-correctness proof. Relevant std08/09 body blocks and TrivialClone declaration lines are byte-equal, while whole-file hashes differ. Arbitrary valid regions still require memory/provenance/permission semantics, intrinsic refinement, source/IR/layout correspondence and a noncircular frame/return composition. Compiler-version semantic equivalence is unproved.

[Publication queue](PUBLICATION-QUEUE.md) tracks other12 crates; formal success does not imply source/fresh/publication readiness. Report22 is excluded and heavy fresh runs are single-job.
