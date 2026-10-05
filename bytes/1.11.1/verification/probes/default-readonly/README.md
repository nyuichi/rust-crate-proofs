# Actual default BytesMut immutable-access frontier

This gate extracts current production methods without `bytes_proof_valid_handle`
and without its stronger type invariant. It retains the ordinary Creusot
ownership fields, default unique-or-registered predicate and exact method bodies.
The focused six-file replay proves `as_slice` (34 goals), `kind`, `Deref`, the
packet reader and pointer address extraction. It rejects actual public `AsRef`
at precisely `as_slice`'s required `self.proof_initialized()` fact: the default
BytesMut type invariant supplies field well-formedness but no matching live
initialized ownership. The failed goal is preserved with the proof artifacts.

Thus the immutable physical chain is body-proved under its explicit authority
preconditions. Actual public AsRef remains incomplete until the default handle
invariant and all transitions establish that authority. A Deref body that merely
calls this incomplete AsRef does not establish public read safety independently.
No function body or protocol is marked trusted here, and no stronger trait
precondition has been silently imposed on public callers.

`bash run-proof.sh` translates the extracted default configuration and proves
only these six selected files through the common lock with one 1024 MiB prover.
It is expected to exit nonzero on the recorded AsRef obligation. Supporting
constructors/other extracted methods are present for type context, not part of
this focused proof claim. The native crate-wide frontend still reports Bytes
Send/Sync, Bytes default Deref and BytesMut mutable Deref separately.
