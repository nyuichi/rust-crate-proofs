# Read-only B4 ghost classification candidate

The isolated positive gate proves **34 files**, including a fresh replay after
production adopted the reviewed immutable classification. Its native matrix
passes all value pairs and capacities. The generated physical module now
retains the exact production `borrow_bound` annotation and body; only the native
allocation-module path is relocated. Mutable and uninitialized write bridges
remain ordinary program functions.

An actual `Deref` implementation reads a live initialized region through that
bridge. The caller writes a byte using ordinary mutable B4, checks both ghost
and native reads, ends their shared borrows, performs another real write, and
checks both reads again before actual deallocation. Ghost evaluation observes
existing bytes; it neither mutates the slot ledger nor changes initialization.
The shared physical-region borrow prevents recovery or writes while the reader
is live. This justifies the narrow immutable classification, subject to the
existing physical-access TCB; it is not a proof of that trusted raw-slice bridge.

`negative_ghost_write` rejects specifically because mutable B4 is non-ghost.
`negative_stale_read` asserts that a later read returns the previous distinct
byte; fresh replay rejects exactly that assertion (1/2 caller goals). Full evidence includes source snapshots, generated
bridge, Coma, proof JSON and logs under `evidence`.

This gate does not establish default native Bytes/BytesMut Deref, their
missing cfg-independent ownership invariants, shared fractional access, mutable
Deref, Clone, Drop or Send/Sync. Classifying a mutable bridge as ghost remains
unsound and is explicitly excluded.
