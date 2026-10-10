# Production URI path scanner proof

This leaf harness includes `src/uri/path_scan.rs`, the exact scanner called by
`PathAndQuery::from_shared` and `PathAndQuery::from_static`. It also includes
the production `ErrorKind`, `uri/limits.rs::MAX_LEN`, and nested path/query
byte classifiers. The harness contains no replacement scanner, error type, or
length-limit constant.

The scanner contract describes the precise result: error priority for empty,
overlong, non-single-`*` paths without a valid initial character, and invalid
bytes; the first query and fragment offsets; and whether the retained prefix
contains bytes at or above 128. A fragment stops validation, including the
high-byte scan, so invalid data after `#` does not affect the result.

Run the ordinary source check and focused proof from this directory:

```sh
cargo check --locked --offline
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove \
  scan_path_and_query -- --locked --offline
```

The last targeted elevated run passed five VCs, including the scanner body,
both byte classifiers, and the slice length/empty wrappers. This target proves
the scanner only. `PathAndQuery` construction/accessors, UTF-8 conversion,
`Bytes` operations, and `from_maybe_shared`'s `dyn Any` optimization remain
separate obligations.
