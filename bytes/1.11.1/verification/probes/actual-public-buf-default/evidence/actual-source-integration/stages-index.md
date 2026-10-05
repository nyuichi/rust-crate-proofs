# Actual-source Buf integration stages

The complete package sources are captured per stage. Stage 00 uses source from checkpoint `3ee5b240a5d82b7a263666851766ca9b843fe9c7`, with `src/bytes.rs` and `src/bytes_mut.rs` replaced by the byte-identical-at-capture versions from `7e8f3a06a70fbffa65bbc34223e9968085736b6d`; all source hashes are in `stage-00-baseline/source-files.sha256`. No production source was edited.

- `stage-00-baseline`: before public `Buf` laws.
- `stage-01-public-laws`: actual trait unread model/laws/default method contract; only `&[u8]` model supplied. Translation logs eight missing models.
- `stage-02-easy-models`: nontrusted forwarding, Chain, Take and VecDeque models; `remaining` accounts for Chain's saturating sum. Logs reduce missing models to Cursor, Bytes and BytesMut.
- `stage-03-handle-as-slice-candidates`: Cursor, Bytes and BytesMut models through program `AsRef`/`as_slice` candidates. Translation reports logic/program and visibility errors.
- `stage-04-bytesmut-slot-model`: total BytesMut slot projection (unknown maps to zero) removes its own translation errors; actual law VCs remain unproved, with Cursor/Bytes still blocking translation.
- `stage-05-disabled-handle-diagnostic`: scratch-only `cfg(any())` on Cursor and Bytes `Buf` impls; compilation stops at the existing `IntoIter<Bytes>: Iterator` bound requiring `Bytes: Buf`.

Each stage contains a full source snapshot, source transition patch, captured logs/statuses, and hash manifests. The recorded logs were captured from the corresponding scratch attempts; archive assembly did not replay them.
