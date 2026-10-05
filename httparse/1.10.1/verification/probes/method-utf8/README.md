# Safe-entry UTF-8 boundary for `parse_method`

`_benchable::parse_method` is a safe public function. A caller can advance its
`Bytes` cursor with the safe `Iterator::next` method while leaving the retained
mark unchanged. The GET, POST, and generic token completion paths then used
`slice_skip(1)` from that mark and passed the returned bytes to
`from_utf8_unchecked`. A non-UTF-8 retained prefix could therefore reach
unchecked string construction.

The parser now routes all three completion paths through the shared
`method_from_bytes` helper in `src/parse_method_utf8.rs`. It validates the exact
slice returned by `slice_skip(1)` and returns `None` for invalid UTF-8; callers
map that outcome to `Error::Token`. The existing `slice_skip` call still commits
the cursor before the UTF-8 result is handled. Valid retained prefixes therefore
keep their prior returned span, and invalid prefixes fail safely after the same
commit point.

`parse_uri` was reviewed alongside this change. It already calls checked
`str::from_utf8` on the entire span returned by `slice_skip(1)` and maps invalid
UTF-8 to `Error::Token`, so it needed no source change.

Native default and no-default test runs cover invalid prefixes on GET, POST,
and generic-token paths; retained valid ASCII and UTF-8 prefixes; partial and
ordinary token-error outcomes; and URI UTF-8 handling. Exact commands and raw
logs are in `evidence/native-20261005/`.

The helper contract and runtime boundary are prepared for Creusot proof. Its
body and the parser callers have not yet been solver-proved. Full crate
verification remains open.
