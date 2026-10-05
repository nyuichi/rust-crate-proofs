# Source-sliced `Bytes` metadata observers

The positive observer scope,
`BYTES_OBSERVER_SCOPE=observers bash run.sh`, generates the exact `Bytes`
layout, `Vtable` layout, `len`, and `is_empty` bodies, and proves their
field-metadata postconditions. The separate `from_static` and `all` scopes
were attempted but are not proof-positive. With the exact table present,
Creusot rejects callback function-item-to-function-pointer casts. Replacing
the initializer by an immutable uninterpreted external `STATIC_VTABLE`
avoids that error, but translation then rejects `&STATIC_VTABLE` as an
unsupported constant pointer scalar. Omitting `const` from `from_static` in a
proof-only retry produced the same error. The exact initializer remains in
source snapshots and native builds; no table facts are claimed. The `all`
scope, which includes exact `new`, fails at the same `from_static` pointer.
Thus neither `new` nor `from_static` is verified.

No `View` or `DeepModel` is assigned to `Bytes`; no raw pointer is dereferenced,
and no physical access authority, byte-content claim, refcount law, clone, Drop,
AsRef, or Buf behavior is included. The native full-scope tests inspect only
length and emptiness, then forget the handles; they pass 2/2. The native result
does not widen the Creusot proof scope.

Exact static callback and vtable source is recorded in the source snapshot and
compiled in native mode. The callback signature mentions `BytesMut`, so the
isolated probe uses an opaque local return-type stand-in and does not claim the
real `BytesMut` implementation. The attempted constructor proof includes a
narrow generic std `AtomicPtr::new` boundary with only `ensures(true)`, plus an
immutable uninterpreted external static when isolating the table initializer.
Neither boundary proves the failed constructor translation. The pinned core
definition is snapshotted and hashed by the build script. The generic boundary
states no pointer-value relation, permission, memory ordering, or bytes
ownership/refcount protocol. An initial local-path dependency attempt made
Creusot compile the whole bytes crate and stopped at Send/Sync and Deref
frontend requirements; the probe removes that dependency.

The positive evidence archive records the exact two-method proof, full-scope
native test, source snapshot, generated observer-only Rust, COMA, and proof
JSON. It supplies no initialized byte view for `Bytes` and establishes no
ownership or refcount behavior.
