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

## Reusable immutable scoped callbacks (133)

`callbacks-133`: 133 Coma files, 133 proof results, zero null leaves, 11 native
tests passed. Independently checked archive SHA-256:
`7f36c4ffb41f7865260842557fea1efb79defa394645d4d40c84ad12eb82f289`.
`with_shared_read` calls two higher-ranked Send FnOnce callbacks on actual
shared bytes in scoped threads, joins them and explicitly reclaims the allocation.
Callback behavior is expressed through each callback's own pre/postcondition,
not trusted universal laws. Results own their data; their types cannot retain
the callback argument lifetime. A concrete length/first-byte client is proved.

Returning the borrowed callback argument as an escaping result is rejected by
both Rust and the proof frontend, before VC generation. Independently checked
negative archive SHA-256:
`07da597ae4095beb567f19126ef2778341882a3849a07ef4e856b62abb247b5b`.
The normal-return boundary remains explicit; arbitrary callback panic cleanup
and callback termination are not claimed.

## Preserved development checkpoints

The earlier tree-first-51 and exclusive-tree-67 archives are retained as
historical source snapshots, superseded by tree-final-52 and phase2-final-67.
The first tree snapshot did not yet explicitly clean up rejected counts and
contained an ignored termination annotation; it is not the admitted tree API.
Routine failed parser/type/spec attempts are also retained: exclusive-view-parse,
exclusive-is-empty-contract, tree-frontend-literal and callback-length-model-typing.
Their archive member hashes have been independently checked. They neither count
as new coverage nor justify retrying a frozen architecture.

## Ordinary Vec-backed typed mutable borrowing (136)

`exclusive-deref-mut`: 136 Coma/proof results, zero null leaves, 11 native tests
passed. Independently checked archive:
`a58c6f0c07ec230f45665d838e086e4bf91ae0fb10a2076b1ac6639446d04a28`.
Deref/AsRef return the actual contents; DerefMut frames the final mutable slice
back into the actual ExclusiveBytes Vec sequence. These are body-proved stock
Vec borrows, not a trusted or ghost-classified raw B4 mutable access.

Removing the final borrow relation breaks the concrete write/read client
(137 files, one failed result at negative_missing_derefmut_frame, 2/3):
`b41c603ce0d299bfe2992798d5b565daf5ecd583bab79602d8a7808b1bdd5c1f`.
Attempting the typed DerefMut borrow of an ordinary owner inside ghost code
is rejected before VC generation: the diagnostic states that a non-ghost
variable cannot be written in ghost code. Native erasure retains Some(7).
The exact source/log archive is preserved in exclusive-derefmut-ghost-erasure.

The initial AsRef purity failure also remains archived; its body was corrected
to the actual stock Vec slice borrow, and the positive source was restored
byte-for-byte after all controls. D01's old raw B4 counterexample remains frozen.

## Sharing -> full authority recovery -> B2 thaw (143)

`thaw-integrated-143`: 143 Coma/proof results, zero null leaves, 12 native tests
passed. Independently checked archive:
`e2ebfd9fd359728b2b1aa2938187436d4a52295238bcbf97829ae9f2c17f05f9`.
The private owner retains B1's original RawAllocation. A body-proved borrowed
metadata accessor supplies B4's BoundPtr without copying physical authority.
Actual close receipts recover all lifetime fractions and the entire original
Recovery/PhysicalRegion. Existing B2 consumes that descriptor and authority to
return Vec; B3 remains the consuming cleanup alternative.

`with_shared_read_then_thaw` returns the contents-preserving Vec and owned
callback results. `share_thaw_set_and_read` mutates that returned Vec and shares
it again in the same production configuration. Native tests check pointer and
capacity preservation for empty/spare-capacity/nonempty inputs. Formal posts
prove exact contents and full affine recovery: stock Vec's model does not
expose returned pointer/capacity identity, so those native identity checks are
not a formal identity theorem. Existing B2's physical mapping is unchanged.

Discarding spare-capacity region authority while retaining initialized-prefix
facts causes the expected B2 full-coverage precondition to fail (19/20):
`thaw-missing-spare-region`, 143 files, exactly one null result. The exact
negative goal and source are preserved, including empty allocated storage.
The routine borrowed-namespace frontend failure is retained separately.

## Ranges, conversions, equality, and numeric closure (183)

`combined-leaves-183`: complete engine exit 0, Proved (183 files),
183 Coma/proof JSON files, zero null leaves, matching 19 native tests.
Independently audited all 407 archive members and hashes:
`c97f86af383fa234783dc81874df9935a964d093804fd208f1902e5fa77cfc70`.
Configuration remains `--locked --lib --no-default-features --features verified,std`
on the host x86_64 target, normal-return execution. This is not full API or
configuration completion.

The actual source graph adds checked ranged callbacks with explicit cleanup on
invalid ranges; slice/string/boxed-slice/IteratorSpec conversions; equality and
consuming equality cleanup; signed variable-width and native-endian cursor
operations; initialized unsigned fixed/variable-width writes; and write ->
actual scoped sharing -> cursor readback -> cleanup composition. Existing
canonical pure definitions be_weight, le_weight and signed_u64 now unfold within
the crate. Their bodies and runtime code are unchanged; no protocol/math trust
is introduced. Write models use finite byte powers and direct indexed sequences.

Preserved failures include frontend visibility, unsupported array-range indexing,
and the first combined numeric-interface failures. The partial interrupted run
with 178 JSON files is not a positive result: a per-file JSON may omit its own
function obligation. Capture now additionally requires an engine completion
header covering every generated file before accepting a positive archive.

The last failed caller exposed a substantive contract bug: the appended-byte
quantifier was nested inside the original-prefix implication, making it vacuous
for an empty prefix. Explicit independent conjunctions strengthen the contract;
all write bodies and their physical readback caller were reproved under that
contract. The failed source remains in combined-leaves-scoped-write-value
(`9e4ece5f684f9b64805b5e7ec80e11c7fca38f10cc2b4716e820c4ab534d40f3`).
Public specification usability outside the writer module/crate, signed/native
write aliases, floats, further adapters/observers and retained configurations
remain separate obligations.

## Ordering, signed/native writes and exact float-bit transport (229)

`ordered-bit-writes-229`: complete engine exit 0, Proved (229 files),
229 Coma/proof JSON files, zero null leaves, matching 26 native tests.
Independently audited all 507 members:
`26c7f386402548e02b53bac3e8441cbb9af6fd7f3f3863000d7c1610b5932735`.
Configuration remains host `verified,std` with the pinned stock tool/library
TCB. No bytes-specific ownership/refcount theorem is trusted.

PartialOrd/Ord use the actual byte-sequence model; PartialOrd delegates to the
proved Ord implementation after the initial slice-to-wrapper projection failed.
The public cmp-and-close caller composes them. Writes add signed fixed/variable
width and native-endian aliases. The physical write/readback client is now in a
sibling module, establishing consumption of writer contracts there. Public
downstream specification usability remains the next interface obligation.

Float32Bits/Float64Bits preserve exact integer bit patterns, including NaN
payloads, and provide checked endian readers/writers. The actual production
write -> physical scoped sharing -> Cursor read -> close client proves exact
binary32 bit recovery. One body-checked numeral recomposition assertion bridges
the encoding digits to the canonical reader; no assumed arithmetic law is added.
These wrappers do not convert to native f32/f64. D08 preserves both actual
materialization failures and the unproved IEEE conversion boundary.

Capture metadata now explicitly records cargo-creusot's injected
creusot-std/creusot and creusot-std/nightly features, the effective proof feature
graph, exact Why3find settings, tool flag source and Vec contract source. Older
native_proof_feature_difference fields compared only requested CLI feature
lists; they must not be read as proof of identical dependency feature graphs.
That correction does not change the completed engine results. Earlier failed
229 candidates and native import diagnostics are preserved.
