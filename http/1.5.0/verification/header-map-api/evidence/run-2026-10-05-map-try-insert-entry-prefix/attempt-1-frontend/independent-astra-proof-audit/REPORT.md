# Independent try_insert_entry proof reconciliation

Verdict: **ACCEPT for the exact frozen local mutation contract and body.** Four independently printed roots exactly match four successful saved Z3 4.15.3 proof leaves. One is the actual `try_insert_entry` body; three are imported literal-true support. There are no tactic nodes, nested child arities, null leaves, missing roots, or additional proof roots.

This audit runs no solver or frontend and modifies no source or COMA. Reproduce it with `python3 audit.py`. The script saves the complete unfiltered printer stdout/stderr, all four complete individual task files, a byte-exact copy of the proof JSON, and all commands and hashes in `task-audit.json`.

All 177 frozen source hashes match. Frozen Map is `a0543453a9ddcac366e871913dc4b102706e96096a12bcf1c38e3109e1d02191`; frozen Name is `271a078034f379ff60a2777ae1b84ac08fdcf3c080f94eeaa6d2b9f8c2921979`; source-freeze SHA-256 is `6d687a13574b5dbcfc96da3e3c3b0acb32f10164c950d472e4f7c99325480e1b`. The original emitted COMA remains `83b7989409c5a9d8d20c1059e6d281c52cf4c4d54250004bb853d140da1cd976`.

The newly printed full task stream and all four individual full task files are byte-identical to the earlier [source/contract audit](../independent-astra-audit/REPORT.md), without normalization. Its non-vacuity, generic-value, and quantifier-scope findings therefore apply to these exact successful tasks.

| Original root | Classification | Saved proof |
|---|---|---|
| `vc_try_insert_entry_T` | Actual body | Z3 success |
| `vc_len_Bucket_T` | Imported `[@coma:solid] true` | Z3 success |
| `vc_push_Bucket_T` | Imported `[@coma:solid] true` | Z3 success |
| `vc_new` | Imported bare `true` | Z3 success |

The successful direct Why3find invocation uses the original archived COMA through the shared resource wrapper, with `--no-cache` and no preprocessing or Cargo re-emission. Its log reports four successful goals and exit status zero. The preceding attempt exited 127 because its wrapper path was wrong; it is not proof evidence. The successful command/log/status are separately hash-linked.

The actual body proof exports the existing Ok/Err length relation and new last-bucket fields, preserves mask/index table/extra values/Danger, and preserves each old bucket's hash, HeaderName deep model, actual generic `T` value, and link variant/offsets. The prefix postcondition is unconditional and separately scoped. On Err, the unchanged entry length makes that prefix cover every entry. On a zero-entry success, only the prefix quantifier is vacuous; the independent new-element and length clauses remain meaningful.

This is a local append-to-entries proof against genuine Vec::push and other callee contracts. It does not prove Vec::push's implementation from its literal-true support root. Old HeaderName models are preserved; whole runtime-key, complete entries-sequence, and whole-map equality are not exported. The helper does not install the new bucket into the index table, so no insertion reachability, table coverage, key uniqueness, global readiness preservation, Robin Hood ordering, hash coherence, or caller closure is established.

The separate native test run with 35 passing header-map cases compiled a later Name revision (`fa9742...`) alongside Map `a0543453...`. It is runtime validation, not a substitution for the frozen Map `a0543453...` / Name `271a0780...` proof source. This audit binds the latter through the complete archived source and task hashes.
