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

## Concrete adapters, public encoding models and explicit iteration (255)

`adapters-public-spec-255`: complete engine exit 0, Proved (255 files),
255 Coma/proof JSON files, zero null leaves and 32 matching native tests.
Root independently checked all 561 archive members:
`5025308b66066f2d308eb731334b701f554430b047011c04038aa7c8f2b4ae74`.
The configuration remains host verified,std with the unchanged primitive TCB.

LimitedCursor exposes a bounded prefix and advances its underlying cursor;
ChainedCursor copies across the two slices, preserves failed destinations and
supports a checked numeric read. Actual shared-storage clients compose both
with the physical lifetime and cleanup protocol. Borrowed iteration uses the
stock slice iterator; OwnedByteCursor retains ExclusiveBytes and closes
explicitly. Its successful next_byte contract specifies the full remaining
suffix. No automatic Iterator destructor cleanup is claimed.

Two equivalent sequence-composition failures triggered structural redesign:
separate ghost helper bodies prove nested-suffix composition and two-piece
prefix reassembly using stock sequence extensionality. They and their actual
callers are included in the integrated positive run; no assumption or weaker
postcondition replaces the failed goals. Initial semantic failures and the
intermediate snapshot frontend diagnostic remain archived.

Encoding byte-sequence models are now public and shared by integer and bit-word
writes. A separate downstream crate test remains required before claiming
external contract usability. Capacity identity, no_std adoption, further trait
observers, configured targets and failure paths remain outstanding. D09 and D10
record concrete Serde and callback-unwind boundaries; neither is completion.

## Public read-model exposure (255, unchanged runtime)

The downstream unsigned variable-width reader initially fails 8/9 goals because
the canonical weighted-sum definition is opaque outside bytes. Its exact
failure is preserved in the consumer evidence. Ten existing pure Seq/Int model
definitions are now public/open and reexported by encoding_spec: endian weights,
signed interpretations and variable-width helpers. Bodies, runtime code and
primitive TCB are unchanged. The actual production replay completes all 255
files, zero null leaves, with 32 native tests. Root independently audits all
561 members and the proof results in public-read-models-255:
`20874bb08c829e65ce679af39605423c44cd29e966db57734517b5625095e565`.
The downstream consumer replay is a separate obligation, still pending here;
this production result alone does not establish external usability.

The downstream integrity replay now completes six proof files, explicit exit 0,
zero null results and three native tests. It exercises BE/LE writing plus
unsigned variable, signed variable, signed fixed and native-endian i128 reads.
Root checks all 69 tar members (68 data members plus the hash manifest) and
matches all 35 dependency src/Cargo files against public-read-models-255:
external-six-positive,
`96b06b29ef745327c631bf8514d0564dca0c076952043d3e4f17a60f7a5f334b`.
An integrity replay obtains the missing transport exit status explicitly; it is
not a new approach. Arithmetic harness and missing Why3find configuration
diagnostics are separate from the actual opaque-weight contract failure.
These are representative external clients, not a claim that every possible
downstream combination or generic trait implementation is covered.

## Production alloc-core integration

The actual production source now separates alloc APIs from std-only scoped
sharing without dropping the public read-model exposure. The core integrated
proof completes 221 files, zero null leaves, exit 0; native core/std tests pass
14/32 and original default/no-default library checks pass. Exact sources and
logs are captured and independently audited in production-alloc-core-221,
SHA256 `9d12b34ab533b7d95612e4782a33905b0034077b50ee287df836f1bde9afff65`.
The relocated production std graph is a separate pending proof. The core proof
still enables stock creusot-std/std models; genuine alloc-only proof dependencies
and final cross-target API coverage remain outstanding.

## Actual observer-trait failures and bounded replacement

Hash-only and Debug-only gates are captured and independently audited after
actual prover runs: 259 files / 2 unresolved results and 259 / 4 respectively.
D12 freezes only the unchanged generic Hasher and standard Formatter contract
routes, after Astra review. Two missing decimal byte-range model prerequisites
are separately repairable, not a formatting impossibility claim. Explicit
deterministic byte-digest/hex-byte APIs remain in development. Their semantics
will be stated directly rather than presented as generic Hash/Debug verification.

The relocated production std graph subsequently completes all 255 files, exit 0,
zero null results, with the matching 32 native tests. Independently audited
production-relocated-std-255 archive:
`aed75d692ae671d64ec71a1e670a94e349b0b44aba82b57976df8477cbbaf018`.
This closes host integration for the scoped relocation and current public read
models; later additions and genuine alloc-only proof dependencies remain separate.

An isolated closed convenience/IO candidate completes 263 files, zero nulls,
exit 0, and 37 native tests. Root audits its exact source and all members:
closed-api-conveniences-io-263, SHA256
`be686ccc2abf42009caf4320d334fc81df4e10bf38fccc040462d3d26173a7db`.
It proves initialized zero allocation, clear, consuming append with explicit
cleanup, copied string append, framed partial-prefix reads and concrete
Read/Write methods. No universal IO implementer law is assumed. Its source is
not yet integrated in production. Initial frontend/configuration/socket failures
are archived separately; the elevated replay is the proof result.

## Production closed convenience and IO integration

The actual current production source now completes263 proof files, exit0,
zero unresolved results, with37 std native tests and16 core native tests.
It incorporates clear/zeroed, consuming append with explicit temporary cleanup,
string append, framed partial reads and concrete Read/Write. Flush also now
proves the owned byte sequence unchanged. Root independently audits all archive
members and263 proof results: production-closed-io-263, SHA256
`f3637b33418fa11c1df9d29641f2362952f16acc77849a6175071ff7610114e1`.
The corresponding core proof is queued separately. Generic IO implementer laws,
allocation failure and unwinding are not implied by normal-return method bodies.

A stale handoff patch contained the already-repaired zeroed literal type error.
Root preserved the actual failed production snapshot before restoring0u8; no
runtime change was needed. production-closed-io-zeroed-type-frontier archive:
`54742d5fdee8f1deb84152283e06fd0b7a95eeafac7482be9add10778b3ba4d8`.
Future source integration must use independently audited positive source hashes
and regenerate patches from those snapshots, rather than assume draft parity.

The current closed-API production alloc core also completes226 files, exit0,
zero null leaves, with16 native core tests. The exact current source and logs
are independently audited in production-closed-io-core-226, SHA256
`425e40a45029106ca8f57fd748c69f7d1679ca3bf9abec1d2a28200307ad58dc`.
As before, bytes/std=false but proof-only creusot-std/std=true; the genuinely
alloc-only library port remains a separate in-progress configuration.

## Actual portable atomic backend adoption

Production adopts the byte-exact positive NativeAtomic/fence mapping. Root
independently rehashes the positive and negative archives and compares all
captured production src/Cargo files against the integrated source. The host
verified,std,extra-platforms graph passes263 files, zero nulls, native37:
portable-atomic-host-263 SHA256
`350bca93c4b3b5060c6ac8b5b4a9937b9ecd70dd74b1aaa5013eaa7079e15b56`.
Both Release RMW and the final Acquire fence select portable-atomic1.15.0,
under an explicitly reviewed generic atomic TCB. No protocol body is trusted.

Removing only that selected Acquire fence fails retirement64/66 (two nulls),
with all263 files captured: portable-atomic-missing-acquire-263 SHA256
`b25d8daf7dca3c5a3f41e8e81d6b34cf12628fe620b9b03168158a50aab31639`.
The actual protocol cannot recover the peer AtView without synchronization.
The orphan sandbox invocation is preserved separately as an environment failure.
Final enlarged API/cross-target configurations and generic primitive adequacy
remain separate obligations; MSP alloc-only does not include scoped atomics.

## Copied checked splits and specified iterator append in production

Actual production with the portable backend completes266 proof files, exit0,
zero nulls; std/portable native38 and core native17 tests pass. Root independently
audits all archived members/results in production-copied-splits-portable-266.
The two checked copy splits specify exact retained/returned sequences and
invalid-index source preservation. Both owners require explicit cleanup;
try_split_to_copy closes its temporary suffix through append_owner. These are
functional copy replacements, not O(1), shared allocation or persistent handles.
append_iter consumes only an IteratorSpec-constrained conversion and closes
the temporary owner; no arbitrary downstream iterator law is trusted.
The separate isolated nonportable266 source is also preserved and audited in
copied-checked-splits-append-iter, SHA256
`63b22a789cd98528d78f55ae92cdecccfcfd10f924db1e7d03e3483d1ef68151`.
Final configurations still require the eventual complete API source.

### 2026-10-07: audited alloc-capacity and external IO evidence

The frozen generic alloc-support candidate bundle `d9a7bcd0d9467215153e4e11aed73766da554b145b4a171b4ab518c16ab26d5d` passed an independent outer archive/member audit (5833 data members plus its self manifest). Its two canonical positive captures have 226 Coma files, 226 matching proof JSON files, zero null leaves and observed exit 0: genuine MSP430 UInt16/Slice16 and host UInt64, each with an alloc-only effective proof graph. These are candidate-source results, not the current 266-file production source. The local candidate Std contracts and feature exposure are retained in the outer bundle; the historical standard capture's stock installed-Std snapshots do not describe that local dependency. Four earlier unproved leaves are preserved in the supplemental failure snapshot; the canonical diagnostic capture for that failure has no Coma files. No body coverage is inferred from that empty canonical capture.

The external concrete Read/Write/flush consumer archive `fb859419b31359be23685512a93ba8acda42fdb42e9fae06c3aa855910f60892` passed an independent audit: 66 data members plus its self manifest, three matching body proof results, zero null leaves, and observed successful exit. Every dependency src/Cargo snapshot equals the independently proved production-closed-io-263 archive. Its two native tests cover 30 input geometries. This proves actual downstream use of the concrete public contracts at that dependency snapshot; it does not prove arbitrary implementations of IO traits or the later production source.

Both increments preserve their failures and record `full_coverage=false`. The generic capacity contracts remain an explicitly reviewed Std trusted boundary; bytes ownership/refcount/cleanup bodies are not moved into trust.

### 2026-10-07: audited changed-interface candidates

Deterministic digest/hex: archive `a4f04f92ce61c3a9ddd43f3ab930d9ee62beba4c08cacc375df321981ab3995d` has 266 matching body proof files, zero null leaves, and observed success. Its source is a different candidate from production-copied-split-portable-266 despite the same count. Exact polynomial modulo-2^64 and lowercase ASCII hex models, runtime steps, sequence lemmas and source-consuming cleanup are body proved; generic Hash/Debug compatibility is not claimed. Native candidate tests pass (34). All preceding frontend and four-null body failures are preserved and independently member-audited.

Copy-slot spawning: historical label copied-slot-option-229 actually contains 256 matching positive body files, zero null leaves, observed success, archive `56d8767eb6117043bc14008288a0be6c3a64bc5c6b29a333605d835b7705502d`. The actual local generic Std method and native forced failure/success tests are retained in augmented bundle `728a49f81def0a6585aa0abcb9ca9c9dbf8bfe21d3199758543bf3941cb9523c`. The method is a reviewed generic trusted Std contract, not a proved library body; bytes parent-slot read/retirement/cleanup caller bodies are proved. Mutation-then-None rejection archive `8dd93014c04786580f39db0d2c187d1dae114f1e11fe56b4705b8a4df1c4f435` has 257 translated files but only one targeted proof JSON with a null result; it is not a complete 257-file negative run. Supplemental review bundle `4cd361a495e4766001caac3b944d24e8de491d0a06904daf2eacc401678475da` preserves its exact source and failure. F:Copy excludes captured destructors; no frame for unrelated atomic/global state is assumed.

Capacity lifecycle archive `42eaea7919b1e5d52ec139412b4acea771fe38a0c3155d7fade31d0db61cc49f` proves 261 matching bodies, zero null results. Its paired missing-spare-region control `ae5619553cb4ae80f9673733dbbabeb21b464c9071b47597ca7a3fc740140a6e` leaves exactly one null in RecoveredAllocation::into_vec. Observable numeric address archive `6cdc582b07544775918645cd2a55dd1d289279bd1f0cc942a3f5281b46833898` proves 262 matching bodies, zero null results; paired missing-spare archive `1a39206bee28eb48fe324446cb1695d283dcf2b091c1b9d299ecbd48bbbee562` again fails that body alone. The initial missing share_pair base frame failure is retained. Actual local Std capacity/base models are metadata on Vec, not permission or provenance witnesses. These isolated results do not claim the current integrated production source; its address witness additionally needs explicit closure of the returned Vec.

### 2026-10-07: actual combined production gate, 285 files

Current production source with copied splits, specified iterator append, portable atomic protocol, closed IO, capacity bounds/error frames, numeric address-through-thaw frames, deterministic digest/hex and parent-retained Copy-slot spawning passed the whole configured body gate: 285 Coma files, 285 proof JSON files, zero null leaves, observed exit 0. Archive `491b1fa8189f9cf0abb74bab19349391e4a7ae56d30aee671a20044a411f2ff6` (production-capacity-address-digest-copy-slot-285) was independently archive/member audited. Native library tests pass: 48 std/portable and 22 genuine alloc-only. The address observation client now explicitly closes its returned Vec, unlike the isolated witness.

The capture retains a pre-run Cargo/src hash fingerprint, the actual locally resolved Std package's complete Rust/TOML sources, native/effective-proof feature graphs, the generated Cargo path override, installer, reviewed patch layers and original/final hash manifest. The private normalized registry package tree is `878a63ae09dde611de547994499803158b6aa6157ce48f4332f645dff3edb155`; installed stock registry and checkout remain unchanged. The target-local ignored override is regenerated per environment. Cargo.lock records the private path-patched creusot-std identity.

The generic Std contracts remain explicit TCB, not body proofs; bytes sharing/refcount/recovery/retirement/cleanup stays body proved. This closes the functional production integration for these increments, not final configuration coverage, general panic cleanup, generic Serde/Hash/Debug/native-float compatibility or universal termination. Those exact limits remain recorded. Finite-cursor termination and final complete-source configuration/control gates follow.

### 2026-10-07: finite byte computation integrated, 287 files

The final production source adds checked termination classifications to allocation-free Vec ownership wrapping, length/get and owned-cursor new/remaining/next. `read_first_and_return_cursor` has exact byte/suffix/initial-length postconditions and check(terminates); it returns the live cursor. `read_first_and_close_normally` connects that computation to explicit cleanup without a termination marker on allocator cleanup. The whole std/portable source gate passes 287 matching Coma/proof JSON files, zero null leaves and observed exit 0; archive `eff1fb5498ac99a65327d9b62c64c3809e34d98f61906c4899422dc049f183d7` is independently audited and captures the actual Std package. Native tests remain 48 std/portable and 22 alloc-only. D13 preserves actual failures and the checked-trait divergence control. Final alloc/configuration/control/consumer gates use this source rather than the preceding 285-file source.

### 2026-10-07: same production source, genuine alloc-only gate

Production-finite-cursor-alloc-247 uses the same Cargo/src bytes as the preceding std/portable 287 gate. Actual configured translation and whole proof pass: 247 matching body files, zero null leaves, observed exit 0, archive `e2ed53f1ddc1812ea262c71d28271f0b349eb9b9e6937d5343a85aea6795fcfd`, independently member-audited. Requested bytes features are verified (implies alloc); the target creusot-std graph is alloc,creusot,creusot-deps,nightly with no std or sc-drf. Host proc-macro dependencies and listed but unused dev-dependencies are not asserted to be no_std. Native alloc-only library tests pass 22. The 40 absent std-only proof bodies are explicitly outside this configuration, not discarded APIs. Both configurations' actual Std dependency sources accompany their captures. Complete-source cross-target and core-atomic std gates remain pending.

### 2026-10-07: same production source, standard core-atomic backend

Production-finite-cursor-core-atomic-287 proves the same compiled Cargo/src snapshot using `verified,std`, without extra-platforms: 287 matching body proof files, zero null leaves, observed exit 0. The actual core AtomicUsize/fence branch and reviewed RA protocol are included. Native library tests pass 48. Archive `c16f9cc895dc828fd6676398b90bcddef5273afd0ffd0f74bff00adf41f19c68` was independently archive/member audited and includes actual Std package, effective normal/build feature graphs and pre-run fingerprint. This is a separate backend gate, not inferred from the portable result. Cross-target and paired control gates use this source; small additional API-closure candidates are isolated and not yet adopted.

### 2026-10-07: fresh paired final-source resource controls

Both controls start with the same compiled Cargo/src snapshot as production-finite-cursor-std-287, including the reviewed capacity/base/Copy-slot Std additions and the explicit address-witness close. Independent archive audits and compiled-source comparisons confirm one changed Rust file each and unchanged Cargo inputs.

- final-root-missing-acquire-portable-ra, archive `6d1c5e132b24ff718c11fe93e85f47d6af39705c3d13c3e9f7e2938db9bbd1c2`: actual portable Atomic backend, 287 Coma/JSON, observed exit 1, exactly two null leaves in SharedRetirement::retire after replacing final Acquire fence with the unacquired current view.
- final-root-missing-spare-region, archive `5d399208782ce535222194c2984c0567c1032a01d0acdb855a4c7c89dfec9b41`: core Atomic backend, 287 Coma/JSON, observed exit 1, exactly one null in RecoveredAllocation::into_vec (19/20 tasks proved) after discarding spare physical region authority.

No unsafe native negative ran. Actual Std source and feature graphs accompany each capture. These establish rejection at this source snapshot; the historical empty-view/lifetime and duplicate-receipt controls are labeled historical, not claimed exact-source matches. Fresh controls for those responsibilities are being prepared. Small extra API-closure candidates remain separate and will require their own production proof/configuration mapping before adoption.
