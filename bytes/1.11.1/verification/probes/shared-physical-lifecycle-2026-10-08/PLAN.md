# Bounded shared physical lifecycle, 2026-10-08

Changed premise: reviewed operation-bound EventAtomic generic synchronization TCB,
plus stock GhostShared<FullBorrow<PhysicalRegion>> and separately retained EndBorrow.
Only the borrow descriptor is permanently shared. Recovering the actual physical
region requires all affine LifetimeTokens and an ended lifetime.

A: two preallocated reader registrations, native refcount 2, Release fetch_sub,
actual Acquire load (not fence), authenticated final publication receipt, full
lifetime recovery and B3. AtView gates retired tokens. Body proofs establish
registration uniqueness and count correspondence. Empty readers remain registered.
This is an interim component, not original constructor/Clone admission.
B follows A: initial count 1, Relaxed clone splitting a residual lifetime pool,
objective live-ID/fraction accounting; no wrap-zero reclamation or bytes protocol axiom.

Positive: actual B1 allocation, overlapping B4 reads, two retirements, B3 cleanup.
Controls: missing Acquire; premature lifetime end; duplicate tickets. Native cases
include empty allocated spare capacity. No automatic Drop/unwind/liveness claim.

Only this directory may change. Original helpers referenced by path, exact hashes
captured. Proof: elevated first, shared flock, one prover, 1024 MiB, no sc-drf.
Preserve first obstruction before edits; two equivalent failures trigger interface
review, third restructure. Do not add trusted protocol laws or increase timeouts.
