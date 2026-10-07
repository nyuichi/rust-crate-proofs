# Tokenless sharing distinguishing experiment

Baseline 8bed373b. Plan recorded before source: stock GhostShared is a changed
premise versus frozen mutable ticket splitting through a shared receiver.
Consume an actual Vec with existing B1; irreversibly share its actual sealed
Recovery/PhysicalRegion; copy the immutable witness through &self; prove two B4
reads observe the input bytes. Retain the allocation without a competing Vec
owner or deallocation. No alternative production buffer/API or bytes protocol.

Negative: attempt moving capabilities out of `*shared.to_ref()` to invoke B3.
This tests extraction from a remaining local shared value, NOT proven lastness:
GhostShared is Copy and has no last-handle tracking. Expected Rust rejection.

Stock GhostShared and B1/B4 are existing generic TCB. No project trusted function
is added. Positive is immutable observation only; resource registration,
actual refcount updates, reclaim, Send/Sync and original Bytes Clone are outside
its claim. Source audit covers other existing stock candidates separately.
One interface review for a failed shape; stop rather than changing timeouts.

## Result

Positive immutable-sharing diagnostic proves34 files, including actual_two_reads.
The recover feature is rejected E0507 before Coma/proving. No complete Clone route
was admitted. See [RESULTS_JA.md](RESULTS_JA.md) for the original-code gap, stock
API audit and Astra's rejection of a simple committer-based opening extension.

Replay (proof command requires elevated execution):

```sh
./scripts/verify-bytes.sh tokenless-sharing-2026-10-08
BYTES_TRANSLATE_ONLY=1 ./scripts/verify-bytes.sh tokenless-sharing-2026-10-08 --features recover
python3 verification/probes/tokenless-sharing-2026-10-08/audit.py
```

The second command is expected to fail; no stale positive artifacts count toward
that negative. The macro-limit diagnostic is separately identified in the log.
All active relevant Std source hashes are in audit.json; exact snapshots are in
both archives. No native execution claim is made for this proof-only diagnostic.
