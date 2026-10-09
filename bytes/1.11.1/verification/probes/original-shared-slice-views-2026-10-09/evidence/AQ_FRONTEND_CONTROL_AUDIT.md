# AQ frontend control archive audit

Read-only audit of the three immutable frontend captures: `duplicate_selected`, `early_selected`, and `snapshot_extract`. No Cargo build, translation, proof run, or solver was performed during this audit. The captured frontend logs were inspected as artifacts.

## Archive and log integrity

| Control | Archive SHA-256 | Members | Frontend log SHA-256 | Observed error |
|---|---|---:|---|---|
| `duplicate_selected` | `05fbd99e7d5e27cd834ba0689e84780d1eed599bb3c63bc27788f1f872e6b47f` | 812 | `fd0c5127f65708d0e4a65b1aaa1cb7adc16aa929f151e5d66331727e72c15127` | E0382 |
| `early_selected` | `f213b71fb2dd1a098614844e37ebbbcc192a359a55d44429a3dca7b4dd97ca61` | 812 | `7ae1274a4fdeff009949ca39006cfcfef32a11ae88b1db25503051a7cf3fad57` | E0502, E0505 |
| `snapshot_extract` | `31c006e9699ce45d4017a4e0c092731117e3d4848e69c027537740c540875b3e` | 812 | `9b987f3dbea3356c3f32b7ad8769121217c47bb60d2f28f9b6eaa79af164d14b` | E0277 |

Each tar archive has 812 regular-file entries with unique paths. Every member SHA-256 matches its receipt, with no missing or extra entries. In all three cases, the archived `frontend.log` is byte-identical to the scratch log referenced by `AQ_FRONTEND_OUTCOMES.json`; the observed error codes match that receipt. Each archive records `frontend_diagnostic_no_proof_claim`, says proof artifacts were deliberately excluded, and corresponds to an outcome with `proof_targets_generated: false`.

The archives contain no `.coma` files, `proof.json` files, `verif/` or `target/` paths, Why3 caches, or current `probe/generated/proof-targets.json`. They do retain 26 other historical/imported `.log` files as captured input context in addition to `frontend.log`; those files are not the result of these frontend controls and are not treated as proof evidence.

## Captured frontend outcomes

- **`duplicate_selected` — E0382.** The generated formal client inserts a second `bytes_view_terminal_drop(selected, ...)` call immediately after the first. The first call moves `selected`; the next call uses the moved value. `generated/active.rs` SHA-256: `89153ce21ee5dd2bcd7c5326605f48a43ced5a6810206c817c40f6fa30b844d5`.
- **`early_selected` — E0505 and E0502.** The generated client moves `bytes_view_terminal_drop(selected, ...)` before `borrowed.to_vec()`. The compiler reports moving `selected` while it remains borrowed (E0505), and mutably borrowing `detached` while the read call's immutable borrow remains live (E0502). `generated/active.rs` SHA-256: `bd20fa917dd50929849a4ec9308d30525b504af7f9149bb309fc839451d3a5a6`.
- **`snapshot_extract` — E0277.** The generated client adds `_extracted: Ghost<Bytes> = snapshot!(selected).into_ghost()`. The compiler reports that `promotion::Bytes` does not satisfy `creusot_std::ghost::Plain`. `generated/active.rs` SHA-256: `4c38d47a6f97cf4db9e920e4d71b3e32bb7137a32135489d9d9194bb0e78931e`.

These are frontend outcomes, not proof results or behavioral counterexamples.

## `Plain` bound and production `Bytes`

The captures include the exact private `creusot-std` 0.13.0 source and the bytes 1.11.1 production source. The relevant archived source hashes are identical across all three captures:

- `inputs/private-std/src/ghost.rs` SHA-256 `34d7cb99ab21dbcb197806a3d1107d6d409b575cec880937cd12ca52959039f9`: line 266 declares `pub trait Plain: Copy`.
- `inputs/private-std/src/snapshot.rs` SHA-256 `ef1c7f26f4caef6c7608e78472a74c8c8d860a2dab139236ebd68a8f6b174476`: `Snapshot::into_ghost` is at line 111 and its `T: Plain` bound is at line 113.
- `inputs/repository/bytes/1.11.1/src/bytes/bytes_record.rs` SHA-256 `cbf965c9da9d738a34c332ef9fdc09b39df9d21b15656c942a35aed81da1625b`: line 82 declares the production `Bytes` struct, including `data: AtomicPtr<()>` at line 86.
- `inputs/repository/bytes/1.11.1/src/bytes.rs` SHA-256 `95789896965446187ecd5cc7bf52f447435fcdc40f4e19c0c49c5a59b22495fd`: line 25 imports the actual atomic pointer type, and line 729 implements `Drop for Bytes`. No `Copy for Bytes` implementation appears in the 61 captured production Rust files.

The captured compiler diagnostic is consistent with these sources: `Snapshot::into_ghost` requires `Bytes: Plain`, while `Plain` requires `Copy`; production `Bytes` contains an atomic pointer and has a `Drop` implementation. The diagnostic is a type-bound rejection, not evidence of a missing proof or a Bytes correctness failure.
