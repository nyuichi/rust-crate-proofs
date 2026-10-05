# Integrated checked advance evidence

This bundle snapshots the current production `bytes_mut.rs` and the crate source, the source-extraction probe, the exact generated Coma for both `RawTransition` helpers and production `BytesMut::advance_unchecked`, their three proof reports, and focused/native compatibility logs. The selected run reports `Proved (3 files)`.

The source SHA256 and extraction fragment hashes are recorded in `manifest.json`; `SHA256SUMS` covers every other file in this bundle. The proof establishes the field transition and placeholder destructor suppression. It does not claim `BytesMut::drop` or shared-handle transition behavior.
