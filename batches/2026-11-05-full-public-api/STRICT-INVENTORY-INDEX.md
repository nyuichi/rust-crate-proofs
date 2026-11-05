# Strict inventory index and current public results

This index covers all13 final scoped ledger reconciliations. Public reports currently contain413 model contracts with their stated configurations, domains and trust boundaries. Neither a final reconciliation nor a source-identity match establishes full API verification, complete hidden/nightly coverage or all downstream trait instantiations.

| Crate | Strict source inventory | Report / revision | Current mappings |
|---|---|---:|---:|
| ascii 1.1.0 | [strict-four](diagnostics/strict-four-crate-reconciliation/README.md) | [28 / 6](https://proofs.rs/report/28) | 49 |
| base16ct 1.0.0 | [base16](diagnostics/base16ct-strict-reconciliation/README.md) | [31 / 3](https://proofs.rs/report/31) | 3 |
| bit_field 0.10.3 | [strict-four](diagnostics/strict-four-crate-reconciliation/README.md) | [34 / 5](https://proofs.rs/report/34) | 27 |
| byteorder 1.5.0 | [strict-four](diagnostics/strict-four-crate-reconciliation/README.md) | [32 / 5](https://proofs.rs/report/32) | 42 |
| byteorder-lite 0.1.0 | [strict-four](diagnostics/strict-four-crate-reconciliation/README.md) | [33 / 5](https://proofs.rs/report/33) | 64 |
| either 1.19.0 | [either](diagnostics/either-strict-reconciliation/README.md) | [27 / 7](https://proofs.rs/report/27) | 40 |
| endian-type 0.2.0 | [strict-three](diagnostics/strict-three-crate-reconciliation/README.md) | [26 / 2](https://proofs.rs/report/26) | 66 |
| nohash-hasher 0.2.0 | [strict-three](diagnostics/strict-three-crate-reconciliation/README.md) | [23 / 3](https://proofs.rs/report/23) | 14 |
| normalize-line-endings 0.3.0 | [strict-three](diagnostics/strict-three-crate-reconciliation/README.md) | [21 / 5](https://proofs.rs/report/21) | 1 |
| oorandom 11.1.5 | [prng-four](diagnostics/prng-four-crate-reconciliation/RECONCILIATION.md) | [25 / 3](https://proofs.rs/report/25) | 12 |
| rand_pcg 0.10.2 | [prng-four](diagnostics/prng-four-crate-reconciliation/RECONCILIATION.md) | [29 / 3](https://proofs.rs/report/29) | 25 |
| rand_xorshift 0.5.0 | [prng-four](diagnostics/prng-four-crate-reconciliation/RECONCILIATION.md) | [24 / 6](https://proofs.rs/report/24) | 1 |
| rand_xoshiro 0.8.1 | [prng-four](diagnostics/prng-four-crate-reconciliation/RECONCILIATION.md) | [30 / 3](https://proofs.rs/report/30) | 69 |

`STRICT-INVENTORY-INDEX.json` records each packet’s original public commit/path together with the current exact run/source/SARIF/readback hashes. `current-public-reports.json` is the current report ledger; `proof-updates.json` current_* fields and current_proof_mappings identify current guarantees. Older integration prose and frozen source packets remain historical.

## Historical count differences

- prng-four: 658 default function entries minus151 standard derive entries leaves507;107 configured model keys and400 catalogue gap, including109 conditional generic families. Seed512 public data constructor is separate. Serde40 nonstandard derive and20 Deserialize defaults remain obligations. Source/configuration observations and inherited/blanket predicates are distinct; no feature/configuration counts are summed as independent APIs.
- strict-three: Preserved nohash std199 ledger becomes189 candidates(159 ordinary+30 gated),no_std50=42+8; ten RandomState alias projections inapplicable. Normalize86 splits65+15+6 unsatisfied; endian174 splits126+2 gated+46 excluded std derive. Counts are within the preserved inputs, whose commands did not explicitly request hidden items. They are not exhaustive hidden/nightly totals; generic bounds remain conditions. Normalize primary1 is adapted-source, with distinct original-source supplements.
- strict-four: Original checkpoint matched ASCII37/BitField27/byteorder42/Lite64=170 keys, with35 shadowed provided defaults. Current ASCII49 adds twelve mappings beyond the original ASCII37 checkpoint: four typed vector conversions, six typed borrowed views and two Error descriptions; current group total182. Local/generic/inherited/default/structural axes remain separate; Kani-only and callback-composition guarantees are not extra Lean mappings. The private held-four notes in the original snapshot are historical, not current publication state.
- base16: Default/minimal56 callable/default ledger rows divide31 ordinary conditional+10 gated+14 std-derive exclusions+1 shadow; all69 divide42+10+14+3. Base16 inputs explicitly document hidden items. This differs from the preserved three-crate input filter; arbitrary-size codec/Formatter/UTF8/allocation/encoded_len overflow remain outside current claims. Bounded Display/codec Kani are separate.
- either: 2096 original rows retained across four configurations; shadow defaults50/50/38/40; two constructor operations have four aliases; seven serde-on operations remain unproved. Binder IR is diagnostic, not a trait-solver certificate. Owner/impl/trait/method/HRTB scope, foreign Result self and unknown lifetime/Sized/associated/type conditions remain explicit; default-only proof domains are not promoted across configurations.

The packet baseline mapped counts sum to401; the current413 adds ASCII’s nine mappings beyond its original37 checkpoint. The strict-four checkpoint remains170 as originally published, while its current group is179. New4 release evidence is the exact typed remote readback under `runs/ascii/aa2d4efc-f0be-4d5d-8ef0-35fc63de1c2b/`; two From cards have null/empty catalogue safety/signature metadata rather than independently verified rustdoc signatures. Their safe source headers are separately pinned by the reviewed finite source validator.

## Resume boundary

All13 current report updates are complete. `python3 resume.py` is a plan-only no-op for already published rows and prints the current413 total. It does not authorize reusing an old run for a changed proof snapshot. Next3/known49 remain future work: preserve old37, exact Vec4 and borrow5 identities and the immutable41/46 snapshots; only a separately pinned, reviewed known49 snapshot may proceed to a fresh run and publication. Unknown snapshots, malformed/private type keys and generic erasure are rejected.
