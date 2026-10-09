# AQ semantic control archive audit — incremental

Read-only audit of the twelve completed immutable AQ semantic-control snapshots. The audit did not run Cargo, Creusot, Why3, or any proof solver. It extracted each archive's recorded COMA task and used `print_null_task` only.

## Archive and target checks

| Control | Archive SHA-256 | Members | Targets | Prover leaves | Null leaves | Structural leaves |
|---|---|---:|---:|---:|---:|---:|
| `wrong_offset` | `62b2259bfe1311ac76570f6dddfedd75f27e15856aae6ae65c2deadd28bf4372` | 1,172 | 139 | 1,253 | 1 | 0 |
| `wrong_length` | `cc181649f879910109a494ac3341d69d7ff86367fea2b13c63b30b6dda12c3f8` | 1,172 | 139 | 1,260 | 3 | 0 |
| `base_read` | `a5d2f49d5df10c5fc02b7452058691a439e7d9d8c8bf58661aed0b78039df7e7` | 1,172 | 139 | 1,237 | 2 | 0 |
| `missing_view_registration` | `fa6b95adcdc0cde597cf58786aead9d631eacbd62f3b2c4ef8e8e94aca230ba7` | 1,172 | 139 | 1,225 | 2 | 0 |
| `view_capacity` | `7656f871b5f1f206ccec740d034ed656deeb087e421380cf9fb9d7a26c7ced74` | 1,172 | 139 | 1,237 | 1 | 0 |
| `empty_register` | `272cfccddde068c1e7042303faf144c79527004a4c033eb027c37ea0670290a7` | 1,172 | 139 | 1,256 | 2 | 0 |
| `omit_first` | `874b19418109538fa177519370c664fe1a46faca6bd6e1d533bd3091bec46322` | 1,172 | 139 | 1,216 | 1 | 0 |
| `omit_owner` | `9c4bfda2d9c5b24d1c305d82efb39ee59501297d6dc7e2323d6657733b96a446` | 1,172 | 139 | 1,220 | 1 | 0 |
| `omit_selected` | `66f6f5d77ac2346c770c11a08c88dcabd913b610d5b958cd2b76c7a8714da731` | 1,172 | 139 | 1,228 | 1 | 0 |
| `negative_missing_acquire` | `f0f6a30f5de52c36cd2cb2d0b8196c90b13ba6570702237cfe4d09daf8370dd9` | 1,172 | 139 | 1,251 | 2 | 0 |
| `negative_missing_payload_free` | `d6b2494ac12b4aec809e07dfea2c5b0a4c85e7b9ac9604222d1a261617ef90bb` | 1,172 | 139 | 1,214 | 2 | 0 |
| `negative_missing_control_free` | `9b6160dd56deab103ca89c7c777b4d6e44450fdad64a3c08ba1f356ed9086b1a` | 1,172 | 139 | 1,218 | 2 | 0 |

For all twelve snapshots, the archive SHA matches its receipt. All 1,172 archive entries are regular files; their paths and SHA-256 values match the member manifest, with no duplicates, omissions, extras, or content mismatches. Each receipt's 139 distinct COMA/proof pairs matches its archived target list and `generated/proof-targets.json`; every target COMA and proof hash matches the bytes in the archive. The proof trees independently recount to the table above, and each listed null path resolves to a null leaf.

Each target list contains 139 entries, with no excluded targets. The run logs end with a failed target: `vc_slice_view`, `vc_read_view`, or `vc_shallow_clone_view_checked`, as detailed below. The archived correspondence receipt is explicitly `not_run`; `proof-targets.json` marks correspondence status 2 and diagnostic mode. These controls therefore record unproved body VCs, not a completed correspondence/admission check.

## Null-task replay and meaning

I extracted each task's COMA from the corresponding archive, then ran the printer with the recorded goal and the path after the leading tree-root `0` (the printer selects `compute_specified` itself). All twenty invocations exited 0. Every printed task is byte-for-byte identical to its durable sidecar under `evidence/aq-semantic-null-tasks/`; the hashes are recorded in the JSON report. Printer stderr contained Why3 plugin-load and ordinary task warnings, but each task was parsed and printed successfully; these are not solver results.

| Control | Mutation in archived generated `active.rs` | Null leaf / exact printed goal |
|---|---|---|
| `wrong_offset` | `view_pointer::add_live(ret.ptr, view_begin, ...)` changed to use offset `0`. | `slice_view.coma`, `vc_slice_view`, `[0,21,1,1,1,0]`: `view_content (T_Bytes'mk (result3.f0'1) result1 (result.data) (result.vtable) (View f0'18 (result3.f1'1))) = (view_content source)[t'int (range.start)..t'int (range.end')]`. |
| `wrong_length` | `ret.len = end - view_begin` changed to `ret.len = end`. | `slice_view.coma`, `vc_slice_view`, `[0,20,1,0,1,0]`: `view_valid (T_Bytes'mk (result2.f0'1) end''03 (result.data) (result.vtable) (View f0'18 (result2.f1'1)))`; `[0,20,1,1,1,0]`: `view_content (T_Bytes'mk (result2.f0'1) end''03 (result.data) (result.vtable) (View f0'18 (result2.f1'1))) = (view_content source)[t'int (range.start)..t'int (range.end')]`; `[0,20,2,0,1,0]`: `t'int end''03 = (t'int (range.end') - t'int (range.start))`. |
| `base_read` | The read body passes `&proof.core.bound` to `physical_projection::borrow` instead of the `bound` argument. | `read_view.coma`, `vc_read_view`, `[0,25,3,1]`: `value'0.ptr = raw_pointer (((result.f0'2).core).bound'1)`; `[0,26,1]`: `view result1 = view_content value'0`. |

The mutations are visible in the archived generated proof source: the `active.rs` SHA-256 values are respectively `36e263469a8f6febc0b3247d462acf9a917cfab84949b9c92b905c9be417dcf7`, `aca511862896d0bbf3da0776235e43d5b32ac47a9d32674e1e9e3167ab02a9ab`, and `854e585cff8d648789eb45cb1ada8353ee541e04df5a24ac690d701381c2c58b`. The base `src/promotion.rs` and `src/slice_extension.rs` snapshots are unchanged across these three archives; the explicit mutation is in the captured generated formal input. The task results show the affected obligations were not proved under that mutation. They do not constitute a native counterexample or establish that production Rust violates the property.

## Next three controls

The next three archives passed the same member, all-target, hash, proof-leaf, and archived-null-path checks. I independently printed all five null tasks from their archived COMA files. Every invocation exited 0 and exactly matched the existing task sidecar.

| Control | Mutation in archived generated `active.rs` | Null leaf / exact printed goal |
|---|---|---|
| `missing_view_registration` | Removes the `State::on_register` call from the ghost callback body, leaving the state and committer unused. | `shallow_clone_view_checked.coma`, `vc_shallow_clone_view_checked`, `[0,12,7,4]`: `shot_store_AtomicUsize (c.final)`; `[0,20]`: negation of the returned view's `BoundPtr`, shared control/physical/payload invariant, lifetime-token, and payload recovery conjunction. |
| `view_capacity` | Constructs the shared view with `capacity: len` instead of `capacity: source.capacity`. | `shallow_clone_view_checked.coma`, `vc_shallow_clone_view_checked`, `[0,26,1,0]`: `view_valid` of the constructed `Bytes`/`View` value. |
| `empty_register` | In the empty-range branch, when `source.len > 0`, calls `clone_shared_view` into a temporary before returning the static empty view. | `slice_view.coma`, `vc_slice_view`, `[0,38,6,3,0,0,1]` and `[0,38,6,3,0,1,1]`: both print `observation (fin_Ghost_refmut_DetachedScope scope') = observation (scope'.current)`. This is the exact unchanged-scope-observation obligation for the empty result against the mutation's extra tracked owner operation; it is not a claim that a native event is absent or a runtime counterexample. |

The `empty_register` control specifically records a proof obligation to preserve the scope observation exactly. Its generated body performs an extra tracked clone in the empty branch; the two null tasks indicate the unchanged-observation obligations were not proved. They do not establish that a native event was omitted, nor that a live owner escapes the branch.


## Third three controls

Each archive records one null leaf in `nested_slice_scope.coma`, with the same all-target and sidecar integrity checks. The captured generated formal client omits one terminal-effect helper call in each control. This does not assert that the corresponding native Rust `Drop` is absent.

| Control | Mutation in generated `active.rs` | Null leaf / goal interpretation |
|---|---|---|
| `omit_first` | Omits `bytes_view_terminal_drop(first, ...)`. | `[0,13]`, `vc_nested_slice_scope`: the printed task is the negation of a conjunction containing the returned nested view's validity/content/length/ownership, scope acceptance/model/public state, and observation postcondition. The exact formula is in the JSON task record. |
| `omit_owner` | Omits `bytes_view_terminal_drop(owner, ...)`. | `[0,17]`, `vc_nested_slice_scope`: the task is the negation of a conjunction about scope model/public state, observation map/count, and the terminal-effect receipt's validity/static/reclaimed fields. See the JSON for the exact formula. |
| `omit_selected` | Omits `bytes_view_terminal_drop(selected, ...)`. | `[0,25]`, `vc_nested_slice_scope`: the task is `not (length (view_Vec_u8_Global result6) = length (view result5) /\ forall i. 0 <= i < length (view result5) -> get (view_Vec_u8_Global result6) i = get (view result5) i)`. This is an unproved negated caller-completion predicate; the null leaf alone does not establish the positive equality or a counterexample. |

The `active.rs` hashes are `3895074b4f540d7c1cd28b41ad1b509f3c5d5c3908fa2da9ca3b58425807aefa` (`omit_first`), `7e1219c489c4c0e49f8b29c03614af7a7b4ca9d034193a9dbedef9238be2933e` (`omit_owner`), and `acaacea4a50842c80a725dad7ccc238020d2c0ac2c4c72640907be2a5b6593b7` (`omit_selected`). The base `src/promotion.rs` and `src/slice_extension.rs` hashes remain unchanged across all twelve audited controls. These remain generated-proof sensitivity results only.

## Final three feature controls

Each of these archives contains one selected Cargo feature and 139 targets. Each feature changes a callback path in both `promotion.rs` and `public_shared.rs`, so the two affected COMA targets are evidence for one feature control across the retained paths, not two independent defects. The generated `active.rs` remains byte-identical to the positive diagnostic snapshot (`167f08c84980ff5ab80c7db50909a99879294c98ed4bc11ff7453d05c267e42b`) for these feature controls.

| Feature | Affected targets and exact printed goals | Interpretation limit |
|---|---|---|
| `negative_missing_acquire` | `release_core.coma`, `[0,26,1,1]`: `acquired_Payload result8 fin6`; `shared_drop_checked.coma`, `[0,29,1,1]`: `acquired_Payload result9 fin7`. | The feature cfg removes the `acquire_owned` / `State::on_acquire` callback block in both release paths. These are two unproved Acquire-state obligations for one shared feature, not separate evidence that the native atomic Acquire is absent. |
| `negative_missing_payload_free` | Both `promotion/free_recovered.coma` and `public_shared/free_recovered.coma`, `[0,11]`: `not inv_Atomic_usize ((val_ptr_Shared (result1.f0'4)).ref_cnt)`. The two printed tasks are byte-identical. | The feature substitutes a conjured `FreeReceipt` for `physical_projection::deallocate`. The exact null goal is the shown atomic invariant formula; it is not a direct native payload-free absence theorem. |
| `negative_missing_control_free` | Both `promotion/free_recovered.coma` and `public_shared/free_recovered.coma`, `[0,12]`: negation of the conjunction requiring the typed receipt's namespace, pointer, size, alignment, and allocated fields. The two printed tasks are byte-identical. | The feature substitutes a conjured `TypedFreeReceipt<Shared>` for `deallocate_typed_box`. This is receipt/body proof sensitivity, not a direct native control-free absence theorem. |

All six task outputs were reprinted from their matching archived COMA files and matched the sidecar SHA-256 values. Each archive records one selected feature in both `target_policy.features` and `proof-targets.json`; correspondence remains `not_run` and diagnostic mode is true. The feature source contains the `Ghost::conjure` substitutions above, so null tasks establish only that the corresponding generated verification obligations did not prove under the feature. They do not provide a native counterexample.

## Reproduction details

The task-only command used `/workspace/work/print_null_task` with `/workspace/bytes-proof-tools/activate.sh` and `WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"`. The archived COMA bytes, goal names, and task paths are listed per row in `AQ_SEMANTIC_CONTROL_AUDIT.json`. No source, archive, or proof input was modified.
