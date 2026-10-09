# AP v2 diagnostic archive audit

Read-only inspection of `ap-positive-diagnostic-v2.tar.gz`; no build or prover was run.

- Archive SHA-256: `cd5d0bee4424579a8dc1c1235b6333de95bdbbca52df44945475e55a4ff068ec` (matches receipt).
- The tar contains 1,057 regular-file members. All 1,057 paths and SHA-256 values match the manifest; no duplicate, unsafe, unmanifested, or missing path was found.
- The 130 target Coma files and 130 proof receipts match their recorded hashes. The policy target list equals the complete archived `probe/verif/**/*.coma` set. Independently traversing the proof-tree JSON yields 1,040 prover leaves, zero null leaves, zero empty structural leaves; prover totals are 978 Alt-Ergo, 55 Z3, and 7 CVC5. This matches the capture statistics `130/1040/0/0`.
- Captured source hashes match the frozen anchors: generated active body `cf05c10e…428b`, finite-extension `92998cd2…cd40`, and elaborated native client `f75b0809…d84f`. The archived native receipt selects 20 MIR bodies. The archived native structural-control receipt says baseline pass and 76/76 controls rejected.
- The archive includes the private Std source bundle (110 files), repository/probe dependency inputs (601 files), probe contents (339 files), six tool/config manifest files, and a hashed `run.log`. Tool binaries are not embedded (`self_contained_toolchain=false`).
- `run.log` is SHA-256 `39edd8de…30dfb` and ends `Proved (130 files)`. However, this remains a diagnostic archive: the target policy marks `diagnostic=true`, has correspondence exit status 2, and the archived `generated/correspondence.json` says `not_run`. No independent source-to-proof correspondence/admission claim follows from the proof counts or native controls.

The added `prove_inventory_push` in the archived `src/finite_extension.rs` is an ordinary body with preconditions and proof assertions relating a Snapshot sequence push to the finite inventory model; it is not marked `#[trusted]`. This is a proof-level model lemma. The archive does not establish executable Vec storage/destructor semantics, general concurrent/CAS-loser behavior, unwind coverage, or full-crate From refinement.
