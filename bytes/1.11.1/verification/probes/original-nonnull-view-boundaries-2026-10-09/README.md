# Explicit nonnull view boundaries (AS)

AS repairs exactly three proof interfaces over the complete, independently
audited AR positive at commit ccca44a694d6553226149f089996bf16943c9f25:

1. The inherited `view_valid` includes actual pointer nonnull, including Static Empty.
2. `physical_projection::borrow_empty` requires actual pointer nonnull.
3. `view_pointer::wrapping_bounded` requires actual pointer nonnull.

The complete20-file Rust inventory contains17 byte-identical inherited files
and exactly these three reconstructed transformations. Every native function
body, production input and private Std file is unchanged. The new Cargo package
has its own actual compiled-input capture; build/lib routes remain unchanged.
The existing native140-case cursor witness and30 selected MIR bodies are reused
from AR, with no fresh native compilation claimed.

Canonical `evidence/as-positive-canonical-v1.tar.gz` SHA256:
`969a1a0406e10677c1849200284881e043aae9c9bdf29c9b15609783c91a9f74`.
The feature-free normal gate proves all150 inherited targets /1328 prover leaves /
zero null or structural leaves, correspondence0, exclusions{}, diagnosticfalse.
Independent reconstruction verifies1421 regular members, the complete AR/AQ/AP
ancestry, source63, privateStd110, and actual four Cargo artifacts with their
original absolute path joins. Archived main36/native76 controls replay without
a live Cargo target. Tool executable payloads remain external; all eight current
binary hashes separately match the captured manifest. One prover/1024MiB and
sc-drf disabled are captured. See `evidence/AS_CANONICAL_AUDIT.md` and `.json`.

Three semantic controls retain five independently reprinted failed tasks:
removing the view nonnull conjunct fails read_view, slice_view and
slice_cursor_entry; weak empty and wrapping callers each fail exactly their
nonnull precondition. These are contract-sensitivity results, not reachable
native null counterexamples. `BoundPtr` physically stores Std `NonNull<u8>`.
See `evidence/AS_DIAGNOSTIC_AUDIT.md` and `AS_SEMANTIC_CONTROLS.json`.

The first launch stopped before translation because the copied clean selector
used AR's package name; its immutable tool-routing diagnostic is retained.
A checker-control construction used a nonunique predicate fragment; the exact
full view_valid anchor repairs that fixture alone. Neither is a frontend,
semantic-interface or native failure. The first actual source experiment passed
all150 targets without further Rust, predicate or trusted-boundary changes.

The canonical archive retains pre-audit README/TCB text; these two live documents
record the completed audit afterward. Executable/proof inputs are unchanged.

AS adds no new trusted function, pointer axiom or ownership law. Generic physical,
atomic/history, pointer/provenance, ghost registration, Std and normal-MIR
interpretations remain explicit TCB. Full original Bytes/BytesMut/API,
representations, escaping/concurrent owners, unwind and configuration coverage
remain NOT ADMITTED. Continue the user-directed Astra recommendation loop.
