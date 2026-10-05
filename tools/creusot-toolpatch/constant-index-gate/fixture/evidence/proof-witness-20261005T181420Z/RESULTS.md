# Constant-index gate proof results

This report records the isolated compiler gate and its diagnostic controls. It does not report a parser proof or change the crate's runtime source.

The narrow compiler patch is `tools/creusot-toolpatch/patches/httparse-constant-index.patch`, SHA-256 `1a43b90e8541e1a5ca54ba021be84b35fd5cd31e125db00ca82b2174693496da`. Its lowering routes array reads and writes through the existing bounds-checked `Slice.get`/`Slice.set` path and keeps those operations non-pure for dead-local removal. The gate evidence here exercises only reads; no write-path VC was tested. Other unsupported projection cases remain fail closed.

The arbitrary-input `get_pattern([u8; 4])` COMA is SHA-256 `34aebbaa915b266961916148f1ec9bc0afc6cba3a1191fb53be9eeafcc3b4450`; it is byte-identical to the positive COMA in the frozen proof bundle. That bundle produced 9/9 `Valid` goals, including four ordinary bounds obligations for lanes 0 through 3. The translation uses compiler SHA-256 `49e855f2d3222a7cd294e0b64e75a637207ad08207e97cda01390ce54c8694ac` and prelude SHA-256 `cca0368e966eacff2188dae670d35678fd95e801a2fce5a3154a61d300b2798b`.

The new `get_witness` target calls the proved positive helper with `[71, 69, 84, 32]`. Its COMA SHA-256 is `3b97b139d624b7502fde776a10034ff7d37c5084cf2ef45cee023db5031158aa`. Both split goals are `Valid`: 0.01089 seconds / 18,263 steps and 0.00394 seconds / 11,202 steps. The run used Z3 4.15.3, one prover, a 30-second per-goal limit, and 1000 MiB. The copied effective configuration SHA-256 is `8bcc5dbb29bd1dfb8586f63d00b6592680468dbefe76d7a688101063000fbafe`; `commands.txt`, the raw log, results, input hashes, and configuration are retained beside this report.

The source/CoMa shape checker passed, and its ground QF_UF query SHA-256 `348d66bdb9fab950c2866836917ecac5beea2c7f60d210ea62621c5d7a85b5a5` returned `sat` under Z3 4.15.3. The exact invocation is in `ground-diagnostic/commands.txt`. This is a separate fixed-witness feasibility diagnostic for a true result against a false postcondition. It is not the full arbitrary-input negative VC and is not reported as that VC's counterexample.

The frozen full negative target `reject_false_contract` remains inconclusive: its first four goals were `Valid`, and its final postcondition goal timed out at 30 seconds after 759,220,119 steps. Its raw logs are retained in `../proof-20261005T171205Z-39275/`; it was not rerun. The separate setup attempt at `../proof-witness-20261005T181117Z/` failed before any VC because it used a stale Why3 data path; the corrected successful run used `/workspace/proof-tools/creusot-data/_opam/share/why3`.

Separate local parser preparation has a four-target translation/type-only result and a 178-task dependency inventory. That unpublished preparation is outside this gate bundle. No parser-body proof or parser integration is claimed here.
