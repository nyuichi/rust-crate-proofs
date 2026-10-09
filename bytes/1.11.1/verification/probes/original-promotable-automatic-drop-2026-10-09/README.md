# First promotion with normal automatic Drop — AM, 2026-10-09

Astra recommended AM after audited/published AL361c7cd2. The unchanged native
API witness takes a nonempty Box, constructs original Bytes, creates/drops a
child in an inner lexical scope, reads/copies original, evaluates the return
Vec, then drops original normally. AL ownership core remains byte-for-byte
fixed; the client-only nonempty premise does not change AE's unrestricted From
refinement. Original full architecture remains NOT ADMITTED.

## Proof and native path

Two ordinary body-proved consuming wrappers copy AL cleanup contracts exactly:
`bytes_child_terminal_drop` and `bytes_root_terminal_drop`. Their only native
argument is the consumed Bytes; PromotionScope and Completion loans are erased.
Child completion is KeptAlive, actual child-ID removal preserves root/pointer
ownership, and the real singleton ledger precedes root final cleanup. Root
consumes updated old/new AtomicPtr history through get_mut_finish, reaches the
promotable ARC branch, and AL release performs the actual Acquire and both
frees, proving actual empty ledger and paired receipts. No readonly re-sealing,
resource-valued logic getter, guessed ticket or trusted Bytes Drop is added.

The selected actual after-ElaborateDrops MIR binds inner child `_5` at bb2
(return bb3, unwind bb9), root AsRef/to_vec at bb3/bb4, saved return `_0 = move
_7` at bb5, then original `_2` Drop at bb6 (return bb7, unwind bb11). The proof
shadow elaborates only these normal edges and has no native Bytes Drop impl;
it must not execute both destruction mechanisms. The native one-test witness
checks lengths1,2,31,256. Nineteen selected distinct MIR bodies (18 production
plus client) are pinned. These executions do not demonstrate both physical
pointer parities; the mathematical constructor/promotion proof covers both.

The mathematical development gate passed115 files/824 prover leaves/zero null,
but its archive `am-positive-diagnostic-v1` deliberately recorded checker-not-run
status2. It is not canonical admission. The final feature-free canonical gate passes115/824/0 with correspondence
status0. Archive `am-canonical-v1` SHA256
7db16b230e56cd75c9cffb34620a2a4593af23ff68bbc03ed8d5913050b4424e contains6281
hashed members. It uses the full independent source/native/generated/compiled-
input checker before proving all115 targets. Independent archive audit gates
publication and records scope limits. A successful proof leaf may come from sequential solver
fallback; only one concurrent prover process is permitted,1024MiB,sc-drf off.

## Checked correspondence and remaining assumptions

The checker fixes AL source modules and complete helper/client token streams,
attributes, signatures, bodies, erasure channels, root/child places, successors
and order. It independently reconstructs actual AM OUT_DIR/public_records.rs
from production sources, checks Cargo build fingerprint/directives/root-output,
and archives all compiled bytes/receipts. It does not rely on AL's Cargo cache.
The manifest `reviewed-production-inputs.json` pins all61 production source
files plus Cargo manifest/lock to AL361c7cd2; its own SHA is hardcoded in the
native checker. This closes global import/macro/record-include redirects even
when capture receipt hashes are recomputed.

Generic TCB remains: precise Core/Std atomic and field interpretation/callbacks,
allocation and physical borrow/free, exposed-provenance tag roundtrip, exact
function-item erasure, and native compiler/MIR normal terminal-place mapping.
Terminal mapping requires no observation/escape of receiver/data-field address
and no independent native field-drop glue. Exact selected callback bodies and
all production global bindings are checked. Compile-time needs_drop assertions
corroborate the four native field categories; references have no drop glue for
any referent by the generic language boundary. Typed free proves storage
consumption, not arbitrary T::drop. Source/MIR checks are not compiler adequacy,
allocator or strict-provenance proofs.

CAS loser/concurrency, independently escaping arbitrary clones, unwind/abort,
allocator/to_vec failure, general termination, arbitrary moves/API composition
and other configurations remain excluded. This gate proves normal completion
of this exact scope and does not claim the full crate.

## Distinguishing experiments

Main source/mapping controls reject45/45; native controls reject32/32. Review
caught and closed two additional gaps: a client macro overriding proof_assert!,
and production record/atomic import redirection. The former now fails a complete
client-file token comparison; the latter fails an independently anchored whole
production input gate. Their concrete mutations are retained as controls.

Six full115-target semantic captures remove child effect, remove root effect,
swap adapters, omit Acquire, omit payload free, or omit control free. Actual
null counts are1/1/3/2/2/2. The two omission-feature failures are in both new
promotion code and the retained public_shared code, not two independent AM
lifecycle defects. Printed exact tasks show that omitted child effect leaves
resource-resolution sensitivity and omitted root effect leaves content/borrow
resolution sensitivity; these are not direct native-drop-absence theorems.
Missing-free Ghost::conjure branches likewise establish proof sensitivity at
invariant/receipt prerequisites, not an independent physical-free theorem.
The unchanged AL first_promotion_client remains in every target closure; its
explicit cleanup calls are distinct from the exact promoted_automatic_scope
control function, which really omits the respective terminal calls.

Four frontend captures swap places, consume child twice, consume root twice,
or move root cleanup across the live slice. They reject with E0382/E0382/E0382
and E0505+E0502, before any Coma/proof output. They are type errors, not null VCs.
All immutable inputs/member/target hashes, exact null tasks and interpretation
limits are audited in AM_SEMANTIC_CONTROL_AUDIT.md, AM_FRONTEND_CONTROL_AUDIT.md
and evidence/*.json. Optional Why3 plugin Dynlink warnings are recorded for
task printing; successful exit0 plus exact stdout is distinguished from an
infrastructure failure. No solver is run by the task printer/audits.

The initial diagnostic capture predates the later native field-profile and
production identity gates. Its original script/log/receipt bytes remain
immutable and are not reinterpreted as the final gate.

## Portable reconstruction

Restore archive inputs/repository at a repository root and copy probe into
`bytes/1.11.1/verification/probes/original-promotable-automatic-drop-2026-10-09/`.
The dependency closure includes AL, AJ, AK and their pinned support/checker
inputs. Restore the private Std110 files and exact tool/config pins from the
installation manifest; binary payloads are not included. Activate pinned tools,
clear verif/backend caches, rerun capture-native.sh and run-proof.sh elevated
(Why3 sockets). The launcher regenerates the exact frozen-prefix/terminal tail,
translates, checks live compiled records and correspondence, then proves every
target. It does not skip targets or enable defect features. Archive-only audits
recompute source/target/proof/member hashes and replay checker controls without
solvers. Location-specific Cargo root-output paths are observations, regenerated
for a fresh layout rather than treated as portable source inputs.
