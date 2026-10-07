# Small standard-library support and retained physical proof

Astra reviewed the integrated production diff before its combined gate. No bytes-specific ownership or refcount theorem was moved into trust.

The generic alloc feature exposes existing owned Vec/Box/String and allocating slice/conversion specifications on the actual alloc types. It does not enable std, synchronized concurrency, or SC-DRF in an alloc-only proof. The capacity model applies to the actual Vec logical type rather than its sequence view. Capacity/reserve contracts describe native metadata and normal return; new try_reserve contracts specify both Ok/Err frames without adding termination assumptions. Old infallible Std termination assumptions remain inherited TCB and cannot establish allocator or whole-program liveness.

The opaque numeric base model observes `Vec::as_ptr().addr_logic()`. B1/B2 connect metadata to the actual raw descriptor; bytes Owner share/recover/thaw frames remain body proof obligations. The full affine Recovery and PhysicalRegion preconditions on reconstruction remain necessary. Numeric address equality implies no provenance, permission, injectivity, allocation existence at zero capacity, dereferenceability or stability across reallocating mutation. The production address observer explicitly closes the returned Vec after observing its address, correcting the isolated witness's implicit-Drop gap.

The generic scoped Copy-slot method frames the ordinary mutable slot on creation None, without applying the unexecuted callback's postcondition and without framing unrelated atomic/global state. F:Copy rejects the actual owned-destructor counterexample retained in D11. The method itself is a reviewed trusted Std boundary, not a library body proof. The bytes caller's parent-retained lease, fallback read, retirement and exclusive final-owner XOR are body obligations. Callback exceptions, OS thread scheduling and join termination remain outside the normal-return claim.

Digest and hex models and their sequence/arithmetic helpers remain body checked. They implement the explicit recurrence/encoding API, without claiming generic Hash/Debug compatibility. Portable Release RMW/final Acquire fence and copied-split bodies are preserved. The actual source, effective dependency features and all new Std files must accompany each integrated proof capture; stock installed Std snapshots cannot describe a local modified dependency.

This review admits the combined gate. It is not evidence that the gate passed, full API coverage, or whole-program totality.
