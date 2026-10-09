# AM frontend type-control audit — 2026-10-09

Read-only audit of the four archived Rust frontend controls. Each archive SHA and all 6,764 regular member hashes were verified. Each has unique tar paths, no missing/extra member rows, and no `.coma` or `proof.json` files. Receipts say `frontend_diagnostic_no_proof_claim` and explicitly exclude stale solver evidence. No prover or source mutation was performed.

| Control | Archive SHA-256 | Diagnostic | Exact shadow mutation |
|---|---|---|---|
| `swap_places` | `7e31227fa455117ae77c12351a9e2cacf3ae015f0eec567abf817c56193cbb2b` | E0382: borrow of moved `original` | Passes `original` to the child adapter, then reads/uses `original` again. |
| `duplicate_child` | `f70f92afc5d4be32784422fe64df23809c97faff2d9544b8498cb0c0b0867243` | E0382: use of moved `child` | Calls the consuming child adapter twice. |
| `duplicate_root` | `a9b25d97991b8e2a2825fd3ca964b798c783bb33fc7454f85c5c47aadafcc950` | E0382: use of moved `original` | Calls the consuming root adapter twice. |
| `early_root` | `23a8c5cd7d326872016cac651516a905516e09420105ad897c99f9fa299dca7a` | E0505 and E0502 | Calls the consuming/mutating root adapter while a borrow from `read_root` remains live for a later `to_vec()`. |

For each capture, the generated mapping feature agrees with the archived `frontend.log` metadata. Each archive’s 12 `probe/src/*.rs` modules compare against its captured AL first-clone input exactly except `lib.rs`, whose only difference is the promotion-module route to `generated/active.rs` (same as the positive capture). The AL source prefix, both adapter bodies, base `promotion.rs`, and native witness are unchanged from the positive source; only the `promoted_automatic_scope` shadow function is mutated. The helper SHA remains `f7e410dcf3c2900464249a3d02ca04deae18cfaf59054ba1f20a9bdf9848d0f7` in all four. The active/client hashes and full receipts are recorded in `evidence/am-type-controls-audit.json`.

These are expected Rust ownership/borrow-checker rejections. They establish that these malformed shadow clients do not reach proof generation; they do not supply semantic VC failures or native correspondence evidence. The log banner mentions a one-prover budget, but the receipts have no proof targets and the archives contain no proof artifacts; treat the results as frontend-only.
