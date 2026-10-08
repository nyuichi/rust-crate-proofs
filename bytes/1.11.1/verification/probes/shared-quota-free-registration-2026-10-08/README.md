# Quota-free fractional registration prerequisite

This isolated experiment checks a registration transition that borrows a live
source ticket, allocates a fresh logical ticket id, splits the residual stock
`LifetimeToken`, inserts its exclusive fraction-map key, and performs the
matching Relaxed atomic increment through the existing generic `EventAtomic`
boundary.

The positive gate proves the constructor, the body-defined map/token update,
and a caller that performs two registrations through the same borrowed source.
Both new tickets remain valid, both are pairwise distinct through the stock
`Fragment::valid_op_lemma`/`Excl` composition rule, and the original source stays
valid. The native single-thread test observes old values `(1, 2)`. The proof does
not claim those exact old values or logical IDs for a general concurrent caller;
the protocol public state intentionally does not expose its private allocation
cursor.

The native guard is preserved exactly: a returned old value greater than
`usize::MAX >> 1` calls `abort`. The checked normal-return contract uses that
same threshold. The invariant tracks an unbounded logical ticket count and
relates it to the native counter modulo `usize::MAX + 1`; it does not prove a
public `Clone` contract through arbitrary wraparound or concurrent aborts.

This is a registration-only prerequisite. Retirement, acquire/release sequence
preservation with repeated registrations, last-owner recovery, typed control
borrowing, physical deallocation, thread transport, and the original public
`Bytes::Clone` integration remain open. The negative controls show the ticket
is affine at the Rust type level and that omitting the fraction-map insertion
breaks protocol preservation.

Evidence is retained under `evidence/`: the final default proof passes 23/23
Coma files (113 prover leaves, zero null leaves), the native caller test passes,
the duplicate-ticket source is rejected with E0382, and the forgotten-map
negative leaves its expected protocol and dependent callback goals unproved.
