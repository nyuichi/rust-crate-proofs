# Diagnostic controls

All seven semantic defects include every one of 97 Coma targets. Their
correspondence receipt deliberately says not_run; the separate frozen checker
mutation suite rejects 115/115 structural controls without invoking a solver.
These captures are negative/development evidence, not canonical acceptance.

| Semantic capture | Files / actual prover leaves / nulls | Archive SHA-256 |
| --- | --- | --- |
| boxed-negative-missing-free-v1 | 97/506/1 | `b3378a28c20f07be13d63eafc07ba0bd5f444e88b0bcbfbb1214ebb57dce3994` |
| boxed-negative-missing-untag-v1 | 97/527/1 | `faf5018e7e561b020114be456d824c81bb160443073e53f520788ddbdbc98f3b` |
| boxed-negative-omit-drop-v1 | 97/509/1 | `df037ada41d00c8d2ac1a3dd353c340ad2b37f8d249b0229d29bd46ebe4d8a1e` |
| boxed-negative-raw-as-arc-v1 | 97/515/1 | `9915965f9317e0a5030df1df790ddb5f00f4d8f416a568905f7b104b7cb9a193` |
| boxed-negative-raw-as-static-v1 | 97/522/1 | `e68c73a220d913d3d7af697aec192d55c5fb6444877af9a6593271ba9d5136ac` |
| boxed-negative-wrong-pointer-v1 | 97/528/2 | `9f6250a54b50ae359ff001c90d28e30b1dbcacc2f15498c759a5e0d9ce7e740b` |
| boxed-negative-wrong-size-v1 | 97/525/6 | `e18b44b02203cb3e741bd005e3f1e8136da97e569fa9536b93bbf67d1e037722` |

The omitted Drop fails the caller gate; its printed null normalizes to a
negated to_vec content-contract condition, rather than a standalone completion
assertion. Treat it as sensitivity/resource-contract rejection. The separate
structural control directly rejects the missing terminal call. Missing untag fails
exact allocation pointer premises; wrong pointer fails the generic deallocation
base match and final metadata; wrong size fails layout/capability capacity
matching. Raw-as-Static fails completion validity for a nonempty allocation.

Missing free and raw-as-ARC reject, but their printed null goals involve
resolution/disposal of the still-owned capabilities (the missing-free task
normalizes through such resolution), rather than a direct standalone theorem
that no receipt was issued. Report these as sensitivity/resource-contract
rejections. Do not treat all null goals as identical effect statements. Printed
first-null tasks accompany the complete archived tasks/results; wrong pointer
has two nulls and wrong size six, rather than artificially one each.

Three type-only controls reject before VC generation with zero Coma files and
no prover: duplicate physical free E0382, duplicate owner Drop E0382, and Drop
before the borrowed read E0505. Their manifests record the exact source-control
selection, reference the complete diagnostic input archive for external pinned
inputs, and exclude stale solver target manifests.

Initial body failures and frontend/tool diagnostics are separately explained in
EXPERIMENT_REVIEW.md. The first successful 97/510/0 diagnostic predates the
finished checker; only the restored no-feature/no-terminal-control final replay
with explicit current-source correspondence is canonical acceptance.
