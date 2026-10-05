# Actual unique growth

This gate mechanically extracts `BytesMut::reserve`,
`reserve_unique_growing`, and a caller from the runtime source. The caller
constructs a Vec-backed handle, advances its view (including past its
initialized length), grows it when the native growth branch applies, preserves
the visible byte values, mutates a nonempty view, and explicitly consumes the
matching allocation authority.

The preserved initial proof has 115 files and no null proof leaves. Its log
records a shell error after proof success, so it is retained as historical
proof-phase evidence rather than a clean runner result. The replay records
include source snapshots and the wrapper's exit status.

| Run | Proof files | Null leaves | Runner exit | Result |
| --- | ---: | ---: | ---: | --- |
| `positive-replay` | 115 | 0 | 0 | Clean proof success |
| `negative-stale` | 116 | 2 | 1 | Stale descriptor rejected, 14/16 caller goals |
| `negative-partial` | 116 | 1 | 1 | Partial region rejected, 18/19 caller goals |

All three replay snapshots match the recorded source hashes and extracted
fragments, with no source changes during their runs. The stale descriptor
fails allocation-namespace and capacity matching; the partial region fails
full-allocation coverage. Their other files are proved. `SHA256SUMS` covers
the gate and retained evidence files; verify it with `sha256sum -c SHA256SUMS`.

## Reproduction and evidence

From this directory, with the bytes proof toolchain installed:

```sh
bash capture-evidence.bash positive-replay
bash capture-evidence.bash negative-stale --features negative_stale_growth_descriptor
bash capture-evidence.bash negative-partial --features negative_partial_growth
```

Choose a new run name to rerun without overwriting preserved evidence. The
negative commands intentionally exit with status 1 after successful
translation and a failed proof obligation. They must not be executed as native
tests, because their calls violate the checked physical-access preconditions.

`capture-evidence.bash` holds the shared proof lock until the generated Coma,
proof trees, extraction, and source snapshots have been copied. It invokes the
standard bytes wrapper with a separate inner lock, preserving its one-prover
and feature checks. The wrapper text is read once before execution so an edit
cannot alter a running shell script.

Each new run records per-file SHA-256 hashes, before/after source hashes,
FNV-1a extraction hashes, proof-file/null-leaf counts, and exit status.
`archive-evidence.py evidence/RUN` compresses source and proof trees into
`artifacts.tar.gz`, verifies every member against the original bytes, then
removes the expanded copies. `archive.json` records the archive SHA-256 and
every member's SHA-256; `sha256.json` retains the original expanded file map.
Logs, summaries, and source/extraction hash records remain outside the archive.
To rebuild a saved run in a separate crate directory, place its archived
`source/runtime-src` at `src`, `source/probe` at
`verification/probes/unique-growing-reserve`, and `source/verify-bytes.sh` at
`scripts/verify-bytes.sh`. The probe has its own Cargo workspace and lockfile.
Use the installed bytes toolchain and the saved wrapper to translate and prove.

The growth-specific native test target passes both tests after explicit
registration in the crate's Cargo manifest. A no-default-features build also
passes. `evidence/native-validation` preserves the logs and test/manifest
snapshot; these are ordinary runtime checks, separate from the extracted proof.

## Boundary

B5 `raw_vec::reallocate_bound` is a trusted physical allocator bridge. It
requires Recovery and the entire exclusive capacity region, an offset-zero
base, matching allocation metadata, and **strictly increasing** capacity. It
preserves old slot values and marks added slots Unknown. It does not trust a
BytesMut handle, capacity arithmetic, a sharing protocol, or a final-owner
decision. Its native leaf calls global `alloc`/`realloc` with byte alignment and
the exact old allocation layout.

The proof promises no inequality between old and new logical allocation IDs.
The strict capacity increase makes the old copied BoundPtr descriptor mismatch
the new region even without such a freshness theorem. Native allocator address
reuse or movement is permitted.

The gate excludes in-place reclaim, shared growth/reclaim, unwind, allocation
failure recovery, automatic Drop invocation, and concurrency. The runtime
unique-growth branch calls the same arithmetic helper. cfg proof access uses
the previously reviewed B1/B3/B4 bridges and explicit cleanup.
