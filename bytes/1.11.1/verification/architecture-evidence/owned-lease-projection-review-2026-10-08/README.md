# Owned-lease draft review

This is an unvalidated private generic-boundary draft, preserved before repair.
It was not adopted as TCB and no completed bytes proof uses it. No VC/native
counterexample run is claimed here: this records the concrete interface flaw
identified by root and independently confirmed by Luna before source integration.

A logic-only TokenLease::lifetime_token projection says nothing about whether
L contains a live affine token. A Snapshot<LifetimeToken> can remember token
metadata after the actual token was ended; a pure projection can return that
snapshot in logic. The draft only checks lifetime IDs, unlike stock FullBorrow
access which requires an actual borrowed LifetimeToken. Model metadata must not
substitute for live permission.

The owned callback also needs the exact input resource. An arbitrary existential
same-lifetime lease can carry unrelated ticket/fraction resources. The first
repair put equality in a universal implication's consequent, requiring every
same-lifetime sibling token to equal the input; that is an impossible precondition
for a real split lifetime. Both defects are interface defects, not bytes laws.

Replace the projection interface with direct actual token argument types, keep
the same input token in the callback contract, and capture other retirement
resources separately. This repair must be translated/proved in the new bounded
source gate. Preserve the token-tied byte read negative and missing-Acquire control.
