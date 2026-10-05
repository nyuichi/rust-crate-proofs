# Actual valid-handle `AsRef` / `AsMut` and observers gate

This independent gate extracts the actual `BytesMut` declaration, `from_vec`,
read/mutable/spare access, length operations, explicit unique cleanup, and core
`AsRef<[u8]>` / `AsMut<[u8]>` implementations directly from `src/bytes_mut.rs`.
The build records exact source fragment offsets and hashes. Generic trait callers
are included, and Creusot generates the actual implementation refinement VCs.
There are no stub traits or stronger implementation preconditions.

This gate also extracts the exact `BytesMut::is_empty` body and its production
contract. A Creusot caller takes a checked valid-handle reference, calls
`is_empty`, compares it with `len() == 0`, and asserts the comparison. The
generated extraction adds only a checker attribute to the exact source method so
Creusot verifies the exact `len` and `is_empty` bodies used by the proof
caller. A separate native `unique_is_empty` caller builds a real unique handle
from a `Vec`, observes emptiness, and explicitly releases the allocation.
Native cases cover canonical empty, allocated-empty, and nonempty handles.

The proof caller is authored in `src/lib.rs` and relocated by the build script
into the generated `actual` module. This keeps the caller in the same module as
the extracted private-field contracts; an earlier outer-module caller produced
an invalid `ERROR_UNBOUND_len` selector in Creusot's generated Coma. The exact
caller assertion is preserved, with only the module qualifier removed during
relocation. The separate `Vec` construction and cleanup remain native-only.

The focused observer proof passes exactly three selected Coma files: `len`,
`is_empty`, and the reference caller. It does not contribute to a full-gate
proof-file count. The generated source snapshot, three proof JSON files, native
and translation logs, and earlier failed caller attempts are archived under
`evidence/exact-is-empty-observer-v2.tar.gz`; its sibling JSON receipt lists the
three selected Coma members and verifies their hashes against the replay inputs.
The v1 archive remains preserved as the earlier evidence snapshot.

`bytes_proof_valid_handle` enables a checked type invariant: initialized unique
ownership or a canonical empty handle
with no allocation resources. The native trait test constructs a unique handle,
uses the real traits to read and mutate it, and explicitly releases its buffer.
The invariant explicitly excludes shared registrations as well as pending descriptors or arbitrary unregistered
handles. Cleanup restores canonical empty metadata field by field only after
allocation authority has been consumed, then forgets the handle to suppress a
second native free.

This is a different invariant context from `sequential-bytesmut-split`.
Promotion, splitting, automatic BytesMut Drop, full trait coverage, and the full
crate are **not** proved here. The lifecycle gate retains its explicit ownership
preconditions without this global invariant. Its successful split proof is not
an integrated split-to-trait proof. Creusot 0.13 checks the current invariant when
forming/resolving mutable reborrows even with `open_inv` / `open_inv_result`;
private incomplete transitions therefore require a larger representation
refactor before these two contexts can be combined.

The three fixed-zero physical reference helpers are explicit local trusted
boundaries. They return only zero-length `u8` or `MaybeUninit<u8>` slices from
non-null sealed descriptor metadata, preserving the native pointer. Their
lifetimes share-borrow the descriptor without changing it. Multiple empty mutable
slices own no bytes. They grant no positive-byte access, initialization,
allocation liveness, or recovery authority. One-byte alignment is sufficient;
these helpers are not generic over arbitrary element types. Other physical
access, Vec ownership, counter, and boxed-alignment boundaries are inherited
unchanged. No ownership/refcount protocol is trusted.

`Deref` and `DerefMut` are excluded: Creusot's standard contracts require ghost
purity, while standard permission-based and local physical access are runtime
operations. The physical bridge is not made ghost-callable to bypass that
restriction.

`negative_unregistered_as_ref` provides structurally valid pointer metadata and
positive length without owner/registration resources, then calls the actual
safe `AsRef` implementation. The intended failed obligation is the handle's type
invariant at that real trait call. It does not assume a false trait precondition.

```sh
./scripts/verify-bytes.sh valid-handle-traits
./scripts/verify-bytes.sh valid-handle-traits --features negative_unregistered_as_ref
cargo test --offline --locked --manifest-path verification/probes/valid-handle-traits/Cargo.toml --tests
```
