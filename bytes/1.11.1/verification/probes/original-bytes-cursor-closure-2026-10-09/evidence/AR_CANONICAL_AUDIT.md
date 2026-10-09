# AR canonical independent audit

**Result:** selected-gate archive audit passed. **Full Bytes 1.11.1 remains NOT ADMITTED.**

## Archive and proofs

Audited `ar-positive-canonical-v2.tar.gz`, SHA-256 `b55677701700979a2b160d1ad7c10a1fe1db2c9b82655520753ac3d1d8b6ccd7`. Its 1312 unique regular-file members all match the companion receipt. The independently recounted proof tree contains 150 distinct Coma targets and 1328 prover leaves, with zero null and structural leaves. The target list equals all archived `.coma` files; the receipt records no excluded targets, features, source-control mode, or diagnostic mode, and correspondence exit status 0.

The v2 archive includes both required AQ ancestry pins (audit Markdown and canonical receipt JSON), plus the corresponding AP audit Markdown and receipt JSON as optional provenance. I reconstructed the nested AQ/AP probe members in an isolated scratch tree; full checker replay passed without Cargo or a prover. The archived AQ comparison checked 1,178 members against the restored input tree. The source and executable bytes matched; only the two inherited README files differed, consistent with post-capture documentation edits.

Version 1 remains immutable history: its 1,308-member archive omitted the two required AQ ancestry pins `AQ_CANONICAL_AUDIT.md` and `aq-positive-canonical-v1.json`; its replay stopped at the missing Markdown. Version 2 adds those and the corresponding AP audit Markdown and receipt JSON. An independent v1/v2 member diff found only those four additions plus a changed `probe/evidence.py` capture helper; all 150 Coma and proof digests are identical, and every other existing source, generated input, and Cargo artifact member is byte-identical. No proof rerun was needed for this packaging correction.

## Independent replays

- The archived main correspondence checker (`bce69486ae82aa5aa0f01dde80a67b18cb1fce25287e87881ebe618cac78d526`) passed in compiled-capture-only mode with `CARGO_TARGET_DIR` unset. The 45 main in-memory controls rejected 45/45 mutations. Their four absolute Cargo artifact paths were redirected to the captured artifact bytes in memory; the recorded OUT_DIR path was not opened.
- The native source/MIR checker and 76-control suite (`a068195a2d21bd9ec2c250cf1c58ba39d476f3eaad60d02276adafec378f6d8f`; controls script `6253f4e631fea8f0ab8cec3c369311dee3a824736f82cf8817a0f2a6de5528d3`) passed; all 76 mutations were rejected, with no accepted case or checker error. The captured native smoke harness covers 140 execution-only cases. No build or solver ran during this audit.
- The separate `--audit-compiled-capture-only` replay passed. It reconstructed the extracted production records and validated the captured Cargo fingerprint, build output, root-output join, and all four archived artifacts against the archived receipt.

The four Cargo artifact hashes are: `public_records.rs` `41eeb72bd7a4e3b59f042311508a436033c1a01e227035beb955bec749a8169b`, `cargo-run-build-fingerprint.json` `0dc409f082460e12664d4f4412cc68fc6e98837a6af9172cdb39eafd524bd8f6`, `cargo-build-output.txt` `0cab1e3220b72008729103a994de3b490da5f9fd8e7148efd9f8dd5f9f508344`, and `cargo-root-output.txt` `38a07c3a21f6835c3cea2d5679c7eb69bd24107620bdd20a4b9a07a05842db45`. Receipt SHA-256: `56075080ba06e64daeb4ea926c0c739b2cfda523618a56df6137ec2d8129a23a`. The receipt preserves its original absolute `/workspace/bytes-proof-tools/targets/bytes/...` paths. This is a location-bound captured build; the replay did not inspect the live target directory.

## Inputs and proof configuration

The 63-file reviewed production manifest (`e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306`), based on `361c7cd261507ac0a705b3b836f73240070891c6`, matches both the v2 reconstruction and current bytes worktree at `8449794d8115286c26fda1ff115c7c222e62e660`. The 110-file private Std archive matches its pinned source manifest and the current installed Std source tree.

All eight current tool binaries match the captured installation manifest. Four manifest rows (`cargo-creusot`, `creusot-rustc`, `why3`, `why3find`) record `matches_prior_manifest: false`; their current bytes still match the captured AR manifest. These embedded earlier reference hashes are not a separate executable baseline in this archive and do not demonstrate changes from AQ/AM to AR.

The local derived Why3 config matches the recorded main config, SHA-256 `a6843a9dcc90ab55e3da72fe4423ac28ae8ce66f24a0fbd1194db5458e9f6945`. The archived proof budget records one prover, 1,024 MiB, one why3find job and sc-drf disabled. The separate Creusot activation config is also archived and carries the same memory and prover concurrency settings.

## Scope and remaining gap

This gate checks a normal-return Bytes Buf cursor over finite valid advances, its Shared/Static view ownership and selected terminal callbacks. It does not admit all of `bytes` 1.11.1.

The selected client supplies an explicitly nonnull pointer, but two inherited generic contracts still need to export this fact. `borrow_empty` has no actual-pointer nonnull precondition, and an unbound descriptor does not establish nonnull. `view_pointer::wrapping_bounded` permits an unbound, zero-count case and promises a nonnull result without requiring that its actual input pointer be nonnull. `BoundPtr` physically stores `NonNull`, so the latter is an interface/contract gap, not evidence of a reachable null `Bytes` pointer. Both contracts need explicit nonnull preconditions and all callers must be reproved before full admission. Root mutation, other representations/APIs/traits, concurrent ownership, unwind and whole-crate closure remain outside this gate. Generic provenance, erased callbacks, atomic events, private Std and compiler/MIR interpretation remain TCB.
