# Generic exact Vec pointer observation candidate

Changed premise: existing numeric base/capacity observer contracts do not
provide raw pointer equality. This private std candidate adds only the exact
mutable getter observer, preservation of pointer/capacity, and no permission,
allocation injectivity, liveness or bytes/refcount fact. Native getter unchanged.
Astra reviewed this generic observation direction. This is not a bytes API proof.
The validated candidate is now provided by the canonical strict std installer.

Positive getter orchestration proves one file. The wrong-Vec negative leaves
exactly vc_wrong_vec unproved; it cannot substitute another Vec's pointer.
This validates contract composition and exclusion only; the standard-library
getter observation is TCB, not a proved allocator or bytes protocol.
The reviewed patch was promoted to verification/std-support/pointer-model.patch
and the strict installer. B1/B2 add exact pointer observation clauses without
new access authority. Atomic/state/Box/Drop facts remain separate.
