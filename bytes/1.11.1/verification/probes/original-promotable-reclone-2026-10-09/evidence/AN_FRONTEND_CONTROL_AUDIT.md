# AN frontend control archive audit

Read-only audit of the five recaptured frontend-control archives. Exact archive digests, member verification, and generated-input hashes are in [AN_FRONTEND_CONTROL_AUDIT.json](AN_FRONTEND_CONTROL_AUDIT.json). No source edits, builds, or prover runs were made for this audit.

## Recaptured v2 controls

Each tarball SHA-256 and its JSON member/hash manifest match the archived files exactly. Each compiler-log feature header hashes the archived `generated/active.rs`, `generated/elaborated-client.rs`, and `generated/reclone-extension.rs`. The logs end in frontend `Compilation failed`; the v2 captures contain no `.why3find`, `.coma`, `.why`, `.vc`, or `.proof` paths.

| Feature | Archive SHA-256 | Members | Rust diagnostic | Why3 cache files | `.coma`/VC/proof paths |
|---|---|---:|---|---:|
| `duplicate_second` | `640c1e0cd7884efa78cd8c66c938e305d52115a8e5fb52aabe4d9e15c6f78c8e` | 630 | E0382 | 0 | 0 |
| `duplicate_first` | `b8f915681dc7c0c8a5fa33efa6ff7dcfe41153fc05d8afb3928b9f7d9464f24d` | 630 | E0382 | 0 | 0 |
| `duplicate_root` | `6b0dbd5b305a356d24b0e8e0a3d3b67b0f4a796a66bc6b950f47d6015afcbf1d` | 630 | E0382 | 0 | 0 |
| `early_root` | `eade82a8d9108c96d40f468f231832dd3c17feffd2953cf9ce6e2bf392a38991` | 630 | E0505, E0502 | 0 | 0 |
| `readonly_reseal` | `3758934047afac27b97c7d0ef5ea5bc125dd93a6d7850a1ea56216911f154120` | 630 | E0507 | 0 | 0 |

Each archived active source differs from its unmutated `generated/positive.rs` only by the intended ownership mutation: an extra second/first/root terminal call; moving root terminal-drop ahead of `borrowed.to_vec()`; or binding read-only state by moving `phase.own` in both clone branches. The three duplicate-call controls produce E0382. `early_root` produces E0505 and E0502. `readonly_reseal` produces E0507. These are concrete Rust frontend ownership rejections, not lifecycle proofs or a stale-history theorem. The hashes for all generated files are recorded in the JSON.

## Immutable v1 and inherited AM capture warning

The five earlier AN v1 tarballs remain unchanged. Each includes 1,578 `probe/.why3find` cached status files (1,121 `Valid`, 381 `Timeout`, 75 `Unknown`, one unparsed status) while its receipt says stale solver evidence was excluded. Those archives contain no `.coma`, `.why`, `.vc`, or `.proof` path. The cached statuses are not attributable to the control compile attempts and were not treated as results; the receipt's exclusion statement did not match the archived contents.

The four checked AM frontend v1 archives have the same inherited issue: 1,045 `probe/.why3find` status files per archive (783 `Valid`, 220 `Timeout`, 41 `Unknown`, one unparsed status), despite the same exclusion wording. Each has no `.coma`, `.why`, `.vc`, or `.proof` path. These cache entries do not represent the archived AM frontend diagnostics.

The AN v2 recaptures omit `.why3find` and close cleanly. All controls remain frontend diagnostics; they do not establish lifecycle correctness or full-target admission.
