# Current assumptions and applicability

The proof contains no new bytes-specific ownership/refcount/free axiom. Bodies
prove both native kind branches, allocation/root framing and exact current views.
It relies on generic physical allocation/free and read permissions, pointer
provenance/distance, atomic/weak-memory contracts, callback registration, and the
reviewed native/shadow source correspondence. These are not proofs of the Rust
compiler, memory model or a general erasure transformation. Exact primitive
source contracts and private Std inputs are retained in results/current.zip.

Clone describes normal return under the native count guard. Exhaustion may abort;
the finite-step witness restores a singleton before each Clone and has no step
quota. The result is scoped, sequential and starts with nonempty Box storage.
Unwind, arbitrary concurrent/escaping owners, other construction paths and the
remaining BytesMut/API surface are not established by this result.

The original positive solver run had diagnostic correspondence status 2. The
later successful native/source/Cargo correspondence is a separate retained record;
reuse of both under identical source is not a new solver or general equivalence
proof. Historical assumptions for other components remain in components.zip.
