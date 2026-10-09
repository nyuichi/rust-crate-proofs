# AO archived diagnostic and semantic control audit

## Result

The archive, target, and proof-tree checks passed for all eight captures. The positive diagnostic baseline has **125 targets, 983 prover leaves, 0 null leaves, and 0 structural leaves**. The negative controls retain the same 125 selected targets: root recovery publication omitted **2** nulls; original root Drop call omitted **1**; surviving child Drop call omitted **1**; DetachedScope handoff omitted **5**; missing Acquire **2**; missing physical payload free **2**; missing typed control free **2**. The proof-tree leaf counts were independently traversed from the archived `proof.json` files.

All captures are diagnostic, not canonical admission evidence. Each has correspondence exit status 2 and `generated/correspondence.json` status `not_run`. The positive baseline capture has no `generated/compiled-inputs/` entries; the root-recovery negative contains five such files, but its correspondence remains `not_run`.

## Archive checks

| Capture | SHA-256 | Members | Targets | Prover / null / structural | Check |
|---|---|---:|---:|---|---|
| `ao-positive-diagnostic-v1.tar.gz` | `6ac281984cc2b3392eac2bde26cc813abfef8c395559b46b2186072b2f2fcede` | 958 | 125 | 983 / 0 / 0 | all member and target hashes match |
| `ao-negative-root-recovery-publication-v1.tar.gz` | `5135a7cdf2a3649cc6962afd0656503aa48f6934ec9728ff0d3c8405e795681b` | 968 | 125 | 1033 / 2 / 0 | all member and target hashes match |
| `ao-negative-omit-original-v1.tar.gz` | `37481452533767ab626934e3561de41f2ebb3aac4069982fb234d2887d36bb9f` | 968 | 125 | 986 / 1 / 0 | all member and target hashes match |
| `ao-negative-omit-survivor-v1.tar.gz` | `b57dd67de9540a80e11336028224b5c03c477bcff2afbebf001685f05ba7fe97` | 968 | 125 | 996 / 1 / 0 | all member and target hashes match |
| `ao-negative-omit-handoff-v1.tar.gz` | `f177d0e2b12b7dcd3a76c3bf6b8590d34875809441242c1b4d385a11467875e4` | 968 | 125 | 997 / 5 / 0 | all member and target hashes match |
| `ao-negative-negative-missing-acquire-v1.tar.gz` | `8b4af9b6f2a51357e026910407e410e78d3642fab653a32f210d9c613d86f380` | 968 | 125 | 1032 / 2 / 0 | all member and target hashes match |
| `ao-negative-negative-missing-payload-free-v1.tar.gz` | `e277f734c150b534611be06c98ca76d84e37c9d0ced6cd008deb7f3c687937a8` | 968 | 125 | 995 / 2 / 0 | all member and target hashes match |
| `ao-negative-negative-missing-control-free-v1.tar.gz` | `721c184409e87ad58654b04052473959291218cae3dee5deb28cd81baf252304` | 968 | 125 | 999 / 2 / 0 | all member and target hashes match |

For each tarball I compared every regular member against the receipt manifest, verified all 125 selected `.coma` and `proof.json` hashes, and recounted prover/null leaves from every archived proof tree. The archives have no duplicate paths, unlisted/missing members, unsafe paths, or non-file entries. Each target policy selects 125 files, excludes none, and has an empty terminal feature; the last three controls each select one named diagnostic Cargo feature.

## Negative source controls and failed obligations

- **Root recovery publication:** `lifecycle::State<T>::on_release` replaces `state.recovery = Some(sealed)` with `let _ = sealed` in the root ID branch. Two leaves in `on_release.coma` remain null: recovery presence (`not (state'01.current).recovery = None'0`) and `protocol_State_T (state.final)`.
- **Original terminal Drop call:** generated verifier client omits `bytes_root_detaching_terminal_drop(original, ...)`. The remaining unproved goal is the negated root descriptor/phase resolution condition.
- **Survivor terminal Drop call:** generated verifier client omits `bytes_detached_child_terminal_drop(survivor, ...)`. The remaining unproved goal is the saved return Vec equality/content condition.
- **DetachedScope handoff:** generated root-detaching helper omits assigning the detached scope after `release_core`. Five postcondition leaves remain null: completion presence and detached cursor model, public, and observation fields.
- **Missing Acquire:** one Cargo feature disables the Acquire callback in both `promotion::release_core` and retained `public_shared::shared_drop_checked`. Each body has an unproved `acquired_Payload` goal. The two nulls are one feature control observed in two bodies, not separate independent controls.
- **Missing physical payload free:** one Cargo feature substitutes `Ghost::conjure` for the physical projection deallocator in both `promotion::free_recovered` and retained `public_shared::free_recovered`. Both nulls have goal `not inv_Atomic_usize (...ref_cnt)`.
- **Missing typed control free:** one Cargo feature substitutes `Ghost::conjure` for the typed Box free effect in both `free_recovered` bodies. Both nulls are the negated typed free receipt property (namespace, pointer, size, alignment, allocation).

The archived private Std `ghost.rs` has hash `34d7cb99ab21dbcb197806a3d1107d6d409b575cec880937cd12ca52959039f9` and its captured `Ghost::conjure` has `#[requires(false)]`. Thus the two deallocation controls are sensitivity experiments with a deliberately uninhabitable ghost primitive. Their nulls are not standalone theorems that a concrete free receipt exists, and they do not establish Rust execution failures.

The generated verifier source deltas and hashes are recorded in `AO_SEMANTIC_CONTROL_AUDIT.json`. These results show proof sensitivity to the listed changes. A null leaf is not a solver counterexample or proof of an event-absence theorem.

## Printed null-task sidecars

All fifteen null leaves were printed from the exact `.coma` member extracted from its archive. Proof-tree paths begin with `0` for `compute_specified`; the printer applies that transform itself, so each CLI split path omits the initial `0`. Every sidecar retains the exact Coma bytes, proof JSON, stdout/stderr, and a receipt binding archive/member/printer/output hashes under [`ao-semantic-null-tasks/`](ao-semantic-null-tasks/). All fifteen print invocations exited 0. Why3 emitted nonfatal Dynlink plugin warnings (`cfg`, `hypothesis_selection`) and parsing/axiom warnings; no solver was invoked.

## Scope

The positive baseline describes a closed nonempty Box promotion with the original root retiring nonfinally, a detached surviving child read, a saved return, and final normal child Drop, both tag parities. It does not cover CAS losers, concurrency, unwind, or full-crate admission. None of these diagnostic captures passes the correspondence gate or establishes full bytes verification.
