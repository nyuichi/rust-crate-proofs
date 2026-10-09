# Actual Shared closed client

This increment connects the closed issuance cursor to the original Shared Bytes
fields, constructor, shared-reference Clone, vtable callbacks, borrowed slice,
and explicit consuming cleanup. It does not admit automatic Drop, unwinding,
arbitrary concurrent clients, other representations, or the complete crate.
Only this client assumes input length is less than capacity. The existing
unrestricted constructor/refinement proof remains separate and unchanged.

`src/native_client.rs` uses the actual public API. `native-check.sh` compiles this
exact file against the production crate and exercises five inputs including an
empty vector with spare capacity. The proof shadow has the same ownership trace:
three handles, two early retirements, another clone, a borrowed slice held while
its peer retires, a content copy, and final retirement. Returned ticket identities
are observed rather than presumed or limited by a quota.

The cursor is a separate affine ghost argument; the Bytes sidecar contains only
its invariant descriptor and the existing ownership resources. Existing generic
three-native-argument erasure preserves native callback signatures. Cleanup has
an erased output channel containing `KeptAlive` or `Reclaimed(buffer, control)`.
The latter carries two opaque affine effect receipts from actual free calls.
The typed receipt records consumption of storage ownership, not execution of
`T::drop`; zero-sized disposal performs no physical free. Shared has nonzero
layout. Buffer and control deallocation remain distinct.

Trusted boundaries are explicit: pinned private Std/toolchain semantics,
closed-source ghost/native correspondence, existing callback reification and
three-argument erasure, scoped field events, physical provenance/allocation
adapters, and generic typed-free effect interpretation. Their contracts preserve
exact state observation, native pointer/model/lifetime identity, memory order,
allocation/layout, and consumed ownership. Bytes-specific issuance, release,
lastness, Acquire recovery, content and final-completion laws are body proved.
The local pointer wrappers retain the existing address-only STD-PTRWRAP-01
contract and all RawVec offset helper targets; no pointer dereference authority
is introduced.

Run `native-check.sh` first to generate and execute the checked harness, then
`python3 check_checker_controls.py`. Run `run-proof.sh` with elevated execution for Why3's Unix sockets. It uses one
prover, the shared lock, a 1024 MiB budget, and no sc-drf feature. Every generated
Coma target is included. Canonical capture requires the independent correspondence
checker to pass; diagnostic runs cannot be labeled canonical. `evidence.py`
hashes archived probe sources, generated targets/results, external source inputs,
private Std, tool configuration and run log. Structural checker controls and
semantic missing-Acquire/payload-free/control-free controls have separate labels.

Restored canonical replay proves **75 files / 433 actual prover leaves /
zero nulls**, with no exclusions/features and successful full correspondence.
All **72 structural controls** reject, and the exact native client passes five
inputs. Intermediate diagnostic evidence is immutable. Canonical receipts and
independent audit are stored under `evidence/`.

## Portable evidence replay

Archives preserve repository-relative inputs under `inputs/repository/`, all
probe files under `probe/`, pinned private Std under `inputs/private-std/`, and
configuration/activation inputs under `inputs/tools/`. Generated native harness
files record the original absolute worktree paths, so regenerate them with
`native-check.sh` after restoring the probe and production inputs at a chosen
worktree location. Its path computation uses the probe's location and does not
require the recorded worktree path. Re-establish the pinned tool installations
from the installation manifest; update the archived Cargo patch path to the
restored private Std location if the tool root changes. The scripts' default
activation root remains `/workspace/bytes-proof-tools`; changing that root is an
explicit environment mapping, not a source/contract change. Solver/runtime and
native erasure interpretation remain tool TCB. `evidence.py audit LABEL` checks
all archive member and target/result hashes without requiring those original
paths or a solver; independent correspondence replay checks source adequacy.

## Retained semantic controls

| Deliberate defect | Files | Actual prover leaves | Null leaves | Failed body |
| --- | ---: | ---: | ---: | --- |
| Missing final Acquire | 75 | 460 | 1 | shared_drop_checked |
| Missing buffer free | 75 | 439 | 1 | free_recovered |
| Missing control free | 75 | 441 | 1 | free_recovered |
| Additional unretired owner | 75 | 449 | 1 | actual_public_shared_driver |

Missing-free controls replace the missing boundary with an attempted generic
`Ghost::conjure` receipt in an untrusted body. Its false precondition cannot be
proved: the caller cannot synthesize a receipt after omitting the actual effect.
The printed failed tasks are retained, including their normalized logical goals.
The missing-buffer control normalizes its failed false precondition to
`not inv_Atomic_usize(Shared.ref_cnt)`; it is a rejection/sensitivity result, not
a separately proved buffer-receipt theorem.
The unretired-owner control's final `Reclaimed` assertion fails. Each negative is
captured before restoring the positive input; none is canonical positive evidence.

This gate uses a checked source mapping from native public calls to selected
proof helpers; it does not use the historical narrowed `From` trait shadow under
`bytes_original_shared_gate` and does not claim a new unconditional trait
refinement. The separate unrestricted AE constructor gate supplies that earlier
constructor result. The native selected-branch mapping is a separate source
correspondence obligation here.

The pinned Std analogue for typed allocation ownership is `std/ptr.rs`
`Perm::from_box` (563) and `Perm::to_box` (681): trusted conversion connects the
actual pointer to an affine `Box<Perm<*const T>>`. `Perm::drop` (696) reconstructs
and destroys a Box, so it also runs the pointee destructor and is not the raw
storage-only operation used here. The receipt extension interprets the actual
raw allocator effect with that same ownership precondition. Replacing this local
trust requires a shipped/native allocator-effect interpretation that preserves
the receipt interface. Scoped field events similarly can be replaced by a
shipped owned-atomic event adapter with the same cursor-state and typed-lease
contract. Native callback ghost mapping can be replaced by supported erased
extra-argument/lifetime interpretation. These replacements do not require
changing Bytes lifecycle body proofs.
