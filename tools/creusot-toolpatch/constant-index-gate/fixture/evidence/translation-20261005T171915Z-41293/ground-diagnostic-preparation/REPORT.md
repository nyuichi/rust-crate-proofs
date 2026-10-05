# Ground negative diagnostic preparation

Status: PREPARED_NOT_RUN. This is not a solver result and does not replace the frozen full negative VC timeout.

Translation evidence: `/workspace/httparse-cloud-resume/tools/creusot-toolpatch/constant-index-gate/fixture/evidence/translation-20261005T171915Z-41293`

The arbitrary-input `get_pattern` body has constant-index reads 0..3 and pattern bytes 71, 69, 84, 32. Its nine goals were separately Valid in proof run `proof-20261005T171205Z-39275`.
The frozen `reject_false_contract` COMA contains a matching branch assigning true while requiring result=false; its fifth VC timed out after 30 seconds with no counterexample. This status is retained unchanged.
The new `get_witness` and `reject_witness_contract` translations both call only `get_pattern` with the fixed satisfying array, and the four literal lanes match the pattern. Translation and type-only checking passed for this bundle. The SMT file is a ground reduction of the helper-contract consequence; it has not been submitted to Z3 and must not be presented as the original VC's counterexample.

| Artifact | SHA-256 |
| --- | --- |
| `lib.rs` | `9eed52838a5f5bb0ce19e26bef90b878551239c095c3416360bd277c30c3a49c` |
| `get_pattern.coma` | `34aebbaa915b266961916148f1ec9bc0afc6cba3a1191fb53be9eeafcc3b4450` |
| `reject_false_contract.coma` | `4f43050279307e41badaf3855c8fb5a8b66984c532812c3ce3079828adf6551f` |
| `get_witness.coma` | `3b97b139d624b7502fde776a10034ff7d37c5084cf2ef45cee023db5031158aa` |
| `reject_witness_contract.coma` | `f1bdc0073fc8a9621acc0708602af33eae951cd01388dba13e9bcca17039026b` |
| `witness-vs-false-postcondition.smt2` | `348d66bdb9fab950c2866836917ecac5beea2c7f60d210ea62621c5d7a85b5a5` |
