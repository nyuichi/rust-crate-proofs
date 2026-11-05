# Intrinsic delegation alternative: source/IR investigation only

No source/harness/model change, compiler build or solver run was made. Existing
successful fixed-slot Kani GOTO programs were read by `cbmc --show-goto-functions`
only. Dumps/hashes and selected route blocks are preserved. For both Rand32/64,
the concrete CloneToUninit function calls CopySpec::clone_one, whose only call is
ptr::copy_nonoverlapping::<Self>(source,destination,1). It maps to exact08
clone.rs650/652 and clone/uninit.rs57/61, the TrivialClone specialization. The
GOTO pointer-copy function includes builtin memcpy behavior and safety guards.
This directly confirms the selected compiled branch, rather than merely citing
the default ptr::write(src.clone()) branch. These machine-readable inspections
are evidence of this finite compiled artifact, not a proof of Rust compiler or
GOTO frontend correctness. No new LLVM source mapping was generated.

A structural checker could freeze GOTO IDs/types/source spans, exact one-call
argument forwarding/count1, lack of other side effects and constant Self layout,
then compare with pinned compiler MIR/LLVM when that IR is already available.
The current readout has not established a verified checker or a proof that all
compiler configurations choose the same branch. Derive(Clone,Copy) on original
integer-pair structs remains source evidence; the GOTO specialization settles
which body these particular checked builds selected.

A mathematical copy theorem can use a pre-existing, independently specified
memory semantics: live source readable for sizeofSelf bytes; destination writable
and aligned; disjoint byte ranges; valid initialized Self source; no size/offset
overflow; lifetime/ownership permissions maintained. A copy operation's defining
rule copies precisely those bytes and leaves every other cell unchanged. Under
an independently established representation lemma for the original two integer
fields, the destination decodes to the same state/inc and the source is unchanged.
This is a normal compositional route if the intrinsic semantics are trusted or
proved independently. It must not introduce an opaque clone_to_uninit model or
assume the same postcondition for the API being proved: that would be circular.
No such model or theorem was implemented here.

To make an arbitrary-heap machine claim, missing obligations remain:
1. A formal memory/provenance/permission semantics capable of arbitrary finite
   allocations, not Kani's fixed pointer-object/offset encoding.
2. A proved or explicitly trusted refinement of Rust copy_nonoverlapping to that
   semantics, including uninitialized padding, alignment, nonoverlap, validity
   and pointer provenance. Byte-array value equality alone does not prove this.
3. Verified source/IR correspondence for the selected original blanket and
   TrivialClone specialization, plus layout/representation of Rand32/64.
4. Composition/return/frame proof under that exact semantics, without silently
   importing physical heap behavior into Aeneas's value/backward-function model.

Using the Rust intrinsic specification as an assumption plus manual source/IR
reduction would yield an explicitly conditional argument, not an independent
verified implementation of arbitrary-heap intrinsic semantics. Existing RustBelt/
RefinedRust/separation-logic infrastructure might support an adequate memory
model, but no available installed bridge from this exact Rust08/KaniGOTO artifact
has been established. Feasibility is not completion. No all-API closure is added.

The Layout/all-N experiments failed pointer address consistency at16/48 and8/56
object/offset splits; those logs remain unchanged. The successful fixed-slot
all-state contracts remain valid in their recorded scopes. This alternative
cannot erase those tool limits or relabel them as an arbitrary-heap proof.
