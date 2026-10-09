# Actual Shared terminal automatic Drop

Astra selected this gate after the audited/published AI increment. The original
native client constructs `first`, clones `second`, copies `second`'s public
AsRef view into an owned Vec and returns it. It has no explicit cleanup or
`mem::drop` call. Only this client requires input length less than capacity,
selecting the unchanged Shared constructor branch. The unrestricted AE
constructor proofs are separate and unchanged.

Pinned native MIR after ElaborateDrops evaluates the returned Vec first, then
drops `second` (`_4`, bb5 -> bb6, unwind bb10), then `first` (`_2`, bb6 -> bb7,
unwind bb12). The external elaborator creates a proof-only artifact without a
Bytes Drop implementation before Creusot borrow/liveness analysis. Each certified
terminal place is consumed through an ordinary, body-proved
`bytes_terminal_drop(Bytes, Ghost<&mut Cursor>, Ghost<&mut Option<Completion>>)`.
It forwards AI's body-proved checked cleanup implementation and contracts.
Nothing asks an already destroyed `&mut Bytes` to regain a live invariant.

This is a terminal destructor path theorem, not a reusable ordinary specification
of `Drop::drop(&mut self)`. The actual production destructor is unchanged. The
proof preserves its expanded native vtable target and data/pointer/length
argument operations. Its completion channel retains actual buffer/control free
receipts rather than a ghost-only flag. It proves the copied contents, second's
KeptAlive result, first's valid Reclaimed pair, and an empty owner map. Per-owner
map-removal facts distinguish swapped owners independently of output labels.

## Additional terminal-place correspondence premise

Moving a value to a consuming shadow is not equivalent to arbitrary native
Drop. The complete selected destructor/target closure must neither observe nor
escape the address of the Bytes handle or its data field. Native operations only
project stored pointer/length/vtable values and copy the stored pointer through
core AtomicMut/get_mut. The actual callback, closure, AtomicMut alias/body and
MIR must be checked. Heap buffer/control pointers retain their real identities;
this premise concerns handle placement, not those allocation identities.
There must also be no independent native Bytes field-drop glue. The native
harness compiles a zero-needs-drop assertion for the exact field categories
(raw pointer, usize, core AtomicPtr<()> and a static borrowed reference).
Matching a static Vtable reference to that borrowed-reference category uses the
generic rustc/Std fact that references do not own their referents. AI's Shared
field profile, actual Shared destructor, no-drop atomic guard, and two raw frees
remain separately checked. Added address observation, escape, field glue or an
unknown callback is rejected rather than modeled speculatively.

Native compiler/MIR, exact terminal-place source/shadow and borrow interpretation,
Ghost erasure/reification, and AI's reviewed generic field/cursor/physical/effect
primitives remain explicit TCB. Bytes-specific issuance, lastness, recovery,
cleanup and this helper/caller are body proved. The removal path is shipped
native destructor-effect/terminal-ownership interpretation preserving these
helper contracts and rejecting the same controls; no bytes destructor axiom or
broad Creusot patch is introduced.

## Reproduction and scope

Run `generate-native-mir.sh` to compile the immutable native source and production
destructor/targets under pinned Rust, capture eight selected after-ElaborateDrops
MIR files, execute five native input cases and generate the positive shadow.
It disables incremental compilation, cleans only the two native packages, and
records pinned Rust/cargo versions and exact capture commands. Scratch dumps
stay outside this probe; archived selected inputs and hashes are in native-mir.

Run the independent correspondence and structural-control checker before the
canonical `run-proof.sh` (elevated for Why3 sockets). It serializes on the shared
lock, uses one prover/1024 MiB, disables sc-drf, includes every Coma target and
forces translation of the selected shadow. Native execution is corroboration;
canonical acceptance also requires body proofs and independent reconstruction.

Semantic diagnostic mode can intentionally skip the evolving checker with
`BYTES_SCOPE_DIAGNOSTIC=1 BYTES_DROP_CHECKER_SKIP=1`. Its receipt explicitly says
`not_run`; those results are not structural acceptance or canonical evidence.
`BYTES_DROP_FEATURE=omit_second`, `omit_first`, `swap_places`, `duplicate_second`,
`wrong_place`, or `early_second` selects deliberately wrong shadows. The latter
three also exercise ordinary affine/loan type rejection. Cargo missing-Acquire
and missing-either-free features retain AI's semantic defect checks. Structural
and Rust-type failures must be distinguished from Why3 null leaves.

`evidence.py` freezes exact probe/target/result/native inputs, all current bytes
sources/manifests, prior compatible lifecycle and AI source/checker inputs,
pinned private Std, tool/configuration/Cargo patch/base activation and run log.
Its audit checks archive members and every target/result hash without a solver.
Absolute native harness/tool-root paths are original-environment metadata;
regenerate the native harness at a restored repository path and map activation
and Cargo patch roots to restored pinned installations. This changes environment
paths, not the executable source/contract semantics.

Admission is limited to the exact two-owner default-std Shared straight-line
normal return. Unwind edges are recorded but excluded from correctness claims.
Other representations, arbitrary moves/Drop glue, arbitrary concurrent closure,
API/configuration coverage and full original-architecture admission remain open.
The frozen stock Drop-to-Goto counterexample is retained; this is a genuinely
changed external elaboration premise, not a stock retry.

## Validation

Canonical restored positive: **77 files / 445 actual prover leaves / zero nulls**,
all targets included, no enabled defect features, explicit active-source
correspondence pass. All **62** structural controls reject, including ten support
module literal-path mutations added after independent review exposed a checker
acceptance gap. Six semantic controls each leave one null and three type-only
controls yield E0382/E0382/E0505 before any VC generation. See
`evidence/SEMANTIC_CONTROLS.md`; diagnostic checker skips are not acceptance.

Canonical archive `automatic-positive-final1-2026-10-09.tar.gz`, SHA-256
`a550b2564160823fe425157ed45dba90471577d3124e6677e7d101b2bacb55c9`,
contains 1717 hashed members. Its manifest and audit accompany it in evidence.
Independent source/checker/archive reconstruction is recorded in
`evidence/INDEPENDENT_AUDIT.md`. Final reporting files are outside the immutable
input snapshot to avoid self-reference; executable proof inputs are frozen.
