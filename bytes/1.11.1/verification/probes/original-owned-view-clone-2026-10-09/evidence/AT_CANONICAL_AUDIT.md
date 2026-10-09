# AT canonical v2 independent audit

**Result: pass.** The immutable archive `at-positive-canonical-v2.tar.gz` has SHA-256 `958ed73642e1b305b12c6f9269f38de0ee4559e014d83ca58616ff37bc33bc8d`. I verified all 1,544 unique regular members against the receipt, all 155 COMA/proof pairs and per-target hashes, and recomputed the proof tree totals: **155 files, 1,417 prover leaves, 0 null leaves, 0 structural leaves**. The run receipt has no features, no exclusions, no source control, correspondence exit status 0, and diagnostic false. The archived log ends with `Proved (155 files)`.

## Replayed evidence

I reconstructed the checker tree under `/workspace/work/at-canonical-v2-replay` from this archive. The ancestry package snapshots intentionally omit verifier output trees; I restored those bytes solely from the nested immutable AS, AR, AQ, and AP canonical archives embedded in the AT archive, and checked the source-only snapshots against those nested archives. The full correspondence checker passed using captured artifacts only. It reported the selected owned-View Clone scope and `full_original_admitted: false`.

The in-memory checker suite replayed **52/52** expected rejections with no accepts or checker errors. The native source/MIR suite replayed **45/45** expected rejections with no accepts or checker errors. Both regenerated fixtures and receipts match their archived counterparts. The lineage replay includes the pinned AS canonical archive (969a1a0406e10677c1849200284881e043aae9c9bdf29c9b15609783c91a9f74), AR (b55677701700979a2b160d1ad7c10a1fe1db2c9b82655520753ac3d1d8b6ccd7), AQ (41e8e9c164a1110c7f611bb1726f490e111c6b10a77af6b57bfc707ce49ceba5), and AP (a62cfcc22afa2756c56fea1230c3d6845ec3bdf66eaa20bf1779d26da12761d1) inputs.

## Captured production and tool inputs

The archived reviewed-production manifest contains 63 entries: 61 `src/` files plus `Cargo.toml` and `Cargo.lock`. All match the current selected crate bytes. All 110 archived private `creusot-std` files match the current source tree. The captured tool manifest has 8 binary entries and the archive includes 6 execution/configuration files. The external binary payloads are not included in the archive; the separate read-only tool check `e15ec42dfd61dc9ae05b27d98e90847a42ae0d2b4e4ab0de69a0b6eb34ded427` reports all eight installed binaries match the captured manifest.

The four captured Cargo outputs (generated public records, build fingerprint, build output, and Cargo root output) match their receipt hashes and the current recorded output files. They were checked by hash only; no Cargo command ran during this audit. The archived proof budget records one prover, 1,024 MiB, and sc-drf disabled.

## Scope

This result covers the selected closed owned-View Clone client over the published ancestry. It does not admit general `Clone` or `From`, the complete original crate, concurrent or escaping owners, or unwind behavior. The v1 archive remains immutable with its separately documented packaging diagnostic; this pass is for canonical v2.
