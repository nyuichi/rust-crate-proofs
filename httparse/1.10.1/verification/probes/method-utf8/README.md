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
ordinary token-error outcomes; and URI UTF-8 handling. The checked-in
`evidence/native-20261005/` logs record those full runs. After qualifying the
shared call as `core::str::from_utf8` so the harness and crate resolve the same
standard-library item, the focused method and URI regressions also passed in
both feature configurations; those logs are in
`evidence/native-qualified-20261005/`.

The fresh proof-tool profile and its source, binary, prelude, Why3, Z3, and
standard-library identities are recorded in
`evidence/tool-rebuild-20261005/`. Translation and Why3 type checking for the
exact included helper and a small outcome-mapping caller passed in
`evidence/translation-20261005T145116Z-12312/`. The helper body proof returned
Valid for all three VCs in
`evidence/proof-20261005T145600Z-method_from_bytes/`; the mapping caller proof
returned Valid for all three VCs in
`evidence/proof-20261005T145626Z-map_method_outcome/`. Both runs used Z3
4.15.3, a 30-second timeout, a 1000 MiB limit, and the shared single-worker
proof lock. Their raw stdout, stderr, commands, exit codes, translated COMAs,
and copied source/tool identities are retained with each result.
The earlier translation attempts in `translation-20261005T144016Z/` and
`translation-20261005T144439Z-11715/` are retained as diagnostics only; neither
is part of the proof result. Their original `SHA256SUMS` preserve the hashes of
the live inputs at translation time. Each translation directory also has a
`SNAPSHOT-SHA256SUMS` manifest over its archived copies and logs, so these
snapshots can be checked without access to the original workspace paths.

This establishes the helper's checked conversion and the harness caller's
mapping contract. The `core::str::from_utf8` contract from the frozen
Creusot standard-library model is trusted; the Rust standard-library body is
not proved here. The mapping caller is a representative harness function, not
the production parser body. Native tests exercise the parser integration, but
the GET/POST/generic scanner paths and the full crate have not been
solver-proved. Full crate verification remains open.
