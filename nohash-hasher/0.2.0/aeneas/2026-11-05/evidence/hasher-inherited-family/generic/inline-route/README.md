# Generic debug length-prefix contract through official compiler inlining

The unchanged public std Hasher::write_length_prefix call on NoHashHasher<T>
was extracted through the small generic UFCS caller in src/lib.rs. Official
Charon used optimized MIR with rustc's explicit +Inline pass. Source files were
not rewritten. Including the original external NoHashHasher type resolved its
initial opaque-type extraction failure. Official Aeneas then generated the full
assert/store/flag transition without a Hasher dictionary projection, new model
or generated-body edit.

For arbitrary represented T with the original IsEnabled dictionary, every old
u64 value, Boolean written flag and usize length, exact_transition proves:
already written => assertionFailure; otherwise store length cast to u64 and set
written true. already_written_panics isolates the universal written-state failure.
These are kernel proofs about the compiler-inlined caller, not a direct default-
method source-card binding. AssertionFailure is the extracted debug panic model;
unwinding, panic messages/locations, Rust Drop/global effects and other profiles
are not specified. The separate900-check Kani supplement retains its exact
concrete-type/feature/profile domains; it is not used as a premise here.

Both theorem audits use only propext/Classical.choice/Quot.sound. Simultaneous
import with the old public nohash Proofs module passed with the same audits.
Original core09 hash source, original nohash Rust/Cargo and caller are hashed in
SOURCE-GUARD.json. The single transparent LLBC body retains source spans from
core hash483–485 (actual default call) and nohash250–252 (actual debug write_usize).
That source/IR correspondence and Rust compiler inlining/official extraction are
explicit trust boundaries, not a proof of compiler correctness. Opaque diagnostic
items outside the actual generated proof closure are unused; no opaque generated
function or new axiom is in the final module. Early failed probe/audit logs are
historical diagnostics and not the final successful audit.

The known original-default direct extraction still fails against the official
Hasher model because write_usize is absent. This route does not fix/extend that
model and does not claim production source-binding support. Original crate bytes,
installed tools and existing public mappings remain unchanged.
