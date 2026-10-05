# Actual valid-handle `AsRef` / `AsMut` gate

This independent gate extracts the actual `BytesMut` declaration, `from_vec`,
read/mutable/spare access, length operations, explicit unique cleanup, and core
`AsRef<[u8]>` / `AsMut<[u8]>` implementations directly from `src/bytes_mut.rs`.
The build records exact source fragment offsets and hashes. Generic trait callers
are included, and Creusot generates the actual implementation refinement VCs.
There are no stub traits or stronger implementation preconditions.

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
