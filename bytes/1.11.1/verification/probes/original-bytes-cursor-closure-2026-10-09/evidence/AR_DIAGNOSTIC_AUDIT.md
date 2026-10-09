# AR v3 diagnostic archive audit

## Result

The v3 archive is internally consistent as diagnostic proof evidence. I checked all 1,297 archive members against the receipt hashes, all 150 target COMA/proof pairs, and recomputed the proof tree totals: 1,328 prover leaves, zero null leaves, and zero structural leaves. The archive SHA-256 is `06860393e6742bdd716426e633ade434b0237ec9963f82a13239e5d4a9566403`.

The archive captured 63 reviewed production inputs (61 source files plus `Cargo.toml` and `Cargo.lock`). Every archived input matches `reviewed-production-inputs.json` for base commit `361c7cd261507ac0a705b3b836f73240070891c6`; the manifest SHA-256 is `e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306`. Its 110-file private `creusot-std` package closure matches the captured receipt and the installed source tree checked read-only. The package metadata identifies `creusot-std 0.13.0`, VCS revision `afd365f7a8ba33a90c67b0809de4e7a097421057`.

## Diagnostic history

- v1 (`ed51e199…`) had 1,373 prover leaves and one null at `promotion/cursor_scope.coma`, goal `vc_cursor_scope`, path `[0,45]`. The archived task sidecar hash is `ed23a335…`; it was independently printed from the archived COMA and matched byte-for-byte. The archived run log warns that the client’s real `slice::is_empty()` call had no contract.
- The immutable v2 frontend capture (`ar-extern-spec-path-frontend-v2.tar.gz`, SHA-256 `1701ae0c…`) contains 882 members whose hashes match its receipt. Its archived `frontend.log` (SHA-256 `f387e0de…`, also matching the earlier scratch log) records `E0433` at `generated/active.rs:1632`: `creusot_std::extern_spec!` was not found. The capture deliberately excludes proof artifacts; it records only this frontend failure.
- v3 uses the qualified `creusot_std::macros::extern_spec!` for the generic slice fact `is_empty() == (self@.len() == 0)`. The archived client retains its native `.is_empty()` call. Its run log has no corresponding missing-contract warning and records `Proved (150 files)`.

The v3 receipt explicitly marks this run `diagnostic=true`; `generated/correspondence.json` says `not_run`, and the target policy records correspondence exit status 2. This audit therefore records a successful diagnostic translation/proof run, not a correspondence pass or canonical admission. The stated scope excludes Root, concurrency, unwind, and full-crate admission.

No Cargo, compiler, prover, or solver was run for this audit. The checks read the immutable archives and receipts, the earlier v1 task sidecar, and the captured private-Std tree.
