# Rejected diagnostic purity extension

These are frozen evidence files, not a production crate or a proposed patch.
`diagnostic-raw-vec.rs` is an intentionally invalid trusted-interface experiment:
it adds `#[check(ghost)]` to the existing trusted mutable B4 bridge. Do not
copy this annotation into the actual bridge.

With this extension the recorded `erased_write.coma` proves `result == 2`.
The same Rust source run natively returns 1 because the only write was inside
`ghost!` and was erased. Returning to the actual ordinary bridge rejects the
caller during translation. This comparison demonstrates why the extension
must not be used to bypass the DerefMut purity diagnostic.

The selected proof ran in the temporary raw-vec-bridge probe with:

```sh
source /workspace/bytes-proof-tools/activate.sh
flock /tmp/itoa-creusot-proof.lock cargo creusot --only=prove \
  --why3find-arg=-j --why3find-arg=1 erased_write
```

The surrounding `VTABLE_DROP_PROGRESS.md`, source manifest and logs identify
the source snapshot, runtime observation and rejected ordinary-interface
control. Other trusted boundaries retain the original contracts in this
experiment; no proof of their implementations is claimed.
