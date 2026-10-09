# AT diagnostic v1 archive audit

- Archive: `evidence/at-full-diagnostic-v1.tar.gz`
- SHA-256: `6c4fe014298a7d89a5e3da6f27ff83da3ab771c69ea57f947805d3cafd898940`
- Archive members: 1,542 regular files; every path and SHA-256 matched the adjacent metadata manifest exactly. All 154 unique COMA/proof pairs matched their recorded hashes.
- Recursive proof-tree leaves: 1,459 prover leaves (1,362 Alt-Ergo, 92 Z3, 5 CVC5), 2 null leaves, 0 structural leaves. The archive metadata reports 154 targets, 1,459 prover leaves, 2 nulls, 0 structural.
- Scope metadata marks this capture diagnostic, with correspondence exit status 2, no selected-feature flags and no exclusions. It does not admit the native source/MIR correspondence or the complete bytes crate.
- The archive's generated mapping is marked ready, binds `native.rs` SHA-256 `9cd33dde8f2995c88417f2baa9faa6deb58991d71f93640ff110c0d091c214e7`, has four normal Drop rows and records assignment bb9/bb10 and saved-return bb16/bb17. The capture receipt binds the same native source. The timing note says authorized mapping-only generation overlapped launch while semantic Rust inputs stayed unchanged. Since correspondence exited 2, mapping readiness is not checker acceptance.

## Null-task replay

Both tasks were printed from the COMA member extracted directly from the archived tar using the archived goal and trail. Printer exited 0 for both tasks. Stdout hashes match `evidence/AT_DIAGNOSTIC_V1_TASKS.json` exactly:

| Goal split | Printed task SHA-256 | Interpretation |
|---|---|---|
| `compute_specified:0,split_vc:19` | `3230ed947343bc995f49ba7c68ce5e52b393a9e6b1480c3de9f4c917b8dc7fd5` | Maintains the detached-scope live-view map as the singleton entry for the current view. |
| `compute_specified:0,split_vc:26` | `56c92d8893e9adca304894744dfec7186aee56b7809e4b8650c5f38211483a40` | After retiring the previous view, the map equals the singleton entry for the fresh clone. |

The printer emitted nonfatal Why3 plugin-load warnings on stderr (`cfg` and `hypothesis_selection` symbol mismatches) and other theory warnings; task stdout matched the recorded hashes and the printer exited successfully. These are proof obligations left unproved in the diagnostic run, not native executions demonstrating a counterexample.
