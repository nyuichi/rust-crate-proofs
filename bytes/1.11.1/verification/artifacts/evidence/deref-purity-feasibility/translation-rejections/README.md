# Standard Deref purity feasibility

Native Rust checks the reduced program successfully. Vanilla Creusot 0.13
rejects its ordinary Deref implementation because the standard trait requires
check(ghost). With ghost_calls_program enabled, it instead rejects calling a
program(terminates) helper from the ghost implementation. These are compiler
translation diagnostics; no Why3 VCs or physical-access proof were generated.

The example uses a trivial field reader to isolate purity, not the BytesMut
implementation or a trusted physical bridge. Current B4/Perm physical access
operations have ordinary program purity, so marking a real pointer-access
Deref ghost is not a valid workaround. In particular mutable physical writes
must not be erased while their ghost slot ledger changes. This diagnostic
does not claim every possible alternative Deref representation is impossible.

Recorded upstream source: creusot-std/src/std/ops.rs (Deref/DerefMut specs) and
creusot/src/validate/traits.rs (trait purity validation), foundation commit
318615be3b8bbc60d1f6d52469ba5c0bdebed4f1.

Run BYTES_TRANSLATE_ONLY=1 ./scripts/verify-bytes.sh deref-purity-feasibility;
repeat with --features ghost_calls_program. Both intentionally reject.
