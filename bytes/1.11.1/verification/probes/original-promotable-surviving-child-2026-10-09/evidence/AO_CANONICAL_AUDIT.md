# AO canonical independent audit

## Result

The archived scoped AO gate independently checks out. The archive SHA-256 is `5cf0ffb4ae25dd3d8cb757677a5a43032e926c0eac4dbe3822a9c482206238b9`; all 968 regular-file members match the member manifest. No duplicate, missing, extra, unsafe-path, or non-file tar entries were found. The verifier receipt selects 125 unique targets; every captured Coma and `proof.json` hash matches, and an independent proof-tree count is 983 prover leaves, zero null leaves, and zero structural leaves. Receipt policy has no exclusions or features, `correspondence_exit_status=0`, and `diagnostic=false`. The archived proof log SHA-256 is `2ad78b7d1706cc795009aa7c074311a6136b344c4e94f43b9bcf21eb5d8cb60e`.

## Independent correspondence replay

I safely extracted the archive and rebuilt the captured repository/probe directory layout from its archived inputs, then replayed only Python correspondence/fixture checkers. No proof translation, Cargo/rustc build, or solver was invoked. Main correspondence returned `pass` (output SHA-256 `ede25b40a241391f867f91111cf0ee79e20f7387fc1829891ade1ec0ac517deb`); native correspondence returned `pass` over 19 captured MIR bodies (18 production, one client; output SHA-256 `5d4bad4c4ce3b6eb42a529ff5fb60a5192c03102e7520ce82c0ec3f6f36ef662`). The main checker reconstructed and passed all 31 negative controls (`fe6752c3859afcfb52aec033ba1064884257d0790218a205acd07802ed52fc35`); the native checker reconstructed and passed all 52 (`07fae113c8efac61b4c95358794d26aae9c5a6a0b63126d83395419d6b391826`). Both suites accepted the positive baseline and rejected every mutation as expected.

## Inputs and toolchain

The captured `reviewed-production-inputs.json` is SHA-256 `e27b9f64634af27cc52497e08e2875228bda50ca36096dceca4d2c4c5f184306`; it pins 63 files (the crate manifest/lock plus 61 source files) at base commit `361c7cd261507ac0a705b3b836f73240070891c6`. Every listed file matched the archive snapshot, current `/workspace/bytes-work/bytes/1.11.1` worktree, and that Git tree. All 110 private Std files also matched the installed `/workspace/bytes-proof-tools/bytes-proof-std`. Six captured activation/configuration files matched the current tool installation. All eight external binary SHA-256 values in the captured installation manifest matched the current binaries. The archive is not toolchain-self-contained.

The four location-bound compiled-input artifacts match both their captured receipt hashes and the corresponding current `/workspace/bytes-proof-tools/targets/bytes` files; no rebuild was done. Captured hashes: fingerprint `4e2778ada2cfb52797a0801457ff2a9a8c50798f8141b377afab79590f69289f`, build output `0cab1e3220b72008729103a994de3b490da5f9fd8e7148efd9f8dd5f9f508344`, root output `2db6dc53f9aeb9dfd3f0fbae320fe27fec5f3effaec62db3a0d382f4fc4734d3`, and generated `public_records.rs` `41eeb72bd7a4e3b59f042311508a436033c1a01e227035beb955bec749a8169b`.

## Frontend controls

Independently checked all five frontend-only archives (689 members each): duplicate original (`1189d207…`), duplicate survivor (`4ea5f9da…`), duplicate scope (`f1d0a373…`), early survivor (`637301a7…`), and swapped adapters (`fcb04904…`). Every member hash matches its receipt; each archive has a captured active source and compiler log, but no Coma, proof JSON, `verif/` output, or `.why3find` cache. Logs contain the expected Rust diagnostics: E0382 for duplicate moves/scope reuse, E0502/E0505 for moving the survivor while borrowed, and E0061 for wrong adapter calls. These are frontend rejection observations only, not proof or runtime results. The archived capture script explicitly excludes proof/cache directories; no cache contents were found in these captures.

## Scope and limits

This gate concerns only the bounded witness: nonempty Box promotion, original normal Drop while a child survives, child read, saved return evaluation, and final child normal Drop, for both parity cases. It excludes CAS losers, concurrency, unwind behavior, and full-crate verification. Toolchain and compiled artifacts are external/location-bound; correspondence results rely on the captured and reviewed compiler/MIR/allocator TCB. A passing scoped gate does not establish complete `bytes` 1.11.1 verification or unrestricted all-domain `From` refinement.
