# Modified bytes 1.11.1: validated increments

The selected modified API is not yet complete. These are integrated production
body proofs under the pinned generic allocation/atomic/library boundary, not
proofs of legacy API compatibility or additional configurations.

## Scoped sharing tree (52)

`tree-final-52` has 52 Coma files, 52 proof results, zero null leaves; every
archive member hash was independently checked. Archive SHA-256:
`45c634540046cb8f0acbedc3a47569bc5e8c4886d0486dc0a84a4a81b30c8379`.

`scoped_tree` creates actual recursive scoped threads for any finite requested
leaf count >= 2. Each node uses a two-child retirement counter and restores its
parent lease; this is a hierarchy, not a flat arbitrary-N refcount or escaping
Clone API. Counts 0/1 explicitly detach and clean up the input allocation.
Native coverage includes 180 tree cases and the existing lifecycle/race tests.
Read counts denote attempts; out-of-range attempts return None.

Proofs cover normal-return executions, byte contents, affine lease recovery and
exact final consuming cleanup. Thread termination, panic/unwind cleanup,
allocation failures, no_std and other platforms remain separate obligations.
There are no new trusted bytes-specific protocol contracts.

## Ordinary exclusive mutation composed with sharing (67)

`phase2-final-67`: 67 Coma files, 67 proof results, zero null leaves.
Independently checked archive SHA-256:
`4e4ec667416b1300098cb7e46565c94568740587a013c8edd69a508d069eabd2`.

ExclusiveBytes uses an ordinary Vec with checked reads/writes, push/pop,
truncate/resize, reserve operations and explicit consuming cleanup.
`scoped_set_and_read` connects ordinary mutation to the same physical allocation's
shared reads and cleanup. Native tests include 411 lifecycle cases, exclusive
operation sequences and the atomic race test. Reserve specifications preserve
contents; numeric capacity guarantees remain unproved.

The missing tree-anchor negative retains one failed proof result: archive
`7d7d2f646994425f415f65f714d0c6d7dd15704a6c85fb37c180c8b28c83fcc6`.
An ordinary mutation attempted only inside ghost code is rejected at translation
(the erased native execution retains 7 rather than the claimed 9):
`3a4356fe1c92111a1911b027f924b2e2ddbc01cb31f32a4e1dd7d052921eeb37`.
These negatives are independently hash-checked and preserved, not passing proofs.

## Checked numeric Cursor integrated with physical reads (131)

`cursor-integrated-131`: 131 Coma files, 131 proof results, zero null leaves;
all archive member hashes independently checked. Archive SHA-256:
`ec7fe13a9003d323fa07bf1adcdc74934a2faf4a68e9a13f8a9d1697e9ab3c14`.
Native selected-feature suite: 10 tests passed.

Cursor proves exact endian integer values and remaining-byte suffixes for
checked reads (u8, u16/u32/u64/u128 BE/LE, signed counterparts, variable-width
u64), advance and copy. Failed operations preserve input and destination.
`scoped_numeric_read` connects the concrete reader to real shared physical
bytes, retirement and cleanup. The existing canonical codec/slice helpers are
now in the same selected production crate configuration. No universal Buf
implementer laws were introduced.

Remaining API correspondence and configuration obligations still include
reusable scoped callback access, same-allocation thaw/conversions, richer write
operations, numeric capacity specifications, no_std/serde/platform variants, and
failure/termination behavior. Passed file counts are not completeness counts.
