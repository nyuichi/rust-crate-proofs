# `Bytes::truncate` and `Bytes::clear` under a non-promotable vtable precondition

The private `Vtable` now carries an explicit `promotable` field, true only in
the two promotable table initializers. `Bytes::truncate` branches on that
field instead of comparing table addresses. The source-sliced Creusot probe
extracts the exact `Bytes` and `Vtable` layouts plus exact `len`, `truncate`,
and `clear` bodies. It proves the metadata contracts and both method bodies
under the explicit precondition `!self.vtable.promotable`. This entails that
the exact `split_off` branch is unreachable. `truncate` establishes
`min(old_len, requested_len)`; `clear` establishes length zero. The serialized
pinned Creusot run proved 6 files.

The actual source body still contains `drop(self.split_off(len))` for
promotable tables. The probe adapter for that path is a `split_off` method
with `requires(false)` and a panic body; the proof establishes that this
branch is unreachable under the tag precondition. This does not prove that
any constructor supplies a nonpromotable vtable tag. The proof omits
`from_static` and all vtable initializers/callbacks because Creusot rejects
function-pointer table values and static pointer materialization. Their exact
source is retained in snapshots and native builds; table-to-tag consistency
is checked separately by the native test. No `split_off`, ownership, clone,
reference-count, callback, or `Drop` behavior is claimed.

The source snapshots record all six table initializers, including the
`BytesMut` shared table. Native mode compiles the exact static callback bodies
and local panic stand-ins for the unselected promotable callbacks. The probe's
opaque `BytesMut` type exists only as the unobserved callback return type. Its
native test constructs `Bytes::from_static`, truncates and clears metadata,
and forgets the handle; it does not read raw bytes or test destruction. It
passed 1/1. A separate crate unit test checks the new tag against the old
pointer classifier across all six initializer sites and reachable static,
owner, shared, boxed-slice, and `BytesMut` shared paths; it passed 1/1. Earlier
failed pointer-expression attempts remain in `truncate-frontier.tar.gz`.

Run native tests with `cargo test --locked --offline`. Run the serialized pinned
Creusot gate with `bash run.sh`. Any success is bounded to the exact
`truncate`/`clear` bodies and the explicit nonpromotable tag precondition; it
does not establish that arbitrary real handles satisfy that precondition.
