# AS trusted boundaries and proof obligations

AS introduces no new trusted function, pointer axiom or ownership law. It adds
explicit nonnull preconditions to two existing generic trusted boundaries and
strengthens the Bytes view predicate whose constructors and callers are body proved.

physical_projection::borrow_empty executes slice::from_raw_parts(pointer, 0).
Its contract now requires an invariant sealed descriptor, exact pointer identity,
and actual pointer nonnull. u8 alignment is one; no allocation authority is
needed for a zero-length slice. It guarantees an empty slice with the selected
borrow lifetime and creates no physical access or recovery capability.

view_pointer::wrapping_bounded executes pointer.wrapping_add(count). Its contract
now requires actual pointer nonnull as well as the existing descriptor identity
and bounds. An unbound descriptor permits only count zero. Allocation-bound
metadata retains its namespace/capacity, exact offset and address relation.
The output remains metadata, not dereference, allocation or ownership authority.

BoundPtr physically stores NonNull<u8>; pinned Std nonnull.rs defines that type's
invariant as nonnull. BoundPtr's unbound metadata invariant alone is true and
previously did not expose that fact. The explicit premises close that interface
gap for these operations. No reachable native null defect is inferred, and no
universal nonnull fact is added as an axiom. Callers must discharge the repaired
premises using their actual constructor, pointer and representation contracts.
new_empty_view already requires nonnull; without_provenance already preserves it.
AS reproves all150 inherited targets; the normal gate passes1328 prover leaves
with zero null/structural leaves and correspondence0.

All other generic physical allocation/free, atomic/event, scoped-history,
callback registration/invoke3, exact slice::is_empty specification and normal
MIR terminal-effect interpretations remain the documented AR/ancestor TCB.
Their full native adequacy is not established merely by successful caller VCs.
Equivalent upstream Std contracts could replace the local physical adapters;
any extra metadata lemma must be body-proved from the actual field and Std
contracts. Native bodies, private Std, production and published ancestors are
unchanged.

The checker must verify the exact three source transformations, full source
inventory, native source/MIR correspondence, package/build/lib routes and the
actual four Cargo input artifacts. A feature, inherited receipt or helper-only
checker result is not canonical admission. Whole-crate, other representations,
owned-empty Clone, concurrent escaping owners, unwind and configuration coverage
remain open; full original architecture remains NOT ADMITTED.

The canonical archive and independent audit are recorded in README.md. All
three exact source transformations,17 other inherited modules, source63, Std110
and the actual four Cargo artifacts are checked. The existing AR native capture
is reused unchanged;76 native controls are replayed and36 AS checker controls
reject in-memory mutations. Three semantic controls yield five exact nonnull
failed tasks, independently reprinted. These do not infer reachable native null
metadata. No extra metadata lemma or nonnull axiom was needed. The local derived
config matches the inherited ar-proof-budget-v1 settings; that reused metadata
records the same actual one-prover/1024MiB/sc-drf-off launcher budget. Absolute
Cargo paths and tool executable payloads remain external/location-bound.
