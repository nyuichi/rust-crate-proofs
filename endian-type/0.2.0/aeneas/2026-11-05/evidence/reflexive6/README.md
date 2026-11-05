# Original endian wrappers: six reflexive conversion/borrow contracts

For BigEndian<T> and LittleEndian<T>, From<Self> returns exactly its input value;
Borrow<Self> returns a shared view with that same represented value; BorrowMut<Self>
returns that value and the identity backward function for modeled replacement.
All six contracts are generic over every represented source T under the original
implicit Sized bound, not limited to ten integer types, values or byte buffers.
These are reflexive external standard blanket implementations, not the primitive
or cross-endian From conversions already in the primary66, and not std derives.

The safe public UFCS callers in src/lib.rs each contain one original API call.
Official rustc/Charon optimized MIR with regular Inline/RemoveStorageMarkers/
ReferencePropagation/DeadStoreElimination-final/SimplifyLocals-final passes reduces
the calls to identity operations. Before/after Inline MIR preserves each exact
selected wrapper and trait target. Debug MIR dumping changes no LLBC bytes.
Final IR has zero remaining calls. Some core source spans disappear during later
cleanup, so no claim is made that every final LLBC span names its core callee.
Pinned std09 From/Borrow/BorrowMut bodies and these typed MIR scopes supply the
explicit source/compiler correspondence. No source, IR, generated-body, standard
model, type axiom or private observer was changed/introduced.

Six Lean theorem audits use only propext/Quot.sound. A simultaneous namespace/type
check imports byte-identical generated type views used by the old6-operator and
60-conversion groups; all six new audits pass, and four type-identity lemmas use
no axioms. This does not rerun the old66 API proofs or byte/slice Kani jobs. The
old public mappings/source files are unchanged. No source-card binding or primary
Aeneas count increase is asserted; these are compiler-inlined caller contracts.

Borrow values and identity backward functions are Aeneas functional views. They
do not prove physical pointer identity, arbitrary heap/provenance/aliasing/lifetime
properties or behavior of arbitrary caller replacement/Drop side effects. Normal
runtime method behavior is covered; const evaluation and different compiler/
target/configurations are not. Original wrapper nominal identities remain distinct
in Rust even though their Aeneas single-field type views are reducible aliases.
The standard template T is substituted with the whole wrapper Self, never its
inner api.T payload. api-bindings6.json preserves this distinction and one strict
inventory row per target, with no overlap with the old66 local bodies.

The complete original endian-type0.2.0 archive is included/extracted byte-identically.
This run uses nightly2026-09-17/compiler923c95, official Charon08d7c9d and official
Aeneas/librarybe86ab0 on x86_64/default empty crate features. It uses no Kani08
execution and makes no unqualified std08/std09 equivalence claim. The original
crate has no float API; none is invented. Publication remains a separate gate.
