# AQ v3 diagnostic archive audit

## Result

Archive/member/source/tool/Std identities and target proof receipts are internally consistent. This remains a diagnostic capture, not full correspondence admission: `generated/correspondence.json` records `not_run`, and the policy has `diagnostic: true` with exit status 2.

## Archive and proof receipts

- Archive SHA-256: `d936353824969733639d56f2b1ad75643e8198e07d27a19eaeea3e1a5ecdc6f8` (matches the claimed hash).
- 1172 unique archive members; every member hash matches the receipt, with no missing, extra, duplicate, or changed paths.
- All 139 unique COMA/proof pairs and the exact included target list match their hash receipts; there are no exclusions or features.
- Independently recounted proof tree: **1202 prover leaves, 0 null leaves, 0 structural failures**. The run log says `Proved (139 files)`. This records the proof run only; it does not change the skipped correspondence status.

## Source, native capture, and dependencies

- `generated/active.rs`: `167f08c84980ff5ab80c7db50909a99879294c98ed4bc11ff7453d05c267e42b`
- `src/slice_extension.rs`: `a5abe021099a257dafeb14043835e1e20c36600e8d9aa4268c470e163175eb5f`
- `src/view_pointer.rs`: `11560870cfe83e8d8ace30ebe27c6efa0c618d1368a749f4ace773b89961c6dd`
- The reviewed production manifest binds 63 inputs to base commit `361c7cd261507ac0a705b3b836f73240070891c6`; all input hashes match the archived 63-file inventory.
- Native MIR capture: 25 selected bodies, all hashes match; rustc `rustc 1.98.0-nightly (91fe22da8 2026-06-21)`, cargo `cargo 1.98.0-nightly (a595d0da2 2026-06-20)`.
- Archived private Std: `creusot-std 0.13.0` at git `afd365f7a8ba33a90c67b0809de4e7a097421057`, with 110 members (103 under `src/`); all member hashes match and the lockfile names the same package version.

The tool installation manifest and all eight currently installed binaries matched the captured hashes. The captured manifest itself marks four binaries—`cargo-creusot`, `creusot-rustc`, `why3`, and `why3find`—as differing from its prior manifest; the current binaries match the hashes recorded in this archive. Exact paths, hashes, and config hashes are in the JSON audit.

## Scope

The archive is a successful 139-file proof run, but the full correspondence checker was deliberately not run in this diagnostic snapshot. Do not treat it as an admission or the current final gate. No Cargo build, prover, or source edit was performed for this audit.

See [AQ_V3_DIAGNOSTIC_AUDIT.json](AQ_V3_DIAGNOSTIC_AUDIT.json) for complete member, tool, source, Std, and native-MIR hash details.
