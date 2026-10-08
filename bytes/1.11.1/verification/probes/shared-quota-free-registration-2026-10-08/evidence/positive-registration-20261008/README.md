# Positive quota-free registration gate

This immutable capture records the proof gate after the exact native overflow
threshold was used as the normal-return bound. The probe proves its generic
state initialization and registration transition, plus a caller that performs
two registrations while borrowing the same live source ticket. Each new ticket
is proved to have a larger logical id than the source; the source remains valid
and both returned tickets remain valid. It does not claim exact old values or
pairwise freshness between the two returned tickets in the abstract concurrent
model.

The registered transition uses the real `RawAtomic::new` and `EventAtomic`
source from the existing generic event probe, and the real fraction-map proof
module from the shared physical lifecycle probe. The native test observes the
sequential result `(1, 2)` in this single-thread caller. The exact output count
and proof tree are in `positive.tar.gz`; `proof.log` is the wrapper log and
`native-positive.log` is the native test log at the probe root.

Scope remains a registration-only prerequisite. Retirement, release-sequence
preservation across arbitrary registrations, final recovery, physical cleanup,
and public `Bytes::Clone` are not established here.
